extern crate exonum;

use exonum::crypto::PublicKey;
use exonum::storage::{Fork, MapIndex};

use service::SERVICE_NAME;
use service::asset::Asset;
use service::wallet::Wallet;

pub struct WalletSchema<'a>(&'a mut Fork);

impl<'a> WalletSchema<'a> {
    pub fn wallets(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        let key = SERVICE_NAME.to_string().replace("/", "_") + ".wallets";
        MapIndex::new(key, self.0)
    }

    // Utility method to quickly get a separate wallet from the storage.
    // If wallet doesn't exist, create new one
    pub fn wallet(&mut self, pub_key: &PublicKey) -> Wallet {
        match self.wallets().get(pub_key) {
            Some(wallet) => wallet,
            None => self.create_wallet(pub_key),
        }
    }

    fn create_wallet(&mut self, pub_key: &PublicKey) -> Wallet {
        let assets: Vec<Asset> = vec![];
        let wallet = Wallet::new(pub_key, 0, assets);
        println!("No wallet in schema. Creating the wallet: {:?}", wallet);
        self.wallets().put(pub_key, wallet.clone());
        wallet
    }

    pub fn map<F, T>(view: &'a mut Fork, f: F) -> T
    where
        F: FnOnce(Self) -> T + 'a,
        T: 'a,
    {
        f(WalletSchema(view))
    }
}
