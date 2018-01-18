extern crate futures;
extern crate hyper;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate exonum;

mod sd;

use sd::ServiceDiscovery;
use hyper::server::Http;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new()
        .bind(&addr, || Ok(ServiceDiscovery::new()))
        .unwrap();
    server.run().unwrap();
}
