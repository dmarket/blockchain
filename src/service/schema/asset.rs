use exonum::crypto::PublicKey;
use exonum::storage::{Fork, MapIndex};
use std::collections::HashMap;

use service::SERVICE_NAME;
use service::asset::{Asset, AssetID, AssetInfo, MetaAsset, Fees};

pub struct AssetSchema<'a>(&'a mut Fork);

pub fn from_meta_to_asset_map(
    meta_assets: Vec<MetaAsset>,
    pub_key: &PublicKey,
) -> HashMap<String, Asset> {
    let mut map_asset_id: HashMap<String, Asset> = HashMap::new();

    for meta_asset in meta_assets {
        let key = &meta_asset.data();
        let new_asset = Asset::from_meta_asset(&meta_asset, pub_key);
        map_asset_id.insert(key.to_string(), new_asset);
    }

    map_asset_id
}

pub fn external_internal(
    meta_assets: Vec<MetaAsset>,
    pub_key: &PublicKey,
) -> HashMap<String, String> {
    let mut meta_asset_to_asset: HashMap<String, String> = HashMap::new();

    for (key, asset) in from_meta_to_asset_map(meta_assets, pub_key) {
        meta_asset_to_asset.insert(key, asset.id().to_string());
    }

    meta_asset_to_asset
}

impl<'a> AssetSchema<'a> {
    pub fn assets(&mut self) -> MapIndex<&mut Fork, AssetID, AssetInfo> {
        let name = SERVICE_NAME.to_string().replace("/", "_") + ".assets";
        MapIndex::new(name, self.0)
    }

    pub fn info(&mut self, asset_id: &AssetID) -> Option<AssetInfo> {
        self.assets().get(&asset_id)
    }

    pub fn add_asset(&mut self, asset_id: &AssetID, creator: &PublicKey, amount: u32, fees: Fees) -> bool {
        match self.info(&asset_id) {
            None => {
                let info = AssetInfo::new(creator, amount, fees);
                self.assets().put(&asset_id, info);
                println!("Add asset {:?} for wallet: {:?}", asset_id, creator);
                true
            }
            Some(info) => {
                let info = AssetInfo::new(creator, info.amount() + amount, fees);
                self.assets().put(&asset_id, info);
                true
            }
        }
    }

    pub fn add_assets(
        &mut self,
        meta_assets: Vec<MetaAsset>,
        pub_key: &PublicKey,
    ) -> HashMap<String, Asset> {
        let mut map_asset_id: HashMap<String, Asset> = HashMap::new();
        for meta_asset in meta_assets {
            let new_asset = Asset::from_meta_asset(&meta_asset, pub_key);
            self.add_asset(&new_asset.id(), pub_key, new_asset.amount(), meta_asset.fees());
            map_asset_id.insert(new_asset.id().to_string(), new_asset);
        }

        map_asset_id
    }

    pub fn del_assets(&mut self, deleted: &[Asset]) {
        let mut infos = self.assets();
        for asset in deleted {
            let info = match infos.get(&asset.id()) {
                Some(info) => info,
                _ => continue,
            };
            let amount = info.amount() - asset.amount();
            let info = AssetInfo::new(info.creator(), amount, info.fees());
            match info.amount() {
                0 => infos.remove(&asset.id()),
                _ => infos.put(&asset.id(), info),
            }
        }
    }

    pub fn map<F, T>(view: &'a mut Fork, f: F) -> T
        where F: FnOnce(Self) -> T + 'a, T: 'a
    {
        f(AssetSchema(view))
    }
}
