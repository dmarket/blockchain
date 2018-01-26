#![feature(type_ascription)]

extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate tokio_timer;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate exonum;
extern crate exonum_configuration;

mod sd;

use futures::future;
use futures::{Future, Stream};
use hyper::server::Http;

use sd::ServiceDiscovery;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let server = core.handle();
    let client = core.handle();

    let discovery = ServiceDiscovery::new(client.clone());

    let serve = Http::new()
        .serve_addr_handle(&addr, &server, move || Ok(discovery.clone()))
        .unwrap();

    let server2 = server.clone();
    server.spawn(
        serve
            .for_each(move |conn| {
                server2.spawn(
                    conn.map(|_| ())
                        .map_err(|err| eprintln!("Serve error: {:?}", err)),
                );
                Ok(())
            })
            .map_err(|_| ()),
    );

    core.run(future::empty::<(), ()>()).unwrap();
}
