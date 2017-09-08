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
        field asset:       Asset       [32 => 40]
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
                println!("Asset {:?}", self.asset());
                creator.add_assets(self.asset());
                println!("Wallet after mining asset: {:?}", creator);
                schema.wallets().put(self.pub_key(), creator)
            }
        }

    }
}


#[test]
fn test_convert_from_json() {
    let json =
        r#"{
    "body": {
        "asset": {
            "amount": 3,
            "hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f"
        },
        "pub_key": "cdfe0378c3b7614410c468b7179cd5ba2b4ff3b9e5e24965b1aa23c5f623d28c",
        "seed": "13"
    },
    "message_id": 3,
    "network_id": 0,
    "protocol_version": 0,
    "service_id": 1,
    "signature": "501eece87348e14263a082f92c2bbc11aa02a216bd8a05073397d11add712ee7dc26c9868434e2992291fa924aefba4ae8dfeaa72fa229b31f49e439ce1f340a"
}"#;

    let tx_add: TxAddAsset = ::serde_json::from_str(&json).unwrap();
}