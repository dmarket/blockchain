extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use exonum::encoding::serialize::FromHex;
use serde_json::Value;

use super::{SERVICE_ID, TX_DEL_ASSETS_ID};
use service;
use service::asset::Asset;
use service::schema::asset::AssetSchema;
use service::schema::transaction_status::{TxStatus, TxStatusSchema};
use service::schema::wallet::WalletSchema;
use service::configuration::Configuration;

message! {
    struct TxDelAsset {
        const TYPE = SERVICE_ID;
        const ID = TX_DEL_ASSETS_ID;
        const SIZE = 48;

        field pub_key:     &PublicKey  [00 => 32]
        field assets:      Vec<Asset>  [32 => 40]
        field seed:        u64         [40 => 48]
    }
}

impl TxDelAsset {
    pub fn get_fee(&self, fork: &Fork) -> u64 {
        Configuration::extract(fork).fees().del_asset()
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        // Invariant: for an asset id, the sum of amounts for all assets in
        // all wallets for this asset id is equal to the amonut stored in the
        // AssetInfo associated with this asset id.

        let platform_key = PublicKey::from_hex(service::PLATFORM_WALLET).unwrap();
        let mut platform = WalletSchema::map(view, |mut schema| schema.wallet(&platform_key));
        let mut creator = WalletSchema::map(view, |mut schema| schema.wallet(self.pub_key()));
        let fee = self.get_fee(view);

        // Fail if not enough coins on creators balance
        if creator.balance() < fee {
            return TxStatus::Fail
        }

        // Take coins for executing transaction
        creator.decrease(fee);
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(self.pub_key(), creator.clone())
        });
        // put fee to platfrom wallet
        platform.increase(fee);
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(&platform_key, platform.clone())
        });

        // Check if asset exists, Fail if not. 
        // If sender (pub_key) is not a creator of asset, then Fail.
        // If amount of assets to delete is bigger than amount of assets are stored, then Fail.
        for asset in self.assets() {
            match AssetSchema::map(view, |mut assets| assets.info(&asset.id())) {
                Some(ref info) => {
                    if info.creator() != self.pub_key() || asset.amount() > info.amount() {
                        return TxStatus::Fail;
                    }
                }
                None => return TxStatus::Fail,
            }
        }

        // if there are no assets to delete, Fail
        if !creator.del_assets(&self.assets()) {
            return TxStatus::Fail
        }

        println!("Asset {:?}", self.assets());
        println!("Wallet after delete assets: {:?}", creator);

        // Remove wallet from db if it is empty, otherwise update db with changed wallet
        WalletSchema::map(view, |mut schema| {
            match creator.is_empty() {
                true => schema.wallets().remove(self.pub_key()),
                false => schema.wallets().put(self.pub_key(), creator)
            }
        });

        AssetSchema::map(view, |mut schema| schema.del_assets(&self.assets()));
        TxStatus::Success
    }
}

impl Transaction for TxDelAsset {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        self.verify_signature(self.pub_key())
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
        })
    }
}
