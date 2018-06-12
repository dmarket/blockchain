extern crate futures;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate exonum;
extern crate exonum_configuration;
extern crate toml;
#[macro_use]
extern crate lazy_static;
extern crate tokio;

mod config;
mod nodes;
mod sd;

use futures::Future;
use hyper::server::Server;
use hyper::service;

fn main() {
    let addr = config::get().listen_address().parse().unwrap();

    let server = Server::bind(&addr)
        .serve(|| service::service_fn(sd::new))
        .map_err(|e| eprintln!("serve failed: {}", e));

    hyper::rt::run(server);
}
