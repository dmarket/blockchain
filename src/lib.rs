#![allow(dead_code)]
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate exonum;
extern crate exonum_configuration;
extern crate router;
extern crate bodyparser;
extern crate iron;
extern crate nats;

pub mod config;
pub mod keys;
pub mod service;

#[cfg(test)]
pub mod test;
