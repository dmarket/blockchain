extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use service::asset::Asset;
use service::configuration::Configuration;

use super::{SERVICE_ID, TX_TRANSFER_ID};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

message! {
    struct TxTransfer {
        const TYPE = SERVICE_ID;
        const ID = TX_TRANSFER_ID;
        const SIZE = 88;

        field from:        &PublicKey  [00 => 32]
        field to:          &PublicKey  [32 => 64]
        field amount:      u64         [64 => 72]
        field assets:      Vec<Asset>  [72 => 80]
        field seed:        u64         [80 => 88]
    }
}

impl TxTransfer {
    pub fn get_fee(&self, fork: &Fork) -> u64 {
        Configuration::extract(fork).fees().transfer()
    }
}

impl Transaction for TxTransfer {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        (*self.from() != *self.to()) && self.verify_signature(self.from())
    }

    fn execute(&self, view: &mut Fork) {
        let mut sender = WalletSchema::map(view, |mut schema| schema.wallet(self.from()));
        let mut tx_status = TxStatus::Fail;

        let amount = self.amount();
        let fee = self.get_fee(view);

        let update_amount = amount == 0 && sender.balance() >= fee
            || amount > 0 && sender.balance() >= amount + fee;
        let update_assets = self.assets().is_empty()
            || !self.assets().is_empty() && sender.is_assets_in_wallet(&self.assets());
        if update_amount && update_assets {
            sender.decrease(amount + fee);
            sender.del_assets(&self.assets());
            WalletSchema::map(view, |mut schema| {
                let mut receiver = schema.wallet(self.to());
                receiver.increase(amount);
                receiver.add_assets(&self.assets());
                println!("Transfer between wallets: {:?} => {:?}", sender, receiver);
                schema.wallets().put(self.from(), sender);
                schema.wallets().put(self.to(), receiver);
            });
            tx_status = TxStatus::Success;
        }
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
