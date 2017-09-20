extern crate exonum;

use exonum::crypto::PublicKey;
use exonum::blockchain;
use exonum::storage::{Fork, MapIndex};

use service::wallet::Wallet;
use super::SERVICE_ID;

pub struct CurrencySchema<'a> {
    pub view: &'a mut Fork,
}


impl<'a> CurrencySchema<'a> {
    pub fn wallets(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        let prefix = blockchain::gen_prefix(SERVICE_ID, 0, &());
        MapIndex::new(prefix, self.view)
    }

    // Utility method to quickly get a separate wallet from the storage
    pub fn wallet(&mut self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }
}
