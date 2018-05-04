#![allow(missing_docs)]

use decimal::UFract64;
use currency::assets::{Fee, Fees};

pub struct Builder {
    trade: Option<Fee>,
    exchange: Option<Fee>,
    transfer: Option<Fee>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            trade: None,
            exchange: None,
            transfer: None,
        }
    }

    pub fn trade(self, tax: u64, fraction: UFract64) -> Self {
        Builder {
            trade: Some(Fee::new(tax, fraction)),
            ..self
        }
    }

    pub fn exchange(self, tax: u64, fraction: UFract64) -> Self {
        Builder {
            exchange: Some(Fee::new(tax, fraction)),
            ..self
        }
    }

    pub fn transfer(self, tax: u64, fraction: UFract64) -> Self {
        Builder {
            transfer: Some(Fee::new(tax, fraction)),
            ..self
        }
    }

    pub fn build(self) -> Fees {
        self.validate();
        Fees::new(
            self.trade.unwrap(),
            self.exchange.unwrap(),
            self.transfer.unwrap(),
        )
    }

    fn validate(&self) {
        assert!(self.trade.is_some());
        assert!(self.exchange.is_some());
        assert!(self.transfer.is_some());
    }
}
