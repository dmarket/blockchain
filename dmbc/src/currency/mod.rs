//! The currency service.

pub mod api;
pub mod assets;
pub mod configuration;
pub mod error;
pub mod status;
pub mod transactions;
pub mod wallet;

mod nats;
mod service;

pub use currency::service::{Service, SERVICE_ID, SERVICE_NAME};
