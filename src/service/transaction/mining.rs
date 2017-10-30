extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use serde_json::Value;

use super::{SERVICE_ID, TX_MINING_ID, AMOUNT_MINING_COIN};
use super::wallet::{Wallet, Asset};
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
            let assets: Vec<Asset> = vec![];
            let wallet = Wallet::new(self.pub_key(), AMOUNT_MINING_COIN, assets);
            println!("Create the wallet: {:?}", wallet);
            schema.wallets().put(self.pub_key(), wallet);
            tx_status = TxStatus::Success;

        }
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

#[test]
fn test_convert_from_json() {
    let json =
        r#"{
  "body": {
    "pub_key": "36a05e418393fb4b23819753f6e6dd51550ce030d53842c43dd1349857a96a61",
    "seed": "25"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 7,
  "signature": "b609d76fb7861a914e89d68e61d16b7f395755b4cd78404205255814683ac3a92257da379cb10eba09fbd4c3ac6253abcca9fb47c9825f274cde95cfcc8a120b"
}
"#;

    let _: TxMining = ::serde_json::from_str(&json).unwrap();
}
