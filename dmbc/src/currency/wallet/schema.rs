use exonum::storage::{Fork, Snapshot, MapIndex};
use exonum::crypto::PublicKey;

use currency::SERVICE_NAME;
use currency::wallet::Wallet;

/// The schema for accessing wallets data.
pub struct Schema<S>(pub S) where S: AsRef<Snapshot>;

impl<S> Schema<S>
where
    S: AsRef<Snapshot>
{
    pub fn index(self) -> MapIndex<S, PublicKey, Wallet> {
        let key = SERVICE_NAME.to_string() + "_v1.wallets";
        MapIndex::new(key, self.0)
    }

    /// Fetch state for the specified wallet from the snapshot.
    pub fn fetch(self, pub_key: &PublicKey) -> Wallet {
        self.index().get(pub_key).unwrap_or_else(|| Wallet::new_empty())
    }
}

impl<'a> Schema<&'a mut Fork> {
    pub fn index_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        let key = SERVICE_NAME.to_string() + "_v1.wallets";
        MapIndex::new(key, &mut*self.0)
    }

    /// Store the new state for a wallet in the database.
    pub fn store(&mut self, pub_key: &PublicKey, wallet: Wallet) {
        self.index_mut().put(pub_key, wallet);
    }

    /// Remove wallet state from the database.
    pub fn remove(&mut self, pub_key: &PublicKey) {
        self.index_mut().remove(pub_key);
    }
}

