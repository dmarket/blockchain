use exonum::storage::{Fork, Snapshot, MapIndex};
use exonum::crypto::PublicKey;

use currency::wallet::Wallet;

/// The schema for accessing wallets data.
pub struct Schema<S>(pub S) where S: AsRef<Snapshot>;

impl<S> Schema<S>
where
    S: AsRef<Snapshot>
{
    pub fn index(&self) -> MapIndex<S, PublicKey, Wallet> {
        unimplemented!()
    }

    /// Fetch state for the specified wallet from the snapshot.
    pub fn fetch(&self, _pub_key: &PublicKey) -> Option<Wallet> {
        unimplemented!()
    }
}

impl<'a> Schema<&'a mut Fork> {
    pub fn index_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        unimplemented!()
    }

    /// Store the new state for a wallet in the database.
    pub fn store(&mut self, _pub_key: &PublicKey, _wallet: Wallet) {
        unimplemented!()
    }

    /// Remove wallet state from the database.
    pub fn remove(&mut self) {
        unimplemented!()
    }
}

