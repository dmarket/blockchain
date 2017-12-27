extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::transaction::TX_DEL_ASSET_FEE;

use super::{SERVICE_ID, TX_DEL_ASSETS_ID};
use service::asset::Asset;
use service::schema::asset::AssetSchema;
use service::schema::transaction_status::{TxStatus, TxStatusSchema};
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
    pub fn get_fee(&self) -> u64 {
        TX_DEL_ASSET_FEE
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        // Invariant: for an asset id, the sum of amounts for all assets in
        // all wallets for this asset id is equal to the amonut stored in the
        // AssetInfo associated with this asset id.

        for a in self.assets() {
            match AssetSchema::map(view, |mut assets| assets.info(&a.id())) {
                Some(ref info) => {
                    if info.creator() != self.pub_key() || a.amount() > info.amount() {
                        return TxStatus::Fail;
                    }
                }
                None => return TxStatus::Fail,
            }
        }

        match WalletSchema::map(view, |mut schema| schema.wallet(self.pub_key())) {
            Some(mut creator) => {
                if creator.balance() >= self.get_fee() && creator.del_assets(&self.assets()) {
                    creator.decrease(self.get_fee());
                    println!("Asset {:?}", self.assets());
                    println!("Wallet after delete assets: {:?}", creator);
                    WalletSchema::map(view, |mut schema| {
                        schema.wallets().put(self.pub_key(), creator)
                    });
                }
            }
            _ => return TxStatus::Fail,
        }

        AssetSchema::map(view, |mut schema| schema.del_assets(&self.assets()));
        TxStatus::Success
    }
}

impl Transaction for TxDelAsset {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let tx_status = self.process(view);
        TxStatusSchema::map(
            view,
            |mut schema| schema.set_status(&self.hash(), tx_status),
        );
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "tx_fee": self.get_fee(),
        })
    }
}


#[cfg(test)]
mod tests {
    use exonum::blockchain::Transaction;
    use service::transaction::del_assets::TxDelAsset;

    fn get_json() -> String {
        r#"{
            "body": {
                "pub_key": "1d9c731ebac3d7da9482470ae8b13a839cb05ef4f21f8d119e2c4bf175333cf7",
                "assets": [
                    {
                        "id": "67e5504410b1426f9247bb680e5fe0c8",
                        "amount": 45
                    },
                    {
                        "id": "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8",
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
    fn test_add_asset_info() {
        let tx: TxDelAsset = ::serde_json::from_str(&get_json()).unwrap();
        assert_eq!(tx.get_fee(), tx.info()["tx_fee"]);
    }
}
