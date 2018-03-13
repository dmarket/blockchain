#![allow(missing_docs)]
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

    pub fn trade(self, tax: u64, ratio: u64) -> Self {
        Builder {
            trade: Some(Fee::new(tax, ratio)),
            ..self
        }
    }

    pub fn exchange(self, tax: u64, ratio: u64) -> Self {
        Builder {
            exchange: Some(Fee::new(tax, ratio)),
            ..self
        }
    }

    pub fn transfer(self, tax: u64, ratio: u64) -> Self {
        Builder {
            transfer: Some(Fee::new(tax, ratio)),
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
