extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use currency::transaction::TRANSACTION_FEE;

use super::SERVICE_ID;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;
use super::wallet::Wallet;

pub const TX_CREATE_WALLET_ID: u16 = 100;
pub const INIT_BALANCE: u64 = 100_000_000; // 1 DMC = 100_000_000 dimosh

message! {
    struct TxCreateWallet {
        const TYPE = SERVICE_ID;
        const ID = TX_CREATE_WALLET_ID;
        const SIZE = 32;

        field pub_key:     &PublicKey  [00 => 32]
    }
}

impl TxCreateWallet {
    pub fn get_fee(&self) -> u64 {
        TRANSACTION_FEE
    }
}

impl Transaction for TxCreateWallet {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let tx_status = WalletSchema::map(view, |mut schema| {
            let wallet = Wallet::new(self.pub_key(), INIT_BALANCE, vec![]);
            println!("Create the wallet: {:?}", wallet);
            schema.wallets().put(self.pub_key(), wallet);
            TxStatus::Success
        });
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
