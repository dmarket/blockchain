extern crate exonum;

use exonum::crypto::{PublicKey, HexValue};
use exonum::blockchain;
use exonum::storage::{Fork, MapIndex};

use service::wallet::{Wallet, Asset};
use super::SERVICE_ID;

pub struct AssetSchema<'a> {
    pub view: &'a mut Fork,
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

    pub fn create_asset_id(&mut self, external_asset_id: String, pub_key: &PublicKey) {
        let s = HexValue::to_hex(pub_key);
        let ful_s = s + &external_asset_id;
        println!("{:?}", ful_s);
    }

    pub fn add_asset(&mut self, asset_id: String, pub_key: &PublicKey) -> bool {

        match self.creator(&asset_id) {
            None => {
                println!("Add asset {:?} for wallet: {:?}", &asset_id, pub_key);
                self.assets().put(&asset_id, *pub_key);
                true
            },
            Some(pub_key) => true
        }
    }

    pub fn add_assets(&mut self, assets: Vec<Asset>, pub_key: &PublicKey) {
        for asset in assets {
            self.add_asset(asset.hash_id().to_string(), pub_key);
        }
    }
}
