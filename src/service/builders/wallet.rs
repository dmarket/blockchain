use exonum::crypto::PublicKey;

use service::asset::{Asset, AssetID};
use service::wallet::Wallet;

pub struct Builder {
    public_key: Option<PublicKey>,
    balance: u64,
    assets: Vec<Asset>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            public_key: None,
            balance: 0,
            assets: Vec::new(),
        }
    }

    pub fn key(self, public_key: PublicKey) -> Self {
        Builder {
            public_key: Some(public_key),
            ..self
        }
    }

    pub fn balance(self, balance: u64) -> Self {
        Builder { balance, ..self }
    }

    pub fn add_asset(self, name: &str, amount: u32) -> Self {
        assert!(self.public_key.is_some());
        let id = AssetID::new(name, self.public_key.as_ref().unwrap()).unwrap();
        let asset = Asset::new(id, amount);
        self.add_asset_value(asset)
    }

    pub fn add_asset_value(mut self, asset: Asset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn build(self) -> Wallet {
        self.validate();
        Wallet::new(self.public_key.as_ref().unwrap(), self.balance, self.assets)
    }

    fn validate(&self) {
        assert!(self.public_key.is_some());
    }
}
