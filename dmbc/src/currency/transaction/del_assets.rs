extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use super::SERVICE_ID;
use currency::CurrencyService;
use currency::asset::Asset;
use currency::schema::transaction_status::{TxStatus, TxStatusSchema};
use currency::transaction::fee::TxFees;

use currency::schema::asset::AssetSchema;
use currency::schema::wallet::WalletSchema;

pub const TX_DEL_ASSETS_ID: u16 = 400;

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
        TxFees::for_del_assets(fork, self.assets())
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        // Invariant: for an asset id, the sum of amounts for all assets in
        // all wallets for this asset id is equal to the amonut stored in the
        // AssetInfo associated with this asset id.

        let mut platform =
            WalletSchema::get_wallet(view, &CurrencyService::genesis_wallet_pub_key());
        let mut creator = WalletSchema::get_wallet(view, self.pub_key());

        let fee = self.get_fee(view);

        // Pay fee for tx execution
        if WalletSchema::transfer_coins(view, &mut creator, &mut platform, fee.amount()).is_err() {
            return TxStatus::Fail;
        }

        // Check if asset exists, Fail if not.
        // If sender (pub_key) is not a creator of asset, then Fail.
        // If amount of assets to delete is bigger than amount of assets are stored, then Fail.
        for asset in self.assets() {
            match AssetSchema::get_asset_info(view, &asset.id()) {
                Some(ref info) => {
                    if info.creator() != self.pub_key() || asset.amount() > info.amount() {
                        return TxStatus::Fail;
                    }
                }
                None => return TxStatus::Fail,
            }
        }

        // if there are no assets to delete, Fail
        if WalletSchema::delete_assets(view, &mut creator, &self.assets()).is_err() {
            return TxStatus::Fail;
        }

        println!("Asset {:?}", self.assets());
        println!("Wallet after delete assets: {:?}", creator);

        if AssetSchema::remove(view, &self.assets()).is_err() {
            return TxStatus::Fail;
        }
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
