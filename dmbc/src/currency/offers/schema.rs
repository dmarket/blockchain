use exonum::storage::{Fork, MapIndex, Snapshot};

use currency::assets::AssetId;
use currency::offers::OpenOffers;
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
    pub fn index(self) -> MapIndex<S, AssetId, OpenOffers> {
        let key = SERVICE_NAME.to_string() + ".open_offers";
        MapIndex::new(key, self.0)
    }

    /// Fetch state for the specified wallet from the snapshot.
    pub fn fetch(self, asset_id: &AssetId) -> OpenOffers {
        self.index()
            .get(asset_id)
            .unwrap_or_else(|| OpenOffers::new_open_offers() )
    }
}

impl<'a> Schema<&'a mut Fork> {
    /// Internal `MapIndex` with mutable access.
    pub fn index_mut(&mut self) -> MapIndex<&mut Fork, AssetId, OpenOffers> {
        let key = SERVICE_NAME.to_string() + ".open_offers";
        MapIndex::new(key, &mut *self.0)
    }

    /// Store the new state for a wallet in the database.
    pub fn store(&mut self, asset_id: &AssetId, open_offers: OpenOffers) {
        match (open_offers.bids().len(), open_offers.asks().len()) {
            (0, 0) => self.remove(asset_id),
            (_, _) => self.index_mut().put(asset_id, open_offers),
        };
    }

    /// Remove wallet state from the database.
    pub fn remove(&mut self, asset_id: &AssetId) {
        self.index_mut().remove(asset_id);
    }
}
