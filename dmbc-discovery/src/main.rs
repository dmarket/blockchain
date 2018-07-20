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
extern crate tokio;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

mod config;
mod keeper;
mod nodes;
mod server;
mod log;

use futures::Future;
use hyper::server::Server;
use hyper::service;

fn main() {
    let addr = config::get().listen_address().parse().unwrap();

    info!(log::ROOT, "Starting server"; "listen_address" => %addr);

    let server = Server::bind(&addr)
        .serve(|| service::service_fn(|req| server::new(req)))
        .map_err(|e| error!(log::ROOT, "Serve failed"; "error" => %e));

    let keeper = keeper::new()
        .map_err(|e| error!(log::ROOT, "Keeper failed"; "error" => %e));

    hyper::rt::run(futures::lazy(|| {
        hyper::rt::spawn(keeper);
        server
    }));
}
