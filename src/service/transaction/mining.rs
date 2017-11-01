extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use serde_json::Value;

use super::{SERVICE_ID, TX_MINING_ID, AMOUNT_MINING_COIN};
use super::wallet::Wallet;
use super::schema::wallet::WalletSchema;
use super::schema::transaction_status::{TxStatusSchema, TxStatus};

message! {
    struct TxMining {
        const TYPE = SERVICE_ID;
        const ID = TX_MINING_ID;
        const SIZE = 40;

        field pub_key:     &PublicKey  [00 => 32]
        field seed:        u64         [32 => 40]
    }
}

impl Transaction for TxMining {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = WalletSchema { view };
        let tx_status;
        if let Some(mut wallet) = schema.wallet(self.pub_key()) {
            wallet.increase(AMOUNT_MINING_COIN);
            println!("Mining {} the wallet: {:?}", AMOUNT_MINING_COIN, wallet);
            schema.wallets().put(self.pub_key(), wallet);
            tx_status = TxStatus::Success;
        } else {
            let wallet = Wallet::new(self.pub_key(), AMOUNT_MINING_COIN, vec![]);
            println!("Create the wallet: {:?}", wallet);
            schema.wallets().put(self.pub_key(), wallet);
            tx_status = TxStatus::Success;

        }
        let mut tx_status_schema = TxStatusSchema { view: schema.view };
        tx_status_schema.set_status(&self.hash(), tx_status);
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "tx_fee": 0,
        })
    }
}
#[cfg(test)]
use exonum::storage::{MemoryDB, Database};

#[cfg(test)]
fn get_json() -> String {
    r#"{
  "body": {
    "pub_key": "e61b4b9945defd1878d7575ddc50993f6a074cdfcafc47d15cba46860cab0060",
    "seed": "43"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 7,
  "signature": "671540cb1bf737c109e7ba7f90364cafa4064f8e7d54cdc74ae31711061efc2f3116be128a09d642970980f87beb19f948f5148f0cd544ba926c2acd304b6d09"
}"#.to_string()
}

#[test]
fn test_convert_from_json() {
    let tx: TxMining = ::serde_json::from_str(&get_json()).unwrap();
    assert!(tx.verify());
}

#[test]
fn mining_test() {
    let tx: TxMining = ::serde_json::from_str(&get_json()).unwrap();

    let db = Box::new(MemoryDB::new());
    let mut wallet_schema = WalletSchema { view: &mut db.fork() };

    let wallet = Wallet::new(tx.pub_key(), 100, vec![]);
    wallet_schema.wallets().put(tx.pub_key(), wallet);

    tx.execute(&mut wallet_schema.view);

    if let Some(wallet) = wallet_schema.wallet(tx.pub_key()) {
        assert_eq!(AMOUNT_MINING_COIN+100, wallet.balance());
    } else {
        panic!("Something wrong!!!");
    }
}

#[test]
fn mining_coin_info_test() {
    let tx: TxMining = ::serde_json::from_str(&get_json()).unwrap();
    assert_eq!(0, tx.info()["tx_fee"]);
}
