//! This crate defines types and operations required for running a blockchain
//! node and external services.

#![allow(dead_code)]
extern crate bodyparser;
#[macro_use]
extern crate exonum;
extern crate exonum_configuration;
extern crate exonum_rocksdb;
#[cfg(test)]
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
#[macro_use]
extern crate log;
extern crate nats;
extern crate router;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate chrono;
extern crate unicase;
extern crate uuid;
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate lazy_static;
extern crate percent_encoding;
extern crate extprim;

pub mod config;
pub mod currency;
pub mod decimal;
