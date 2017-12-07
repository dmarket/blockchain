extern crate exonum;
extern crate uuid;

use std::collections::HashMap;
use exonum::crypto::{PublicKey, HexValue};
use exonum::storage::{Fork, MapIndex};
use self::uuid::Uuid;

use service::wallet::{Asset, AssetInfo};
use service::SERVICE_NAME;
use service::assetid::AssetID;

pub struct AssetSchema<'a>(&'a mut Fork);

pub fn generate_asset_id(external_asset_id: &AssetID, pub_key: &PublicKey) -> AssetID {
    let s = HexValue::to_hex(pub_key);
    let ful_s = s + &external_asset_id.to_string();

    let uuid = Uuid::new_v5(&uuid::NAMESPACE_DNS, &ful_s);
    match AssetID::from_bytes(uuid.as_bytes()) {
        Ok(asset_id) => asset_id,
        Err(..) => AssetID::new(0, 0)
    }
}

pub fn get_new_assets_id(assets: Vec<Asset>, pub_key: &PublicKey) -> HashMap<String, Asset> {
    let mut map_asset_id: HashMap<String, Asset> = HashMap::new();
    for asset in assets {
        let asset_id = generate_asset_id(&asset.hash_id(), pub_key);
        let new_asset = Asset::new(asset_id, asset.amount());
        map_asset_id.insert(asset.hash_id().to_string(), new_asset);
    }

    map_asset_id
}

pub fn external_internal(assets: Vec<Asset>, pub_key: &PublicKey) -> HashMap<String, String> {
    let mut old_id_new_id: HashMap<String, String> = HashMap::new();
    for (key, asset) in get_new_assets_id(assets, pub_key) {
        old_id_new_id.insert(key, asset.hash_id().to_string());
    }

    old_id_new_id
}

impl<'a> AssetSchema<'a> {
    pub fn assets(&mut self) -> MapIndex<&mut Fork, String, AssetInfo> {
        let key = SERVICE_NAME.to_string().replace("/", "_") + ".assets";
        MapIndex::new(key, self.0)
    }

    pub fn info(&mut self, asset_id: &AssetID) -> Option<AssetInfo> {
        self.assets().get(&asset_id.to_string())
    }

    pub fn add_asset(&mut self, asset_id: &AssetID, creator: &PublicKey, amount: u32) -> bool {
        match self.info(&asset_id) {
            None => {
                let info = AssetInfo::new(creator, amount);
                self.assets().put(&asset_id.to_string(), info);
                println!("Add asset {:?} for wallet: {:?}", asset_id, creator);
                true
            }
            Some(_) => true,
        }
    }

    pub fn add_assets(
        &mut self,
        assets: Vec<Asset>,
        pub_key: &PublicKey,
    ) -> HashMap<String, Asset> {
        let mut map_asset_id: HashMap<String, Asset> = HashMap::new();
        for asset in assets {
            let asset_id = generate_asset_id(&asset.hash_id(), pub_key);
            let new_asset = Asset::new(asset_id, asset.amount());
            self.add_asset(&new_asset.hash_id(), pub_key, asset.amount());
            map_asset_id.insert(asset.hash_id().to_string(), new_asset);
        }

        map_asset_id
    }

    pub fn del_assets(&mut self, deleted: &[Asset]) {
        let mut infos = self.assets();
        for asset in deleted {
            let info = match infos.get(&asset.hash_id().to_string()) {
                Some(info) => info,
                _ => continue,
            };
            let amount = info.amount() - asset.amount();
            let info = AssetInfo::new(info.creator(), amount);
            match info.amount() {
                0 => infos.remove(&asset.hash_id().to_string()),
                _ => infos.put(&asset.hash_id().to_string(), info),
            }
        }
    }

    pub fn map<F, T>(view: &'a mut Fork, f: F) -> T
    where
        F: FnOnce(Self) -> T + 'a,
        T: 'a,
    {
        f(AssetSchema(view))
    }
}
