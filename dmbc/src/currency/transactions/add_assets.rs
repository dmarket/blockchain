use std::collections::HashMap;
use std::collections::hash_map::Entry;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;
use prometheus::{Counter, Histogram};

use currency::SERVICE_ID;
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
        let genesis_fees = Configuration::extract(view).fees();
        let fees = ThirdPartyFees::new_add_assets(&view, self.meta_assets())?;   

        let mut fees_table = HashMap::new();
        if genesis_fees.recipient() != self.pub_key() {
            fees_table.insert(*self.pub_key(), genesis_fees.add_assets());
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

        let genesis_fees = Configuration::extract(&*view).fees();
        let tx_fee = genesis_fees.add_assets();

        let genesis_pub = genesis_fees.recipient();
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

        for meta in self.meta_assets() {
            let id = AssetId::from_data(meta.data(), key);

            let wallet = wallets.entry(*meta.receiver())
                .or_insert_with(|| wallet::Schema(&*view).fetch(meta.receiver()));
            wallet.add_assets(Some(meta.to_bundle(id)));

            match infos.entry(id) {
                Entry::Occupied(entry) => {
                    let info = entry.into_mut();
                    *info = info.clone().merge(meta.to_info(key, &info.origin()))?;
                }
                Entry::Vacant(entry) => {
                    let origin = self.hash();
                    let new_info = meta.to_info(key, &origin);
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

lazy_static! {
    static ref VERIFY_COUNT: Counter = register_counter!(
        "dmbc_transaction_add_assets_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_add_assets_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: Counter = register_counter!(
        "dmbc_transaction_add_assets_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_add_assets_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: Counter = register_counter!(
        "dmbc_transaction_add_assets_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
    static ref EXECUTE_DURATION: Histogram = register_histogram!(
        "dmbc_transaction_add_assets_execute_duration_seconds",
        "Duration of transaction execution."
    ).unwrap();
}

impl Transaction for AddAssets {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

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

        VERIFY_SUCCESS_COUNT.inc();

        true
    }

    fn execute(&self, view: &mut Fork) {
        EXECUTE_COUNT.inc();
        let timer = EXECUTE_DURATION.start_timer();

        let result = self.process(view);

        if let &Ok(_) = &result {
            EXECUTE_SUCCESS_COUNT.inc();
        }

        status::Schema(view).store(self.hash(), result);

        timer.observe_duration();
        EXECUTE_FINISH_COUNT.inc();
    }

    fn info(&self) -> serde_json::Value {
        json!(self)
    }
}
