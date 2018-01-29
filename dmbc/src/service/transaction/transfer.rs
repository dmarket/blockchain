extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::CurrencyService;
use service::asset::Asset;
use service::transaction::fee::{calculate_fees_for_transfer, TxFees};

use service::schema::wallet::WalletSchema;

use super::{SERVICE_ID, TX_TRANSFER_ID};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};

message! {
    struct TxTransfer {
        const TYPE = SERVICE_ID;
        const ID = TX_TRANSFER_ID;
        const SIZE = 96;

        field from:        &PublicKey  [00 => 32]
        field to:          &PublicKey  [32 => 64]
        field amount:      u64         [64 => 72]
        field assets:      Vec<Asset>  [72 => 80]
        field seed:        u64         [80 => 88]
        field data_info:   &str        [88 => 96]
    }
}

impl TxTransfer {
    pub fn get_fee(&self, fork: &mut Fork) -> TxFees {
        calculate_fees_for_transfer(fork, self.assets())
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        let mut platform = WalletSchema::get_wallet(view, &CurrencyService::get_platform_pub_key());
        let mut sender = WalletSchema::get_wallet(view, self.from());
        let mut receiver = WalletSchema::get_wallet(view, self.to());

        let fee = self.get_fee(view);

        // Pay fee for tx execution
        if WalletSchema::transfer_coins(view, &mut sender, &mut platform, fee.transaction_fee())
            .is_err()
        {
            return TxStatus::Fail;
        }

        // initial point for db rollback, in case if transaction has failed
        view.checkpoint();

        if !self.assets().is_empty() {
            if WalletSchema::transfer_assets(view, &mut sender, &mut receiver, &self.assets())
                .is_err()
            {
                view.rollback();
                return TxStatus::Fail;
            }

            // send fees to creators of assets
            for (mut creator, fee) in fee.assets_fees() {
                println!("Creator {:?} will receive {}", creator.pub_key(), fee);
                if WalletSchema::transfer_coins(view, &mut sender, &mut creator, fee).is_err() {
                    view.rollback();
                    return TxStatus::Fail;
                }
            }
        }

        // check if sender wants to send coins and has enough coins to send, otherwise - Fail.
        let coins_to_send = self.amount();
        if coins_to_send > 0 {
            if WalletSchema::transfer_coins(view, &mut sender, &mut receiver, coins_to_send)
                .is_err()
            {
                view.rollback();
                return TxStatus::Fail;
            }
        }

        TxStatus::Success
    }
}

impl Transaction for TxTransfer {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        let data_info_ok = self.data_info().len() <= 1024;
        let signature_ok = self.verify_signature(self.from());
        let keys_ok = *self.from() != *self.to();
        let payload_ok = self.amount() > 0 || !self.assets().is_empty();

        keys_ok && signature_ok && data_info_ok && payload_ok
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
