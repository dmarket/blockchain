use exonum::crypto::PublicKey;
use exonum::storage::{Fork, MapIndex, map_index::MapIndexIter, Snapshot, StorageKey};

use currency::assets::{AssetId, AssetBundle};
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

    pub fn index_assets(self) -> MapIndex<S, Vec<u8>, AssetBundle> {
        let key = SERVICE_NAME.to_string() + ".wallets";
        MapIndex::new(key, self.0)
    }

    pub fn fetch(self, pub_key: &PublicKey) -> Wallet {
        self.index()
            .get(pub_key)
            .unwrap_or_else(|| Wallet::new_empty())
    }

    pub fn fetch_asset(
        self,
        pub_key: &PublicKey,
        asset_id: &AssetId,
    ) -> Option<AssetBundle> {
        let key = wallet_asset_key(pub_key, asset_id);
        self.index_assets()
            .get(&key)
    }

    /*
    pub fn fetch_assets(
        self,
        pub_key: &PublicKey,
    ) -> MapIndexIter<Vec<u8>, AssetBundle> {
        let mut key = vec![0; pub_key.size() + 1];
        pub_key.write(&mut key[..]);
        self.index_assets()
            .iter_from(&key)
    }
    */
}

impl<'a> Schema<&'a mut Fork> {
    /// Internal `MapIndex` with mutable access.
    pub fn index_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        let key = SERVICE_NAME.to_string() + ".wallets";
        MapIndex::new(key, &mut *self.0)
    }

    pub fn index_assets_mut(&mut self) -> MapIndex<&mut Fork, Vec<u8>, AssetBundle> {
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

    pub fn store_assets(&mut self, pub_key: &PublicKey, asset_id: &AssetId, value: AssetBundle) {
        let key = wallet_asset_key(pub_key, asset_id);
        self.index_assets_mut()
            .put(&key, value);
    }

    /// Remove wallet state from the database.
    pub fn remove(&mut self, pub_key: &PublicKey) {
        self.index_mut().remove(pub_key);
    }

    pub fn remove_asset(&mut self, pub_key: &PublicKey, asset_id: &AssetId) {
        let key = wallet_asset_key(pub_key, asset_id);
        self.index_assets_mut().remove(&key);
    }
}

fn wallet_asset_key(pub_key: &PublicKey, asset_id: &AssetId) -> Vec<u8> {
    let mut key = vec![0; pub_key.size() + asset_id.size() + 1];
    pub_key.write(&mut key[..]);
    asset_id.write(&mut key[(pub_key.size() + 1)..]);
    key
}

