extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use service::wallet::Asset;
use serde_json::Value;

use super::{SERVICE_ID, TX_TRANSFER_ID};
//use service::wallet::Wallet;
use service::schema::wallet::WalletSchema;

pub const FEE_FOR_TRANSFER: u64 = 1;

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

impl Transaction for TxTransfer {
    fn verify(&self) -> bool {
        (*self.from() != *self.to()) && self.verify_signature(self.from())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = WalletSchema { view };
        if let Some(mut sender) = schema.wallet(self.from()) {
            let amount = self.amount();
            let update_amount = amount == 0 && sender.balance() >= FEE_FOR_TRANSFER || amount > 0 && sender.balance() >= amount + FEE_FOR_TRANSFER;
            let update_assets = self.assets().len() == 0 || self.assets().len() > 0 && sender.in_wallet_assets(self.assets());
            if update_amount && update_assets {
                sender.decrease(amount + FEE_FOR_TRANSFER);
                sender.del_assets(self.assets());
                let mut receiver = schema.create_wallet(self.to());
                receiver.increase(amount);
                receiver.add_assets(self.assets());

                println!("Transfer between wallets: {:?} => {:?}", sender, receiver);
                let mut wallets = schema.wallets();
                wallets.put(self.from(), sender);
                wallets.put(self.to(), receiver);
            }
        }
    }

    fn info(&self) -> Value {
        json!(self)
    }

}



#[test]
fn test_transfer_convert_from_json() {
    let json =
        r#"{
  "body": {
    "from": "cdfe0378c3b7614410c468b7179cd5ba2b4ff3b9e5e24965b1aa23c5f623d28c",
    "to": "cdfe0378c3b7614410c468b7179cd5ba2b4ff3b9e5e24965b1aa23c5f623d28c",
    "amount": "13",
    "assets": [
      {
        "hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f",
        "amount": 2
      },
      {
        "hash_id": "a8d5c97d-9978-444b-9947-7a95dfg31d0f",
        "amount": 1
      }
    ],
    "seed": "13"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 2,
  "signature": "3db781e2e944668788abaa7a5d5add868f8548662bcf01360988730539790c3f71d6a7f593e979aae891162d0f39c807d3cef20f39ccb8d7a4c4040db5733b0f"
}"#;

    let tx_add: TxTransfer = ::serde_json::from_str(&json).unwrap();
}