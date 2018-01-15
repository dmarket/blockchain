use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use exonum::crypto::PublicKey;
use futures::future;
use futures::future::Future;
use futures::stream::Stream;
use serde_json;
use hyper;
use hyper::{Body, Method, StatusCode};
use hyper::server::{Request, Response, Service};

#[derive(Debug, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct ValidatorInfo {
    public: SocketAddr,
    private: SocketAddr,
    peer: SocketAddr,
    consensus: PublicKey,
    service: PublicKey,
}

pub struct ServiceDiscovery {
    nodes: Arc<RwLock<HashSet<ValidatorInfo>>>,
}

impl ServiceDiscovery {
    pub fn new() -> Self {
        ServiceDiscovery {
            nodes: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

impl ServiceDiscovery {
    fn get_nodes(&self) -> <Self as Service>::Future {
        match serde_json::to_string_pretty(&*self.nodes.read().unwrap()) {
            Ok(nodes) => Box::new(future::ok(Response::new().with_body(nodes))),
            Err(e) => {
                println!("error in GET: {}", e);
                Box::new(future::ok(
                    Response::new().with_status(StatusCode::ImATeapot),
                ))
            }
        }
    }

    fn post_node(&self, body: Body) -> <Self as Service>::Future {
        let nodes = Arc::clone(&self.nodes);
        Box::new(body.concat2().and_then(move |v| {
            match serde_json::from_slice::<ValidatorInfo>(&v) {
                Ok(info) => {
                    println!("got value: {:?}", &info);
                    nodes.write().unwrap().insert(info);
                }
                Err(e) => println!("error in POST: {}", e),
            };
            future::ok(Response::new())
        }))
    }
}

impl Service for ServiceDiscovery {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        println!("Got request: {:?}", req);
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

