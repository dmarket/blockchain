extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use serde_json::Value;

use super::{SERVICE_ID, TX_ADD_ASSETS_ID};
use super::wallet::Asset;
use super::schema::wallet::WalletSchema;
use super::schema::asset::{AssetSchema, external_internal};
use super::schema::transaction_status::{TxStatusSchema, TxStatus};

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
        let mut wallet_schema = WalletSchema { view };
        let mut tx_status = TxStatus::Fail;
        let creator = wallet_schema.wallet(self.pub_key());
        if let Some(mut creator) = creator {

            if creator.balance() >= FEE_FOR_MINING {
                let mut asset_schema = AssetSchema{ view: wallet_schema.view };
                let map_assets = asset_schema.add_assets(self.assets(), self.pub_key());
                creator.decrease(FEE_FOR_MINING);
                println!("Convert {:?}", map_assets);
                let new_assets: Vec<Asset> = map_assets
                    .iter()
                    .map(|(_, asset)|{ Asset::new(asset.hash_id(),asset.amount())})
                    .collect();
                creator.add_assets(new_assets);
                tx_status = TxStatus::Success;
            }
            println!("Wallet after mining asset: {:?}", creator);
            wallet_schema.wallets().put(self.pub_key(), creator);
        }
        let mut tx_status_schema = TxStatusSchema{view: wallet_schema.view};
        tx_status_schema.set_status(&self.hash(), tx_status);
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "external_internal": external_internal(self.assets(), self.pub_key()),
            "tx_fee": FEE_FOR_MINING,
        })
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
  "service_id": 2,
  "message_id": 3,
  "signature": "3db781e2e944668788abaa7a5d5add868f8548662bcf01360988730539790c3f71d6a7f593e979aae891162d0f39c807d3cef20f39ccb8d7a4c4040db5733b0f"
}"#;

    let tx_add: TxAddAsset = ::serde_json::from_str(&json).unwrap();
    assert_eq!(3, tx_add.assets()[0].amount());
    assert_eq!("a8d5c97d-9978-444b-9947-7a95dfg31d0f", tx_add.assets()[1].hash_id());
}
