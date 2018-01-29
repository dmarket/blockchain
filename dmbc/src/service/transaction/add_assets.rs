extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use std::collections::HashMap;

use service::CurrencyService;
use service::asset::{Asset, Fees, MetaAsset};
use service::transaction::fee::{calculate_fees_for_add_assets, TxFees};

use service::schema::asset::AssetSchema;
use service::schema::wallet::WalletSchema;

use super::SERVICE_ID;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};

pub const TX_ADD_ASSETS_ID: u16 = 300;

message! {
    struct TxAddAsset {
        const TYPE = SERVICE_ID;
        const ID = TX_ADD_ASSETS_ID;
        const SIZE = 48;

        field pub_key:       &PublicKey       [00 => 32]
        field meta_assets:   Vec<MetaAsset>   [32 => 40]
        field seed:          u64              [40 => 48]
    }
}

impl TxAddAsset {
    pub fn get_fee(&self, fork: &mut Fork) -> TxFees {
        calculate_fees_for_add_assets(fork, self.meta_assets(), self.pub_key())
    }

    fn get_assets_fees_receivers(&self) -> (Vec<Asset>, Vec<Fees>, Vec<PublicKey>) {
        let mut assets = Vec::new();
        let mut fees_list = Vec::new();
        let mut receivers = Vec::new();

        for meta_asset in self.meta_assets() {
            let asset = Asset::from_meta_asset(&meta_asset.clone(), self.pub_key());
            assets.push(asset);
            fees_list.push(meta_asset.fees());
            receivers.push(meta_asset.receiver().clone());
        }

        (assets, fees_list, receivers)
    }

    fn from_meta_to_asset_map(
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

    fn external_internal(
        meta_assets: Vec<MetaAsset>,
        pub_key: &PublicKey,
    ) -> HashMap<String, String> {
        let mut meta_asset_to_asset: HashMap<String, String> = HashMap::new();

        for (key, asset) in Self::from_meta_to_asset_map(meta_assets, pub_key) {
            meta_asset_to_asset.insert(key, asset.id().to_string());
        }

        meta_asset_to_asset
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        let mut platform = WalletSchema::get_wallet(view, &CurrencyService::get_platform_pub_key());
        let mut creator = WalletSchema::get_wallet(view, self.pub_key());

        let fee = self.get_fee(view);

        // Pay fee for tx execution
        if WalletSchema::transfer_coins(view, &mut creator, &mut platform, fee.transaction_fee())
            .is_err()
        {
            return TxStatus::Fail;
        }

        // initial point for db rollback, in case if transaction has failed
        view.checkpoint();

        // pay fee for assets
        if WalletSchema::transfer_coins(view, &mut creator, &mut platform, fee.assets_fees_total())
            .is_err()
        {
            view.rollback();
            return TxStatus::Fail;
        }

        // `Fail` status can occur due two reasons:
        // 1. `schema.add_assets` will fail if asset id generation has collision
        // 2. Asset exists and AssetId is the same but new fees are different for existing asset
        // rollback changes if adding procedure has failed

        // store new assets in asset schema
        let (assets, fees_list, receivers) = self.get_assets_fees_receivers();
        if AssetSchema::store(view, self.pub_key(), &assets, &fees_list).is_err() {
            view.rollback();
            return TxStatus::Fail;
        }

        // send assets to receivers
        for (receiver_key, asset) in receivers.iter().zip(assets) {
            let mut receiver = WalletSchema::get_wallet(view, receiver_key);
            if WalletSchema::add_assets(view, &mut receiver, &[asset]).is_err() {
                view.rollback();
                return TxStatus::Fail;
            }
        }

        TxStatus::Success
    }
}

impl Transaction for TxAddAsset {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        if self.verify_signature(self.pub_key()) {
            for asset in self.meta_assets().iter() {
                if !asset.is_valid() {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    fn execute(&self, view: &mut Fork) {
        let tx_status = self.process(view);

        TxStatusSchema::map(view, |mut schema| {
            schema.set_status(&self.hash(), tx_status)
        });
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "external_internal": Self::external_internal(self.meta_assets(), self.pub_key()),
        })
    }
}
