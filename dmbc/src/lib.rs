#![allow(dead_code)]
extern crate bodyparser;
#[macro_use]
extern crate exonum;
extern crate exonum_configuration;
#[cfg(test)]
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate nats;
extern crate router;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate unicase;
extern crate uuid;
extern crate chrono;

pub mod config;
pub mod keys;
pub mod service;
