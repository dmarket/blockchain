use exonum::crypto::PublicKey;

use currency::asset::{Asset, AssetId};
use currency::wallet::Wallet;

pub struct Builder {
    data: Option<String>,
    amount: u32,
    creator: Option<PublicKey>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            data: None,
            amount: 0,
            creator: None,
        }
    }

    pub fn data(self, data: &str) -> Self {
        Builder {
            data: Some(data.to_string()),
            ..self
        }
    }

    pub fn amount(self, amount: u32) -> Self {
        Builder { amount, ..self }
    }

    pub fn creator(self, creator: PublicKey) -> Self {
        Builder {
            creator: Some(creator),
            ..self
        }
    }

    pub fn creator_wallet(self, creator: &Wallet) -> Self {
        let key = creator.pub_key().clone();
        self.creator(key)
    }

    pub fn build(self) -> Asset {
        self.validate();
        let id = AssetId::new(self.data.as_ref().unwrap(), self.creator.as_ref().unwrap()).unwrap();
        Asset::new(id, self.amount)
    }

    fn validate(&self) {
        assert!(self.data.is_some());
        assert_ne!(self.amount, 0);
        assert!(self.creator.is_some());
    }
}
