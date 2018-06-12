use std::error::Error;

use exonum::crypto::PublicKey;
use futures::{future, Future, Stream};
use hyper;
use hyper::{Body, Method, Request, Response, StatusCode};
use serde_json as json;

use nodes;
use nodes::{NodeInfo, NodeKeys};

const _PROPOSE_HEIGHT_INCREMENT: u64 = 25; // TODO

#[derive(Debug, Hash, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ValidatorInfo {
    public: String,
    private: String,
    peer: String,
    consensus: PublicKey,
    service: PublicKey,
}

impl ValidatorInfo {
    fn into_kv(self) -> (NodeKeys, NodeInfo) {
        let keys = NodeKeys {
            consensus: self.consensus,
            service: self.service,
        };
        let info = NodeInfo {
            public: self.public,
            private: self.private,
            peer: self.peer,
        };
        (keys, info)
    }
}

pub type ResponseFuture = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send + 'static>;

pub fn new(req: Request<Body>) -> ResponseFuture {
    eprintln!("Got request: {:?}", req);
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/nodes") => get_nodes(),
        (&Method::POST, "/nodes") => post_node(req.into_body()),
        _ => Box::new(future::ok(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap(),
        )),
    }
}

fn get_nodes() -> ResponseFuture {
    match json::to_string_pretty(&nodes::list()) {
        Ok(nodes) => Box::new(future::ok(Response::new(nodes.into()))),
        Err(e) => {
            eprintln!("Error when parsing GET: {}", e);
            Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::IM_A_TEAPOT)
                    .body(json::to_string_pretty(&json!{()}).unwrap().into())
                    .unwrap(),
            ))
        }
    }
}

fn update_peer(_info: ValidatorInfo) -> ResponseFuture {
    Box::new(future::ok(Response::new(Body::empty())))
}

fn post_node(body: Body) -> ResponseFuture {
    let post = body
        .concat2()
        .and_then(move |v| match json::from_slice::<ValidatorInfo>(&v) {
            Ok(info) => update_peer(info),
            Err(e) => Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(e.description().to_string().into())
                    .unwrap(),
            )),
        });
    Box::new(post)
}

