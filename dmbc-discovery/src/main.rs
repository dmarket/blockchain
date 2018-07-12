extern crate futures;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate exonum;
extern crate exonum_configuration;
extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate tokio;

mod config;
mod keeper;
mod nodes;
mod server;

use futures::Future;
use hyper::server::Server;
use hyper::service;

fn main() {
    let addr = config::get().listen_address().parse().unwrap();

    let server = Server::bind(&addr)
        .serve(|| service::service_fn(server::new))
        .map_err(|e| eprintln!("serve failed: {}", e));

    let keeper = keeper::new()
        .map_err(|e| eprintln!("keeper failed: {}", e));

    hyper::rt::run(futures::lazy(|| {
        hyper::rt::spawn(keeper);
        server
    }));
}
