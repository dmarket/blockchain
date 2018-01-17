extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use exonum::encoding::serialize::FromHex;
use serde_json::Value;
use std::collections::HashMap;

use service;
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

impl TxAddAsset {
    pub fn get_fee(&self, fork: &Fork) -> u64 {
        let configuration = Configuration::extract(fork);
        let count = self.meta_assets()
            .iter()
            .fold(0, |acc, asset| acc + asset.amount() as u64);

        configuration.fees().add_asset() + configuration.fees().per_add_asset() * count
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
        let mut tx_status = TxStatus::Fail;
        let mut creator = WalletSchema::map(view, |mut schema| schema.wallet(self.pub_key()));
        let platform_key = PublicKey::from_hex(service::PLATFORM_WALLET).unwrap();
        let mut platform = WalletSchema::map(view, |mut schema| schema.wallet(&platform_key));

        let fee = self.get_fee(view);

        if creator.balance() >= fee {
            // remove fee from creator and update creator wallet balance
            creator.decrease(fee);
            WalletSchema::map(view, |mut schema| {
                schema.wallets().put(self.pub_key(), creator.clone())
            });
            // put fee to platfrom wallet
            platform.increase(fee);
            WalletSchema::map(view, |mut schema| {
                schema.wallets().put(&platform_key, platform.clone())
            });

            // initial point for db rollback, in case if transaction has failed
            view.checkpoint();

            // store new assets in asset schema
            let (assets, fees_list, receivers) = self.get_assets_fees_receivers();
            let is_assets_added = AssetSchema::map(view, |mut schema| {
                schema.add_assets(&assets, &fees_list, self.pub_key())
            });

            if is_assets_added {
                tx_status = TxStatus::Success;

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
                tx_status = TxStatus::Fail;
            }

            // `Fail` status can occur due two reasons:
            // 1. `schema.add_assets` will fail if asset id generation has collision
            // 2. any from receivers wallet does not exist
            // rollback changes if adding procedure has failed
            if tx_status == TxStatus::Fail {
                println!("Unable to add assets {:?}", self.meta_assets());
                view.rollback();
            }

            println!("Wallet after mining asset: {:?}", creator);
        } else {
            // if creator.balance() >= fee
            println!(
                "Insuficient funds at {:?} wallet, required {}",
                creator, fee
            );
        }

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
