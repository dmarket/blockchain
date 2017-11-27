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
                let mut asset_schema = AssetSchema { view: wallet_schema.view };
                let map_assets = asset_schema.add_assets(self.assets(), self.pub_key());
                creator.decrease(FEE_FOR_MINING);
                println!("Convert {:?}", map_assets);
                let new_assets: Vec<Asset> = map_assets
                    .iter()
                    .map(|(_, asset)| Asset::new(asset.hash_id(), asset.amount()))
                    .collect();
                creator.add_assets(new_assets);
                tx_status = TxStatus::Success;
            }
            println!("Wallet after mining asset: {:?}", creator);
            wallet_schema.wallets().put(self.pub_key(), creator);
        }
        let mut tx_status_schema = TxStatusSchema { view: wallet_schema.view };
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

#[cfg(test)]
use service::wallet::Wallet;
#[cfg(test)]
use exonum::storage::{MemoryDB, Database};

#[cfg(test)]
fn get_json() -> String {
    r#"{
  "body": {
    "pub_key": "06f2b8853d37d317639132d3e9646adee97c56dcbc3899bfb2b074477d7ef31a",
    "assets": [
      {
        "hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f",
        "amount": 45
      },
      {
        "hash_id": "a8d5c97d-9978-4111-9947-7a95dcb31d0f",
        "amount": 17
      }
    ],
    "seed": "85"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 3,
  "signature": "11ab7e8236084cb68fe949242f7107068ca54ad3cdfd927a933a282c4781b2f2b4993824eb2dc2b0dc275d1a86bbb8f3b48640680cc1258bb7000748c2b29407"
}"#.to_string()
}

#[test]
fn test_convert_from_json() {
    let tx_add: TxAddAsset = ::serde_json::from_str(&get_json()).unwrap();
    assert!(tx_add.verify());
    assert_eq!(45, tx_add.assets()[0].amount());
    assert_eq!("a8d5c97d-9978-4111-9947-7a95dcb31d0f", tx_add.assets()[1].hash_id());
}

#[test]
fn add_assets_test() {
    let tx_add: TxAddAsset = ::serde_json::from_str(&get_json()).unwrap();
    let internal_assets_ids = external_internal(tx_add.assets(), tx_add.pub_key());
    let internal_a_id_1 = &internal_assets_ids[&"a8d5c97d-9978-4111-9947-7a95dcb31d0f".to_string()];
    let internal_a_id_2 = &internal_assets_ids[&"a8d5c97d-9978-4b0b-9947-7a95dcb31d0f".to_string()];

    let db = Box::new(MemoryDB::new());
    let mut wallet_schema = WalletSchema { view: &mut db.fork() };

    let wallet = Wallet::new(tx_add.pub_key(), 100, vec![Asset::new(internal_a_id_1, 3),]);
    wallet_schema.wallets().put(tx_add.pub_key(), wallet);

    if let Some(wallet) = wallet_schema.wallet(tx_add.pub_key()) {
        assert!(wallet.in_wallet_assets(&vec![Asset::new(internal_a_id_1, 3)]));
        tx_add.execute(&mut wallet_schema.view);
        if let Some(wallet) = wallet_schema.wallet(tx_add.pub_key()) {
            assert_eq!(100 - FEE_FOR_MINING, wallet.balance());
            assert!(wallet.in_wallet_assets(&vec![Asset::new(internal_a_id_1, 20),]));
            assert!(wallet.in_wallet_assets(&vec![Asset::new(internal_a_id_2, 45),]));
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}

#[test]
fn add_asset_info_test() {
    let tx_add: TxAddAsset = ::serde_json::from_str(&get_json()).unwrap();
    assert_eq!(FEE_FOR_MINING, tx_add.info()["tx_fee"]);
}
