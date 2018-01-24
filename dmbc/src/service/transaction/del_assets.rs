extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use super::{SERVICE_ID, TX_DEL_ASSETS_ID};
use service::CurrencyService;
use service::asset::Asset;
use service::schema::asset::AssetSchema;
use service::schema::transaction_status::{TxStatus, TxStatusSchema};
use service::schema::wallet::WalletSchema;
use service::transaction::fee::{calculate_fees_for_del_assets, TxFees};

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
    pub fn get_fee(&self, fork: &mut Fork) -> TxFees {
        calculate_fees_for_del_assets(fork, self.assets())
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        // Invariant: for an asset id, the sum of amounts for all assets in
        // all wallets for this asset id is equal to the amonut stored in the
        // AssetInfo associated with this asset id.

        let (mut platform, mut creator) = WalletSchema::map(view, |mut schema| {
            let platform_key = CurrencyService::get_platfrom_wallet();
            (schema.wallet(&platform_key), schema.wallet(self.pub_key()))
        });

        let fee = self.get_fee(view);

        // Fail if not enough coins on creators balance
        if !creator.is_sufficient_funds(fee.amount()) {
            return TxStatus::Fail;
        }

        // Take coins for executing transaction
        creator.decrease(fee.amount());
        // put fee to platfrom wallet
        platform.increase(fee.amount());
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(self.pub_key(), creator.clone());
            schema.wallets().put(&platform.pub_key(), platform.clone());
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
            return TxStatus::Fail;
        }

        println!("Asset {:?}", self.assets());
        println!("Wallet after delete assets: {:?}", creator);

        // Remove wallet from db if it is empty completely, otherwise update db with changed wallet
        WalletSchema::map(view, |mut schema| match creator.is_empty() {
            true => schema.wallets().remove(self.pub_key()),
            false => schema.wallets().put(self.pub_key(), creator),
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
