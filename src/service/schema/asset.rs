extern crate exonum;
extern crate uuid;

use std::collections::HashMap;
use exonum::crypto::{PublicKey, HexValue};
use exonum::blockchain;
use exonum::storage::{Fork, MapIndex};
use self::uuid::Uuid;

use service::wallet::Asset;
use super::SERVICE_ID;

pub struct AssetSchema<'a> {
    pub view: &'a mut Fork,
}

pub fn generate_asset_id(external_asset_id: &str, pub_key: &PublicKey) -> String {
    let s = HexValue::to_hex(pub_key);
    let ful_s = s + external_asset_id;
    let hash_id = Uuid::new_v5(&uuid::NAMESPACE_DNS, &ful_s).hyphenated().to_string();

    hash_id
}

pub fn get_new_assets_id(assets: &Vec<Asset>, pub_key: &PublicKey) -> HashMap<String, Asset> {
    let mut map_asset_id: HashMap<String, Asset> = HashMap::new();
    for asset in assets {
        let new_hash_id = generate_asset_id(asset.hash_id(), pub_key);
        let new_asset = Asset::new(&new_hash_id, asset.amount());
        map_asset_id.insert(asset.hash_id().to_string(), new_asset);
    }

    map_asset_id
}

pub fn external_internal(assets: &Vec<Asset>, pub_key: &PublicKey) -> HashMap<String, String> {
    let mut old_id_new_id: HashMap<String, String> = HashMap::new();
    for (key, asset) in get_new_assets_id(assets, pub_key).into_iter() {
        old_id_new_id.insert(key, asset.hash_id().to_string());
    }

    old_id_new_id
}

impl<'a> AssetSchema<'a> {
    pub fn assets(&mut self) -> MapIndex<&mut Fork, String, PublicKey> {
        let prefix = blockchain::gen_prefix(SERVICE_ID, 1, &());
        MapIndex::new(prefix, self.view)
    }

    // Utility method to quickly get a separate wallet from the storage
    pub fn creator(&mut self, asset_id: &String) -> Option<PublicKey> {
        self.assets().get(asset_id)
    }

    pub fn add_asset(&mut self, asset_id: &String, pub_key: &PublicKey) -> bool {
        match self.creator(asset_id) {
            None => {
                println!("Add asset {:?} for wallet: {:?}", asset_id, pub_key);
                self.assets().put(asset_id, *pub_key);
                true
            },
            Some(_) => true
        }
    }

    pub fn add_assets(&mut self, assets: &Vec<Asset>, pub_key: &PublicKey) -> HashMap<String, Asset> {
        let mut map_asset_id: HashMap<String, Asset> = HashMap::new();
        for asset in assets {
            let new_hash_id = generate_asset_id(asset.hash_id(), pub_key);
            let new_asset = Asset::new(&new_hash_id, asset.amount());
            self.add_asset(&new_hash_id, pub_key);
            map_asset_id.insert(asset.hash_id().to_string(), new_asset);
        }

        map_asset_id
    }
}
