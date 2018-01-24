extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::CurrencyService;
use service::asset::Asset;
use service::transaction::fee::{calculate_fees_for_transfer, TxFees};

use super::{SERVICE_ID, TX_TRANSFER_ID};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

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
        let (mut platform, mut sender, mut receiver) = WalletSchema::map(view, |mut schema| {
            let platform_key = CurrencyService::get_platfrom_wallet();
            (
                schema.wallet(&platform_key),
                schema.wallet(self.from()),
                schema.wallet(self.to()),
            )
        });

        let fee = self.get_fee(view);

        // Fail if not enough coins on senders balance
        if !sender.is_sufficient_funds(fee.transaction_fee()) {
            return TxStatus::Fail;
        }

        // Take coins for executing transaction
        sender.decrease(fee.transaction_fee());
        // put fee to platfrom wallet
        platform.increase(fee.transaction_fee());
        // store data
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(self.from(), sender.clone());
            schema.wallets().put(&platform.pub_key(), platform.clone());
        });

        // initial point for db rollback, in case if transaction has failed
        view.checkpoint();

        let send_assets = !self.assets().is_empty();
        if send_assets {
            if sender.is_assets_in_wallet(&self.assets()) {
                // Check if sender has enough coins to pay fee to creators of assets
                if !sender.is_sufficient_funds(fee.assets_fees_total()) {
                    view.rollback();
                    return TxStatus::Fail;
                }

                sender.del_assets(&self.assets());
                receiver.add_assets(&self.assets());

                // send fees to creators of assets
                for (mut creator, fee) in fee.assets_fees() {
                    println!("Creator {:?} will receive {}", creator.pub_key(), fee);
                    sender.decrease(fee);
                    creator.increase(fee);
                    WalletSchema::map(view, |mut schema| {
                        schema.wallets().put(creator.pub_key(), creator.clone());
                        schema.wallets().put(sender.pub_key(), sender.clone());
                    });
                }
            } else {
                view.rollback();
                return TxStatus::Fail;
            }
        }

        // check if sender wants to send coins and has enough coins to send, otherwise - Fail.
        let coins_to_send = self.amount();
        let send_coins = coins_to_send > 0;
        if send_coins {
            if sender.is_sufficient_funds(coins_to_send) {
                sender.decrease(coins_to_send);
                receiver.increase(coins_to_send);
            } else {
                view.rollback();
                return TxStatus::Fail;
            }
        }

        if send_coins || send_assets {
            WalletSchema::map(view, |mut schema| {
                schema.wallets().put(self.from(), sender);
                schema.wallets().put(self.to(), receiver);
            });
            return TxStatus::Success;
        }

        view.rollback();
        TxStatus::Fail
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

        keys_ok && signature_ok && data_info_ok
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
