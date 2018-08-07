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
// externing a deprecated crate because of hyper =__=
extern crate tokio_core;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

mod config;
mod keeper;
mod nodes;
mod server;
mod log;

use futures::{Future, IntoFuture};
use futures::Stream;
use hyper::service;
use hyper::server::conn::Http;
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

fn main() {
    let addr = config::get().listen_address().parse().unwrap();

    info!(log::ROOT, "Starting server"; "listen_address" => %addr);

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let http = Http::new();
    let listener = TcpListener::bind(&addr, &handle).expect(&format!("Failed to bind to {}", &addr));

    let server = listener.incoming().for_each(move |(socket, remote_addr)| {
        handle.spawn(
            http.serve_connection(socket, service::service_fn(move |req| server::serve(req, remote_addr)))
                .map(|_|())
                .map_err(|_|())
        );
        Ok(()).into_future()
    });

    let keeper = keeper::new()
        .map_err(|e| error!(log::ROOT, "Keeper failed"; "error" => %e));

    let handle = core.handle();
    core.run(futures::lazy(|| {
        handle.spawn(keeper);
        server
    })).expect("Run unsuccessfull");
}

