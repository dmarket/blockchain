extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::asset::{Asset, MetaAsset};
use service::transaction::{PER_ADD_ASSET_FEE, TX_ADD_ASSET_FEE};

use super::{SERVICE_ID, TX_ADD_ASSETS_ID};
use super::schema::asset::{AssetSchema, external_internal};
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
    pub fn get_fee(&self) -> u64 {
        let count = self.meta_assets().iter().fold(0, |acc, asset| {
            acc + asset.amount() as u64
        });

        TX_ADD_ASSET_FEE + PER_ADD_ASSET_FEE * count
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
        let creator = WalletSchema::map(view, |mut schema| schema.wallet(self.pub_key()));
        if let Some(mut creator) = creator {
            if creator.balance() >= self.get_fee() {
                let map_assets = AssetSchema::map(view, |mut schema| {
                    schema.add_assets(self.meta_assets(), self.pub_key())
                });
                creator.decrease(self.get_fee());
                println!("Convert {:?}", map_assets);
                let new_assets: Vec<Asset> = map_assets
                    .iter()
                    .map(|(_, asset)| Asset::new(asset.id(), asset.amount()))
                    .collect();
                creator.add_assets(&new_assets);
                tx_status = TxStatus::Success;
            }
            println!("Wallet after mining asset: {:?}", creator);
            WalletSchema::map(view, |mut schema| {
                schema.wallets().put(self.pub_key(), creator)
            });
        }
        TxStatusSchema::map(
            view,
            |mut schema| schema.set_status(&self.hash(), tx_status),
        );
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "external_internal": external_internal(self.meta_assets(), self.pub_key()),
        })
    }
}
