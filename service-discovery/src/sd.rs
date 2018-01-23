use std::collections::HashSet;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use exonum::blockchain::Block;
use exonum::blockchain::config::{StoredConfiguration, ValidatorKeys};
use exonum::crypto::PublicKey;
use exonum::helpers::Height;
use exonum_configuration::config_api::{ApiResponseConfigHashInfo, ApiResponseProposePost, ApiResponseVotePost};
use futures::{Future, Stream};
use futures::future;
use futures::stream;
use serde_json;
use hyper;
use hyper::{Request, Response, Body, Method, StatusCode};
use hyper::header::{ContentType, ContentLength};
use hyper::server::Service;
use hyper::client::Client;
use tokio_core::reactor::Handle;

const PROPOSE_HEIGHT_INCREMENT: u64 = 100;

#[derive(Debug, Hash, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct ValidatorInfo {
    public: SocketAddr,
    private: SocketAddr,
    peer: SocketAddr,
    consensus: PublicKey,
    service: PublicKey,
}

pub struct ServiceDiscovery {
    handle: Handle,
    nodes: Arc<RwLock<HashSet<ValidatorInfo>>>,
}

impl ServiceDiscovery {
    pub fn new(handle: Handle) -> Self {
        ServiceDiscovery {
            handle,
            nodes: Arc::new(RwLock::new(HashSet::new())),
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
        let post = body.concat2().and_then(move |v| {
            let publish = match serde_json::from_slice::<ValidatorInfo>(&v) {
                Ok(info) => {
                    let mut nodes = nodes.write().unwrap();
                    eprintln!("Received value: {:?}", &info);
                    nodes.insert(info);
                    ServiceDiscovery::publish_peer(handle, nodes.clone())
                }
                Err(e) => Box::new(future::err(io::Error::from(e).into())),
            };
            publish.and_then(|_| {
                future::ok(Response::new().with_body(serde_json::to_string(&json!{()}).unwrap()))
            })
        });
        Box::new(post)
    }

    fn publish_peer(handle: Handle, nodes: HashSet<ValidatorInfo>)
        -> Box<Future<Item=(), Error=hyper::Error>>
    {
        let api_node = nodes.iter().next().unwrap().clone();
        let config = {
           let client = Client::new(&handle);
           let uri = format!(
               "http://{}/api/services/configuration/v1/configs/actual",
               &api_node.public
           ).parse().unwrap();
           client.get(uri).and_then(|response| {
               response.body().concat2().and_then(|config_data| {
                   let config = serde_json::from_slice::<ApiResponseConfigHashInfo>(&config_data)
                       .map_err(|e| io::Error::from(e).into());
                   future::result(config)
               })
           })
        };

        let height = {
            let client = Client::new(&handle);
            let uri = format!(
                "http://{}/api/explorer/v1/blocks?count=1",
                &api_node.public
            ).parse().unwrap();
            client.get(uri).and_then(|response| {
                response.body().concat2().and_then(|block_data| {
                    future::result(
                        serde_json::from_slice::<Block>(&block_data)
                            .map_err(|e| io::Error::from(e).into())
                    )
                })
            }).map(|block| block.height())
        };

        let propose_handle = handle.clone();
        let validators = nodes.iter().map(|node| ValidatorKeys {
            service_key: node.service,
            consensus_key: node.consensus,
        }).collect();
        let propose = config.join(height)
            .and_then(move |(config, height)| {
                let propose_config = ServiceDiscovery::gen_propose(&config, validators, height);
                let client = Client::new(&propose_handle);
                let uri = format!(
                    "http://{}/api/services/configuration/v1/configs/postpropose",
                    &api_node.private
                ).parse().unwrap();
                let mut req = Request::new(Method::Post, uri);
                req.headers_mut().set(ContentType::json());
                req.headers_mut().set(ContentLength(propose_config.len() as u64));
                req.set_body(propose_config);
                client.request(req)
            }).and_then(|response_stream| {
                response_stream.body().concat2().and_then(|response| {
                    let response = serde_json::from_slice::<ApiResponseProposePost>(&response);
                    future::result(response.map_err(|e| io::Error::from(e).into()))
                })
            });

        let votes_handle = handle.clone();
        let votes_to_send = nodes.len() * (2/3) + 1;
        let votes = propose.and_then(move |response| {
            let iter = nodes.into_iter().take(votes_to_send).map(|node| {
                let vote = response.cfg_hash.to_hex();
                let client = Client::new(&votes_handle);
                let uri = format!(
                    "http://{}/api/services/configuration/v1/configs/{}/postvote",
                    &node.private,
                    &vote,
                ).parse().unwrap();
                let req = Request::new(Method::Post, uri);
                client.request(req)
            });
            stream::futures_unordered(iter).for_each(|response| {
                response.body().concat2().and_then(|data| {
                    let parsed = serde_json::from_slice::<ApiResponseVotePost>(&data);
                    future::result(parsed.map_err(|e| io::Error::from(e).into()))
                        .and_then(|vote_info| {
                            eprintln!("Voted, tx_hash: {:?}.", vote_info.tx_hash);
                            future::ok(())
                        })
                })
            })
        });

        Box::new(votes)
    }

    fn gen_propose(
        old_config: &ApiResponseConfigHashInfo,
        validators: Vec<ValidatorKeys>,
        current_height: Height
    ) -> String {
        let config = StoredConfiguration {
            previous_cfg_hash: old_config.hash,
            validator_keys: validators,
            actual_from: Height(current_height.0 + PROPOSE_HEIGHT_INCREMENT),
            ..old_config.config.clone()
        };
        serde_json::to_string(&config).unwrap()
    }
}

impl Service for ServiceDiscovery {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        eprintln!("Got request: {:?}", req);
        let response = match (req.method(), req.path()) {
            (&Method::Get, "/nodes") => self.get_nodes(),
            (&Method::Post, "/nodes") => self.post_node(req.body()),
            _ => Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound),
            )),
        };
        response
    }
}

