use exonum::crypto::Hash;
use exonum::storage::{Fork, MapIndex, Snapshot};
use currency::SERVICE_NAME;

encoding_struct! {
    struct HistoryOffer {
        tx_hash: &Hash,
        amount: u64,
    }
}

encoding_struct!{
    struct HistoryOffers {
        history: Vec<HistoryOffer>,
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
    pub fn index(self) -> MapIndex<S, Hash, HistoryOffers> {
        let key = SERVICE_NAME.to_string() + ".history_offer";
        MapIndex::new(key, self.0)
    }

    /// Fetch state for the specified wallet from the snapshot.
    pub fn fetch(self, tx_hash: &Hash) -> HistoryOffers {
        self.index()
            .get(tx_hash)
            .unwrap_or_else(|| HistoryOffers::new(vec![]) )
    }
}

impl<'a> Schema<&'a mut Fork> {

    /// Internal `MapIndex` with mutable access.
    pub fn index_mut(&mut self) -> MapIndex<&mut Fork, Hash, HistoryOffers >{
        let key = SERVICE_NAME.to_string() + ".history_offer";
        MapIndex::new(key, &mut *self.0)
    }

    pub fn fetch_mut(&mut self, tx_hash: &Hash) -> HistoryOffers {
        self.index_mut()
            .get(tx_hash)
            .unwrap_or_else(|| HistoryOffers::new(vec![]) )
    }

    /// Store the new state for a wallet in the database.
    pub fn store(&mut self, tx_hash: &Hash, history: HistoryOffers) {
        self.index_mut().put(tx_hash, history);
    }

    /// Store the new state for a wallet in the database.
    pub fn update(&mut self, tx_hash: &Hash, data: &Vec<HistoryOffer>) {

        for closed_offer_info in data {
            let update = self.fetch_mut(closed_offer_info.tx_hash());
            update.history().push(HistoryOffer::new(tx_hash, closed_offer_info.amount()));
            self.store(tx_hash, update);
        }

        let base = self.fetch_mut(tx_hash);
        base.history().append(&mut data.clone());
        self.store(tx_hash, base);
    }
}
