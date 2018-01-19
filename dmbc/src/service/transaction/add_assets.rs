extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use std::collections::HashMap;

use service::CurrencyService;
use service::asset::{Asset, Fees, MetaAsset};
use service::configuration::Configuration;

use super::{SERVICE_ID, TX_ADD_ASSETS_ID};
use super::schema::asset::AssetSchema;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

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

pub struct AddAssetFee {
    transaction_fee: u64,
    assets_fees: u64,
}

impl TxAddAsset {
    pub fn get_fee(&self, fork: &Fork) -> AddAssetFee {
        let configuration = Configuration::extract(fork);
        let count = self.meta_assets()
            .iter()
            .fold(0, |acc, asset| acc + asset.amount() as u64);

        let tx_fee = configuration.fees().add_asset();
        let assets_fees = configuration.fees().per_add_asset() * count;
        AddAssetFee::new(tx_fee, assets_fees)
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
        let platform_key = CurrencyService::get_platfrom_wallet();
        let mut platform = WalletSchema::map(view, |mut schema| schema.wallet(&platform_key));
        let mut creator = WalletSchema::map(view, |mut schema| schema.wallet(self.pub_key()));
        let fee = self.get_fee(view);

        // Check if creator has enough coins to execute transaction, fail otherwise
        if creator.balance() < fee.transaction_fee() {
            return TxStatus::Fail
        }

        // extract fee
        // remove fee from creator and update creator wallet balance
        creator.decrease(fee.transaction_fee());
        // put fee to platfrom wallet
        platform.increase(fee.transaction_fee());
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(self.pub_key(), creator.clone());
            schema.wallets().put(&platform.pub_key(), platform.clone());
        });

        // Now, check if creator has enough coins for asset fees
        if creator.balance() < fee.assets_fees() {
            return TxStatus::Fail
        }
        // initial point for db rollback, in case if transaction has failed
        view.checkpoint();

        // extract fee
        // remove fee from creator and update creator wallet balance
        creator.decrease(fee.assets_fees());
        // put fee to platfrom wallet
        platform.increase(fee.assets_fees());
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(self.pub_key(), creator.clone());
            schema.wallets().put(&platform.pub_key(), platform.clone());
        });


        // `Fail` status can occur due two reasons:
        // 1. `schema.add_assets` will fail if asset id generation has collision
        // 2. Asset exists and AssetId is the same but new fees are different for existing asset
        // rollback changes if adding procedure has failed

        // store new assets in asset schema
        let (assets, fees_list, receivers) = self.get_assets_fees_receivers();
        let is_assets_added = AssetSchema::map(view, |mut schema| {
            schema.add_assets(&assets, &fees_list, self.pub_key())
        });

        if is_assets_added {

            // send assets to receivers
            for (receiver_key, asset) in receivers.iter().zip(assets) {
                let mut receiver =
                    WalletSchema::map(view, |mut schema| schema.wallet(receiver_key));
                receiver.add_assets(&[asset]);
                WalletSchema::map(view, |mut schema| {
                    schema.wallets().put(receiver_key, receiver)
                });
            }
        } else {
            println!("Unable to add assets {:?}, rolling back transaction...", self.meta_assets());
            view.rollback();
            return TxStatus::Fail
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

impl AddAssetFee {
    pub fn new(tx_fee: u64, assets_fees: u64) -> Self {
        AddAssetFee {
            transaction_fee: tx_fee,
            assets_fees: assets_fees,
        }
    }

    pub fn amount(&self) -> u64 {
        self.transaction_fee + self.assets_fees
    }

    pub fn transaction_fee(&self) -> u64 {
        self.transaction_fee
    }

    pub fn assets_fees(&self) -> u64 {
        self.assets_fees
    }
}