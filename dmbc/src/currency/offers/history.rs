use exonum::crypto::Hash;
use exonum::storage::{Fork, MapIndex, Snapshot};
use currency::SERVICE_NAME;

encoding_struct! {
    struct OfferHistory {
        tx_hash: &Hash,
        amount: u64,
    }
}

/// The schema for accessing wallets data.
pub struct Schema<S>(pub S)
    where
        S: AsRef<Snapshot>;

impl<S> Schema<S>
    where
        S: AsRef<Snapshot>,
{
    /// Internal `MapIndex` with immutable access.
    pub fn index(self) -> MapIndex<S, Hash, Vec<OfferHistory>> {
        let key = SERVICE_NAME.to_string() + ".offer_history";
        MapIndex::new(key, self.0)
    }

    /// Fetch state for the specified wallet from the snapshot.
    pub fn fetch(self, tx_hash: &Hash) -> Vec<OfferHistory> {
        self.index()
            .get(tx_hash)
            .unwrap_or_else(|| vec![] )
    }
}

impl<'a> Schema<&'a mut Fork> {

    /// Internal `MapIndex` with mutable access.
    pub fn index_mut(&mut self) -> MapIndex<&mut Fork, Hash, Vec<OfferHistory>> {
        let key = SERVICE_NAME.to_string() + ".offer_history";
        MapIndex::new(key, &mut *self.0)
    }

    /// Store the new state for a wallet in the database.
    pub fn store(&mut self, tx_hash: &Hash, offer_history: Vec<OfferHistory>) {
        self.index_mut().put(tx_hash, offer_history);
    }
}
