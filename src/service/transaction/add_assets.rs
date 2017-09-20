extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;

use super::{SERVICE_ID, TX_ADD_ASSETS_ID};
use service::wallet::Asset;
use service::schema::currency::CurrencySchema;

pub const FEE_FOR_MINING: u64 = 1;

message! {
    struct TxAddAsset {
        const TYPE = SERVICE_ID;
        const ID = TX_ADD_ASSETS_ID;
        const SIZE = 48;

        field pub_key:     &PublicKey  [00 => 32]
        field assets:      Vec<Asset>  [32 => 40]
        field seed:        u64         [40 => 48]
    }
}

impl Transaction for TxAddAsset {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = CurrencySchema { view };
        let creator = schema.wallet(self.pub_key());
        if let Some(mut creator) = creator {

            if creator.balance() >= FEE_FOR_MINING {
                creator.decrease(FEE_FOR_MINING);
                println!("Assets {:?}", self.assets());
                creator.add_assets(self.assets());
                println!("Wallet after mining asset: {:?}", creator);
                schema.wallets().put(self.pub_key(), creator)
            }
        }

    }
}


#[test]
fn test_convert_from_json() {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let json =
        r#"{
  "body": {
    "pub_key": "cdfe0378c3b7614410c468b7179cd5ba2b4ff3b9e5e24965b1aa23c5f623d28c",
    "assets": [
      {
        "hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f",
        "amount": 3
      },
      {
        "hash_id": "a8d5c97d-9978-444b-9947-7a95dfg31d0f",
        "amount": 7
      }
    ],
    "seed": "13"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 1,
  "message_id": 3,
  "signature": "3db781e2e944668788abaa7a5d5add868f8548662bcf01360988730539790c3f71d6a7f593e979aae891162d0f39c807d3cef20f39ccb8d7a4c4040db5733b0f"
}"#;

    let tx_add: TxAddAsset = ::serde_json::from_str(&json).unwrap();
    assert_eq!(3, tx_add.assets()[0].amount());
    assert_eq!("a8d5c97d-9978-444b-9947-7a95dfg31d0f", tx_add.assets()[1].hash_id());
}