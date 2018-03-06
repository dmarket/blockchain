use exonum::storage::{Fork, MapIndex, Snapshot};

use currency::SERVICE_NAME;
use currency::assets::{AssetId, AssetInfo};

/// Schema for accessing global asset information.
pub struct Schema<S>(pub S)
where
    S: AsRef<Snapshot>;

impl<S> Schema<S>
where
    S: AsRef<Snapshot>,
{
    pub fn index(self) -> MapIndex<S, AssetId, AssetInfo> {
        let key = SERVICE_NAME.to_string() + "_v1.assets";
        MapIndex::new(key, self.0)
    }

    /// Fetch asset info from the database.
    pub fn fetch(self, id: &AssetId) -> Option<AssetInfo> {
        self.index().get(id)
    }
}

impl<'a> Schema<&'a mut Fork> {
    pub fn index_mut(&mut self) -> MapIndex<&mut Fork, AssetId, AssetInfo> {
        let key = SERVICE_NAME.to_string() + "_v1.assets";
        MapIndex::new(key, self.0)
    }

    /// Store asset info in the database.
    pub fn store(&mut self, id: &AssetId, asset: AssetInfo) {
        self.index_mut().put(&*id, asset);
    }

    /// Remove asset info from the database.
    pub fn remove(&mut self, id: &AssetId) {
        self.index_mut().remove(id)
    }
}
