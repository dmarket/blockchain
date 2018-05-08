#[macro_use]
extern crate exonum;
extern crate libc;
extern crate serde;
extern crate serde_json;
extern crate extprim;
extern crate uuid;

#[macro_use]
mod macros;
mod error;
mod transactions;
mod assets;
mod decimal;
mod service;