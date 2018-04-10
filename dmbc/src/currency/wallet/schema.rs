use exonum::crypto::PublicKey;
use exonum::storage::{Fork, MapIndex, Snapshot};

use currency::wallet::Wallet;
use currency::SERVICE_NAME;

/// The schema for accessing wallets data.
pub struct Schema<S>(pub S)
where
    S: AsRef<Snapshot>;

impl<S> Schema<S>
where
    S: AsRef<Snapshot>,
{
    /// Internal `MapIndex` with immutable access.
    pub fn index(self) -> MapIndex<S, PublicKey, Wallet> {
        let key = SERVICE_NAME.to_string() + ".wallets";
        MapIndex::new(key, self.0)
    }

    /// Fetch state for the specified wallet from the snapshot.
    pub fn fetch(self, pub_key: &PublicKey) -> Wallet {
        self.index()
            .get(pub_key)
            .unwrap_or_else(|| Wallet::new_empty())
    }
}

impl<'a> Schema<&'a mut Fork> {
    /// Internal `MapIndex` with mutable access.
    pub fn index_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        let key = SERVICE_NAME.to_string() + ".wallets";
        MapIndex::new(key, &mut *self.0)
    }

    /// Store the new state for a wallet in the database.
    pub fn store(&mut self, pub_key: &PublicKey, wallet: Wallet) {
        match (wallet.balance(), wallet.assets().len()) {
            (0, 0) => self.remove(pub_key),
            (_, _) => self.index_mut().put(pub_key, wallet),
        };
    }

    /// Remove wallet state from the database.
    pub fn remove(&mut self, pub_key: &PublicKey) {
        self.index_mut().remove(pub_key);
    }
}
