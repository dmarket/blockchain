use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::{Read, Write};
use std::fs::File;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use exonum::blockchain::Block;
use exonum::blockchain::config::{StoredConfiguration, ValidatorKeys};
use exonum::crypto::PublicKey;
use exonum::helpers::Height;
use exonum_configuration::config_api::{ApiResponseConfigHashInfo, ApiResponseProposePost,
                                       ApiResponseVotePost};
use futures::{Future, Stream};
use futures::future;
use futures::stream;
use serde_json;
use hyper;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::header::{ContentLength, ContentType};
use hyper::server::Service;
use hyper::client::Client;
use tokio_core::reactor::Handle;
use tokio_timer::Timer;
use toml;

const PROPOSE_HEIGHT_INCREMENT: u64 = 25;

type PKeys = String;

#[derive(Debug, Hash, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct ValidatorInfo {
    public: SocketAddr,
    private: SocketAddr,
    peer: SocketAddr,
    consensus: PublicKey,
    service: PublicKey,
}

impl ValidatorInfo {
    pub fn keys(&self) -> PKeys {
        String::new() + &self.consensus.to_hex() + &self.service.to_hex()
    }
}

#[derive(Clone, Debug)]
pub struct ServiceDiscovery {
    handle: Handle,
    timer: Timer,
    nodes: Arc<RwLock<HashMap<PKeys, ValidatorInfo>>>,
}

impl ServiceDiscovery {
    pub fn new(handle: Handle) -> Self {
        let nodes = ServiceDiscovery::load_peers().unwrap_or_default();
        ServiceDiscovery {
            handle,
            timer: Timer::default(),
            nodes: Arc::new(RwLock::new(nodes)),
        }
    }

    fn get_nodes(&self) -> <Self as Service>::Future {
        match serde_json::to_string_pretty(&*self.nodes.read().unwrap()) {
            Ok(nodes) => Box::new(future::ok(Response::new().with_body(nodes))),
            Err(e) => {
                eprintln!("Error when parsing GET: {}", e);
                Box::new(future::ok(
                    Response::new()
                        .with_status(StatusCode::ImATeapot)
                        .with_body(serde_json::to_string(&json!{()}).unwrap()),
                ))
            }
        }
    }

    fn post_node(&self, body: Body) -> <Self as Service>::Future {
        let nodes = Arc::clone(&self.nodes);
        let handle = self.handle.clone();
        let timer = self.timer.clone();
        let post = body.concat2().and_then(move |v| {
            match serde_json::from_slice::<ValidatorInfo>(&v) {
                Ok(info) => {
                    let mut nodes = nodes.write().unwrap();
                    eprintln!("Received value: {:?}", &info);
                    nodes.insert(info.keys(), info);
                    ServiceDiscovery::publish_peer(&handle, timer, nodes.clone(), info);
                    if let Err(e) = ServiceDiscovery::save_peers(&nodes) {
                        eprintln!("Error when saving peers: {}", e);
                    }
                    future::ok(Response::new().with_status(StatusCode::Ok))
                }
                Err(e) => future::err(io::Error::from(e).into()),
            }
        });
        Box::new(post)
    }

