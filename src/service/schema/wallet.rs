extern crate exonum;

use exonum::crypto::PublicKey;
use exonum::storage::{Fork, MapIndex};

use service::SERVICE_NAME;
use service::wallet::{Asset, Wallet};

pub struct WalletSchema<'a>(&'a mut Fork);

impl<'a> WalletSchema<'a> {
    pub fn wallets(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        let key = SERVICE_NAME.to_string().replace("/", "_") + ".wallets";
        MapIndex::new(key, self.0)
    }

    // Utility method to quickly get a separate wallet from the storage
    pub fn wallet(&mut self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }

    pub fn create_wallet(&mut self, pub_key: &PublicKey) -> Wallet {
        match self.wallet(pub_key) {
            None => {
                let assets: Vec<Asset> = vec![];
                let wallet = Wallet::new(pub_key, 0, assets);
                println!("Create the wallet: {:?}", wallet);
                self.wallets().put(pub_key, wallet.clone());
                wallet
            }
            Some(wallet) => wallet,
        }
    }

    pub fn map<F, T>(view: &'a mut Fork, f: F) -> T
        where F: FnOnce(Self) -> T + 'a, T: 'a
    {
        f(WalletSchema(view))
    }
}
