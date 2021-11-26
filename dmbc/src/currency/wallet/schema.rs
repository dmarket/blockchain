use exonum::crypto::PublicKey;
use exonum::storage::{Fork, MapIndex, Snapshot, StorageKey};

use currency::assets::{AssetId, AssetBundle};
use currency::wallet::Wallet;
use currency::SERVICE_NAME;
use currency::error::Error;

/// The schema for accessing wallets data.
pub struct Schema<S>(pub S)
where
    S: AsRef<dyn Snapshot>;

impl<S> Schema<S>
where
    S: AsRef<dyn Snapshot>,
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
        if wallet.balance() == 0 {
            self.remove(pub_key);
        } else {
            self.index_mut().put(pub_key, wallet);
        }
    }

    pub fn store_asset(&mut self, pub_key: &PublicKey, asset_id: &AssetId, value: AssetBundle) {
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
    pub_key.write(&mut key[..pub_key.size()]);
    asset_id.write(&mut key[(pub_key.size() + 1)..]);
    key
}

pub fn move_assets(
    fork: &mut Fork,
    from: &PublicKey,
    to: &PublicKey,
    move_specs: &[AssetBundle],
) -> Result<(), Error> {
    fork.checkpoint();

    for spec in move_specs {
        let id = spec.id();

        let from_asset = Schema(&mut *fork).fetch_asset(from, &id);
        let to_asset = Schema(&mut *fork).fetch_asset(to, &id);

        let (from_asset, to_asset) = match (from_asset, to_asset) {
            (_, _) if spec.amount() == 0 => {
                continue;
            }
            (Some(from_asset), _) if from_asset.amount() < spec.amount() => {
                fork.rollback();
                return Err(Error::InsufficientAssets);
            }
            (None, _) => {
                fork.rollback();
                return Err(Error::InsufficientAssets)
            }
            (Some(from_asset), Some(to_asset)) => (
                AssetBundle::new(id.clone(), from_asset.amount() - spec.amount()),
                AssetBundle::new(id.clone(), to_asset.amount() + spec.amount()),
            ),
            (Some(from_asset), None) => (
                AssetBundle::new(id.clone(), from_asset.amount() - spec.amount()),
                AssetBundle::new(id.clone(), spec.amount()),
            ),
        };

        if from_asset.amount() > 0 {
            Schema(&mut *fork).store_asset(from, &id, from_asset);
        } else {
            Schema(&mut *fork).remove_asset(from, &id);
        }
        Schema(&mut *fork).store_asset(to, &id, to_asset);
    }

    fork.commit();
    Ok(())
}

