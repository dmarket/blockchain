extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use serde_json::Value;

use super::{SERVICE_ID, TX_CREATE_WALLET_ID, INIT_BALANCE};
use super::wallet::Wallet;
use super::schema::wallet::WalletSchema;
use super::schema::transaction_status::{TxStatusSchema, TxStatus};

message! {
    struct TxCreateWallet {
        const TYPE = SERVICE_ID;
        const ID = TX_CREATE_WALLET_ID;
        const SIZE = 32;

        field pub_key:     &PublicKey  [00 => 32]
    }
}

impl Transaction for TxCreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = WalletSchema { view };
        let tx_status = if schema.wallet(self.pub_key()).is_none() {
            let wallet = Wallet::new(self.pub_key(), INIT_BALANCE, vec![]);
            println!("Create the wallet: {:?}", wallet);
            schema.wallets().put(self.pub_key(), wallet);
            TxStatus::Success
        } else {
            TxStatus::Fail
        };
        let mut tx_status_schema = TxStatusSchema{view: schema.view};
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
    "pub_key": "06f2b8853d37d317639132d3e9646adee97c56dcbc3899bfb2b074477d7ef31a"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 1,
  "signature": "8b46f5e5034c4168c7bd8a305b7173c0467df3cea9b62fc8f0da03e1d9a6f9a09ca14d259f714ada1e7c52787bdbcaa5eaa3d940c4a5ced453a3c56930f73e0a"
}"#.to_string()
}

#[test]
fn test_convert_from_json() {
    let tx_create: TxCreateWallet = ::serde_json::from_str(&get_json()).unwrap();
    assert!(tx_create.verify());
}

#[test]
fn add_assets_test() {
    let tx_create: TxCreateWallet = ::serde_json::from_str(&get_json()).unwrap();

    let db = Box::new(MemoryDB::new());
    let mut wallet_schema = WalletSchema { view: &mut db.fork() };

    let wallet = Wallet::new(tx_create.pub_key(), 100, vec![]);
    assert_eq!(None, wallet_schema.wallet(tx_create.pub_key()));

    tx_create.execute(&mut wallet_schema.view);

    assert_eq!(Some(wallet), wallet_schema.wallet(tx_create.pub_key()));
}

#[test]
fn create_wallet_info_test() {
    let tx_create: TxCreateWallet = ::serde_json::from_str(&get_json()).unwrap();
    assert_eq!(0, tx_create.info()["tx_fee"]);
}
