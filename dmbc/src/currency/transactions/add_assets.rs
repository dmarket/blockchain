use std::collections::HashMap;
use std::collections::hash_map::Entry;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{Service, SERVICE_ID};
use currency::assets;
use currency::assets::{AssetId, AssetInfo, MetaAsset};
use currency::wallet;
use currency::status;
use currency::error::Error;
use currency::transactions::components::{ThirdPartyFees, FeesCalculator};
use currency::configuration::Configuration;

/// Transaction ID.
pub const ADD_ASSETS_ID: u16 = 300;

message!{
    /// `add_assets` transaction.
    struct AddAssets {
        const TYPE = SERVICE_ID;
        const ID = ADD_ASSETS_ID;
        const SIZE = 48;

        field pub_key:     &PublicKey     [00 => 32]
        field meta_assets: Vec<MetaAsset> [32 => 40]
        field seed:        u64            [40 => 48]
    }
}

impl FeesCalculator for AddAssets {
    fn calculate_fees(&self, view: &mut Fork) -> Result<HashMap<PublicKey, u64>, Error> {
        let genesis_fee = Configuration::extract(view).fees().add_assets();
        let fees = ThirdPartyFees::new_add_assets(&view, self.meta_assets())?;   

        let mut fees_table = HashMap::new();
        if Service::genesis_wallet() != *self.pub_key() {
            fees_table.insert(*self.pub_key(), genesis_fee);
        }

        for (pub_key, fee) in fees.0 {
            if pub_key != *self.pub_key() {
                *fees_table.entry(*self.pub_key()).or_insert(0) += fee;
            }
        }
        Ok(fees_table)
    }
}

impl AddAssets {

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);

        let tx_fee = Configuration::extract(view).fees().add_assets();

        let genesis_pub = Service::genesis_wallet();
        let creator_pub = self.pub_key();

        let mut genesis = wallet::Schema(&*view).fetch(&genesis_pub);
        let mut creator = wallet::Schema(&*view).fetch(&creator_pub);

        wallet::move_coins(&mut creator, &mut genesis, tx_fee)?;

        let fees = ThirdPartyFees::new_add_assets(&view, self.meta_assets())?;

        wallet::Schema(&mut *view).store(&genesis_pub, genesis);
        wallet::Schema(&mut *view).store(&creator_pub, creator);

        let mut wallets = fees.collect(view, &creator_pub)?;
        let mut infos: HashMap<AssetId, AssetInfo> = HashMap::new();

        let key = self.pub_key();
        let tx_hash = self.hash();

        for meta in self.meta_assets() {
            let id = AssetId::from_data(meta.data(), key);

            let wallet = wallets.entry(*meta.receiver())
                .or_insert_with(|| wallet::Schema(&*view).fetch(meta.receiver()));
            wallet.add_assets(Some(meta.to_bundle(id)));

            match infos.entry(id) {
                Entry::Occupied(entry) => {
                    let info = entry.into_mut();
                    *info = info.clone().merge(meta.to_info(key))?;
                }
                Entry::Vacant(entry) => {
                    let new_info = meta.to_info(key);
                    let info = match assets::Schema(&*view).fetch(&id) {
                        Some(info) => info.merge(new_info)?,
                        None => new_info,
                    };
                    entry.insert(info);
                }
            }
        }

        for (key, wallet) in wallets {
            wallet::Schema(&mut *view).store(&key, wallet);
        }

        for (id, info) in infos {
            assets::Schema(&mut *view).store(&id, info);
        }

        Ok(())
    }
}

impl Transaction for AddAssets {
    fn verify(&self) -> bool {
        for asset in self.meta_assets() {
            if !asset.verify() {
                return false;
            }
        }

        if cfg!(fuzzing) {
            return true;
        }

        if !self.verify_signature(&self.pub_key()) {
            return false;
        }

        true
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!(self)
    }
}
