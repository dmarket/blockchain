extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::asset::{Asset, MetaAsset};
use service::transaction::{PER_ASSET_FEE, TRANSACTION_FEE};

use super::{SERVICE_ID, TX_ADD_ASSETS_ID};
use super::schema::asset::{AssetSchema, external_internal};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

message! {
    struct TxAddAsset {
        const TYPE = SERVICE_ID;
        const ID = TX_ADD_ASSETS_ID;
        const SIZE = 48;

        field pub_key:       &PublicKey       [00 => 32]
        field meta_assets:   Vec<MetaAsset>   [32 => 40]
        field seed:          u64              [40 => 48]
    }
}

impl TxAddAsset {
    pub fn get_fee(&self) -> u64 {
        TRANSACTION_FEE + PER_ASSET_FEE * MetaAsset::count(&self.meta_assets())
    }
}

impl Transaction for TxAddAsset {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        if self.verify_signature(self.pub_key()) {
            for asset in self.meta_assets().iter() {
                if !asset.is_valid() {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    fn execute(&self, view: &mut Fork) {
        let mut tx_status = TxStatus::Fail;
        let creator = WalletSchema::map(view, |mut schema| schema.wallet(self.pub_key()));
        if let Some(mut creator) = creator {
            if creator.balance() >= self.get_fee() {
                let map_assets = AssetSchema::map(view, |mut schema| {
                    schema.add_assets(self.meta_assets(), self.pub_key())
                });
                creator.decrease(self.get_fee());
                println!("Convert {:?}", map_assets);
                let new_assets: Vec<Asset> = map_assets
                    .iter()
                    .map(|(_, asset)| Asset::new(asset.hash_id(), asset.amount()))
                    .collect();
                creator.add_assets(new_assets);
                tx_status = TxStatus::Success;
            }
            println!("Wallet after mining asset: {:?}", creator);
            WalletSchema::map(view, |mut schema| {
                schema.wallets().put(self.pub_key(), creator)
            });
        }
        TxStatusSchema::map(
            view,
            |mut schema| schema.set_status(&self.hash(), tx_status),
        );
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "external_internal": external_internal(self.meta_assets(), self.pub_key()),
            "tx_fee": self.get_fee(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TxAddAsset;

    use exonum::blockchain::Transaction;

    fn get_json() -> String {
        r#"{
            "body": {
                "pub_key": "06f2b8853d37d317639132d3e9646adee97c56dcbc3899bfb2b074477d7ef31a",
                "meta_assets": [
                {
                    "data": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f",
                    "amount": 45
                },
                {
                    "data": "a8d5c97d-9978-4111-9947-7a95dcb31d0f",
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
    fn test_add_asset_info() {
        let tx_add: TxAddAsset = ::serde_json::from_str(&get_json()).unwrap();
        assert_eq!(tx_add.get_fee(), tx_add.info()["tx_fee"]);
    }
}
