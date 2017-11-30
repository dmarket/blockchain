extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use serde_json::Value;

use service::transaction::{TRANSACTION_FEE, PER_ASSET_FEE};

use super::{SERVICE_ID, TX_DEL_ASSETS_ID};
use service::wallet::Asset;
use service::schema::asset::AssetSchema;
use service::schema::wallet::WalletSchema;

message! {
    struct TxDelAsset {
        const TYPE = SERVICE_ID;
        const ID = TX_DEL_ASSETS_ID;
        const SIZE = 48;

        field pub_key:     &PublicKey  [00 => 32]
        field assets:      Vec<Asset>  [32 => 40]
        field seed:        u64         [40 => 48]
    }
}

impl TxDelAsset {
    fn get_fee(&self) -> u64 {
        TRANSACTION_FEE + PER_ASSET_FEE * Asset::count(&self.assets())
    }
}

impl Transaction for TxDelAsset {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        // Invariant: for an asset id, the sum of amounts for all assets in
        // all wallets for this asset id is equal to the amonut stored in the
        // AssetInfo associated with this asset id.

        let mut schema = AssetSchema { view };
        for a in self.assets() {
            match schema.info(a.hash_id()) {
                Some(ref info) if info.creator() != self.pub_key() => return,
                None => return,
                _ => (),
            }
        }

        let mut schema = WalletSchema { view: schema.view };
        match schema.wallet(self.pub_key()) {
            Some(mut creator) => {
                if creator.balance() >= self.get_fee() && creator.del_assets(&self.assets()) {
                    creator.decrease(self.get_fee());
                    println!("Asset {:?}", self.assets());
                    println!("Wallet after delete assets: {:?}", creator);
                    schema.wallets().put(self.pub_key(), creator);
                }
            }
            _ => return,
        }

        let mut schema = AssetSchema { view: schema.view };
        schema.del_assets(&self.assets());
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "tx_fee": self.get_fee(),
        })
    }
}


#[cfg(test)]
mod test {
    use exonum::storage::{MemoryDB, Database};
    use exonum::blockchain::Transaction;
    use service::schema::asset::AssetSchema;
    use service::schema::wallet::WalletSchema;
    use service::wallet::{Wallet, Asset, AssetInfo};
    use service::transaction::del_assets::{TxDelAsset, FEE_FOR_MINING};

    fn get_json() -> String {
        r#"{
        "body": {
            "pub_key": "1d9c731ebac3d7da9482470ae8b13a839cb05ef4f21f8d119e2c4bf175333cf7",
            "assets": [
                {
                    "hash_id": "asset_1",
                    "amount": 45
                },
                {
                  "hash_id": "asset_2",
                  "amount": 17
                }
            ],
        "seed": "113"
        },
        "network_id": 0,
        "protocol_version": 0,
        "service_id": 2,
        "message_id": 4,
        "signature": "e7a3d71fc093f9ddaba083ba3e1618514c96003d9a01cdf6d5c0da344f12c800db9e7b210f9a7b372ddd7e57f299d8bc0e55d238ad1fa6b9d06897c2bda29901"
    }"#.to_string()
    }

    #[test]
    fn test_convert_from_json() {
        let tx_del: TxDelAsset = ::serde_json::from_str(&get_json()).unwrap();
        assert!(tx_del.verify());
        assert_eq!(45, tx_del.assets()[0].amount());
        assert_eq!("asset_2", tx_del.assets()[1].hash_id());
    }

    #[test]
    fn positive_delete_assets_test() {
        let tx_del: TxDelAsset = ::serde_json::from_str(&get_json()).unwrap();

        let db = Box::new(MemoryDB::new());

        let mut asset_schema = AssetSchema { view: &mut db.fork() };
        asset_schema.assets().put(
            &"asset_1".to_string(),
            AssetInfo::new(tx_del.pub_key(), 100),
        );
        asset_schema.assets().put(
            &"asset_2".to_string(),
            AssetInfo::new(tx_del.pub_key(), 17),
        );
        {
            let assets = asset_schema.assets();
            for info in assets.iter() {
                println!("Put info: {:?}", info);
            }
        }

        let mut wallet_schema = WalletSchema { view: &mut asset_schema.view };

        let assets = vec![
            Asset::new("asset_1", 100),
            Asset::new("asset_2", 17),
        ];

        let wallet = Wallet::new(tx_del.pub_key(), 2000, assets);
        wallet_schema.wallets().put(tx_del.pub_key(), wallet);

        tx_del.execute(&mut wallet_schema.view);

        if let Some(wallet) = wallet_schema.wallet(tx_del.pub_key()) {
            assert!(wallet.in_wallet_assets(&vec![
                                        Asset::new("asset_1", 55)
            ]));
            assert!(!wallet.in_wallet_assets(&vec![
                                         Asset::new("asset_2", 0)
            ]));
        } else {
            panic!("Something wrong!!!");
        }
    }

    #[test]
    fn negative_delete_assets_test() {
        let tx_del: TxDelAsset = ::serde_json::from_str(&get_json()).unwrap();

        let db = Box::new(MemoryDB::new());
        let mut wallet_schema = WalletSchema { view: &mut db.fork() };

        let assets = vec![
            Asset::new("asset_1", 400),
        ];

        let wallet = Wallet::new(tx_del.pub_key(), 100, assets);
        wallet_schema.wallets().put(tx_del.pub_key(), wallet);

        tx_del.execute(&mut wallet_schema.view);

        if let Some(wallet) = wallet_schema.wallet(tx_del.pub_key()) {
            assert!(wallet.in_wallet_assets(&vec![
                                        Asset::new("asset_1", 400)
            ]));
        } else {
            panic!("Something wrong!!!");
        }
    }

    #[test]
    fn add_asset_info_test() {
        let tx: TxDelAsset = ::serde_json::from_str(&get_json()).unwrap();
        assert_eq!(tx.get_fee(), tx.info()["tx_fee"]);
    }
}