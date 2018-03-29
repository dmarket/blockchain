use std::collections::HashMap;

use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{Service, SERVICE_ID};
use currency::assets;
use currency::assets::AssetBundle;
use currency::wallet;
use currency::error::Error;
use currency::status;
use currency::transactions::components::FeesCalculator;
use currency::configuration::Configuration;

/// Transaction ID.
pub const DELETE_ASSETS_ID: u16 = 400;

message! {
    /// `delete_assets` transaction.
    struct DeleteAssets {
        const TYPE = SERVICE_ID;
        const ID = DELETE_ASSETS_ID;
        const SIZE = 48;

        field pub_key:     &PublicKey       [00 => 32]
        field assets:      Vec<AssetBundle> [32 => 40]
        field seed:        u64              [40 => 48]
    }
}

impl FeesCalculator for DeleteAssets {
    fn get_fees(&self, view: &mut Fork) -> Result<HashMap<PublicKey, u64>, Error> {
        let mut fees_map = HashMap::<PublicKey, u64>::new();
        let tx_fee = Configuration::extract(view).fees().delete_assets();
        fees_map.insert(*self.pub_key(), tx_fee);
        Ok(fees_map)
    }
}

impl DeleteAssets {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);

        let genesis_fee = Configuration::extract(view).fees().delete_assets();

        let genesis_pub = Service::genesis_wallet();
        let creator_pub = self.pub_key();

        let mut genesis = wallet::Schema(&*view).fetch(&genesis_pub);
        let mut creator = wallet::Schema(&*view).fetch(&creator_pub);

        wallet::move_coins(&mut creator, &mut genesis, genesis_fee)?;

        wallet::Schema(&mut*view).store(&genesis_pub, genesis);
        wallet::Schema(&mut*view).store(&creator_pub, creator.clone());

        let mut infos = HashMap::new();

        for asset in self.assets() {
            let info = match assets::Schema(&*view).fetch(&asset.id()) {
                Some(info) => info,
                None => return Err(Error::AssetNotFound),
            };
            if info.creator() != creator_pub {
                return Err(Error::InvalidTransaction);
            }
            let mut entry = infos.remove(&asset.id()).unwrap_or(info);
            let entry = entry.decrease(asset.amount())?;
            infos.insert(asset.id(), entry);
        }

        creator.remove_assets(self.assets())?;

        wallet::Schema(&mut*view).store(creator_pub, creator);

        for (id, info) in infos {
            assets::Schema(&mut*view).store(&id, info);
        }

        Ok(())
    }
}

impl Transaction for DeleteAssets {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return true;
        }

        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