    fn publish_peer(
        handle: &Handle,
        timer: Timer,
        nodes: HashMap<PKeys, ValidatorInfo>,
        new_node: ValidatorInfo,
    ) {
        let api_node = match nodes.iter().filter(|&(k, _)| *k != new_node.keys()).next() {
            Some((_, &node)) => node,
            None => return,
        };

        let config = {
            let client = Client::new(handle);
            let uri = format!(
                "http://{}/api/services/configuration/v1/configs/actual",
                &api_node.public
            ).parse()
                .unwrap();
            client.get(uri).and_then(|response| {
                response.body().concat2().and_then(|config_data| {
                    let config = serde_json::from_slice::<ApiResponseConfigHashInfo>(&config_data)
                        .map_err(|e| {
                            eprintln!("config error: {}", &e);
                            io::Error::from(e).into()
                        });
                    config
                })
            })
        };

        let height = {
            let client = Client::new(handle);
            let uri = format!("http://{}/api/explorer/v1/blocks?count=1", &api_node.public)
                .parse()
                .unwrap();
            client
                .get(uri)
                .and_then(|response| {
                    response.body().concat2().and_then(|block_data| {
                        eprintln!("Got block: {}", &String::from_utf8_lossy(&block_data));
                        serde_json::from_slice::<Vec<Block>>(&block_data)
                            .map(|mut v| v.remove(0))
                            .map_err(|e| {
                                eprintln!("height error: {}", &e);
                                io::Error::from(e).into()
                            })
                    })
                })
                .map(|block| block.height())
        };

        let propose_handle = handle.clone();
        let validators = nodes
            .iter()
            .map(|(_, info)| ValidatorKeys {
                consensus_key: info.consensus,
                service_key: info.service,
            })
            .collect();
        let propose_sleep = timer.sleep(Duration::new(5, 0));
        let propose = propose_sleep
            .map_err(|_| hyper::error::Error::Timeout)
            .and_then(move |_| {
                config
                    .join(height)
                    .and_then(move |(config, height)| {
                        let propose_config =
                            ServiceDiscovery::gen_propose(&config, validators, height);
                        eprintln!("Proposing config: {}", &propose_config);
                        let client = Client::new(&propose_handle);
                        let uri = format!(
                            "http://{}/api/services/configuration/v1/configs/postpropose",
                            &api_node.private
                        ).parse()
                            .unwrap();
                        let mut req = Request::new(Method::Post, uri);
                        req.headers_mut().set(ContentType::json());
                        req.headers_mut()
                            .set(ContentLength(propose_config.len() as u64));
                        req.set_body(propose_config);
                        client.request(req)
                    })
                    .and_then(|response_stream| {
                        response_stream.body().concat2().and_then(|response| {
                            let response =
                                serde_json::from_slice::<ApiResponseProposePost>(&response);
                            response.map_err(|e| {
                                eprintln!("propose error: {}", &e);
                                io::Error::from(e).into()
                            })
                        })
                    })
            });

        let votes_handle = handle.clone();
        let votes_to_send = (nodes.len() - 1) * 2 / 3 + 1;
        let votes_timer = timer.clone();
        let votes = propose
            .then(move |p| {
                votes_timer
                    .sleep(Duration::new(5, 0))
                    .then(move |_| future::result(p))
            })
            .and_then(move |response| {
                let iter = nodes
                    .into_iter()
                    .filter(|&(_, info)| info != new_node)
                    .take(votes_to_send)
                    .map(|(_, node)| {
                        let vote = response.cfg_hash.to_hex();
                        let client = Client::new(&votes_handle);
                        let uri = format!(
                            "http://{}/api/services/configuration/v1/configs/{}/postvote",
                            &node.private, &vote,
                        ).parse()
                            .unwrap();
                        let req = Request::new(Method::Post, uri);
                        client.request(req)
                    });
                stream::futures_unordered(iter).for_each(|response| {
                    response.body().concat2().and_then(|data| {
                        let parsed = serde_json::from_slice::<ApiResponseVotePost>(&data);
                        parsed
                            .map_err(|e| {
                                eprintln!("height error: {}", &e);
                                io::Error::from(e).into()
                            })
                            .and_then(|vote_info| {
                                eprintln!("Voted, tx_hash: {:?}.", vote_info.tx_hash);
                                Ok(())
                            })
                    })
                })
            });

        handle.spawn(
            votes
                .map_err(|e| {
                    eprintln!("Error in publish_peer: {}", e);
                    ()
                })
                .map(|_| ()),
        );
    }

    fn gen_propose(
        old_config: &ApiResponseConfigHashInfo,
        validators: Vec<ValidatorKeys>,
        current_height: Height,
    ) -> String {
        let config = StoredConfiguration {
            previous_cfg_hash: old_config.hash,
            validator_keys: validators,
            actual_from: Height(current_height.0 + PROPOSE_HEIGHT_INCREMENT),
            ..old_config.config.clone()
        };
        serde_json::to_string(&config).unwrap()
    }

    fn save_peers(peers: &HashMap<PKeys, ValidatorInfo>) -> io::Result<()> {
        // TODO: make this configurable.
        let mut file = File::create("./etc/discovery-peers.toml")?;
        let ser = toml::to_string_pretty(peers).unwrap();
        file.write(ser.as_bytes()).map(|_| ())
    }

    fn load_peers() -> Result<HashMap<PKeys, ValidatorInfo>, Box<Error>> {
        let mut file = File::open("./etc/discovery-peers.toml")?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(Box::new)?;
        toml::from_slice(&data).map_err(|e| e.into())
    }
}

impl Service for ServiceDiscovery {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        eprintln!("Got request: {:?}", req);
        match (req.method(), req.path()) {
            (&Method::Get, "/nodes") => self.get_nodes(),
            (&Method::Post, "/nodes") => self.post_node(req.body()),
            _ => Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound),
            )),
        }
    }
}

