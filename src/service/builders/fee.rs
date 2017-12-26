use service::asset::{Fee, FeeType, Fees};

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

    pub fn trade(self, value: u64, pattern: FeeType) -> Self {
        Builder {
            trade: Some(Fee::new(value, &pattern.to_string())),
            ..self
        }
    }

    pub fn exchange(self, value: u64, pattern: FeeType) -> Self {
        Builder {
            exchange: Some(Fee::new(value, &pattern.to_string())),
            ..self
        }
    }

    pub fn transfer(self, value: u64, pattern: FeeType) -> Self {
        Builder {
            transfer: Some(Fee::new(value, &pattern.to_string())),
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
