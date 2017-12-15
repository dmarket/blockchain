extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::asset::Asset;
use service::transaction::{PER_ASSET_FEE, TRANSACTION_FEE};

use super::{SERVICE_ID, TX_TRADE_ASSETS_ID};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

const FEE_FOR_TRADE: f64 = 0.025;

encoding_struct! {
    struct TradeOffer {
        const SIZE = 48;

        field seller:              &PublicKey   [00 => 32]
        field assets:              Vec<Asset>   [32 => 40]
        field price:               u64          [40 => 48]
    }
}

message! {
    struct TxTrade {
        const TYPE = SERVICE_ID;
        const ID = TX_TRADE_ASSETS_ID;
        const SIZE = 112;

        field buyer:              &PublicKey    [00 => 32]
        field offer:              TradeOffer    [32 => 40]
        field seed:               u64           [40 => 48]
        field seller_signature:   &Signature    [48 => 112]

    }
}

impl TxTrade {
    fn offer_verify(&self) -> bool {
        *self.buyer() != *self.offer().seller() &&
            crypto::verify(
                self.seller_signature(),
                &self.offer().raw,
                self.offer().seller(),
            )
    }

    pub fn get_offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn get_fee(&self) -> u64 {
        //todo: необходимо определится с генергацией fee
        let price_fee = ((self.offer().price() as f64) * FEE_FOR_TRADE).round() as u64;
        price_fee + TRANSACTION_FEE + PER_ASSET_FEE * Asset::count(&self.offer().assets())
    }
}

impl Transaction for TxTrade {
    fn verify(&self) -> bool {
        self.offer_verify() && self.verify_signature(self.buyer())
    }

    fn execute(&self, view: &mut Fork) {
        let participants = WalletSchema::map(view, |mut schema| {
            (
                schema.wallet(self.buyer()),
                schema.wallet(self.offer().seller()),
            )
        });
        if let (Some(mut buyer), Some(mut seller)) = participants {
            let price = self.offer().price();
            let assets = self.offer().assets();
            println!("Buyer {:?} => Seller {:?}", buyer, seller);
            let tx_status = if (buyer.balance() >= price) && seller.in_wallet_assets(&assets) &&
                seller.balance() + price >= self.get_fee()
            //todo: необходимо определится с генергацией fee
            {
                println!("--   Trade transaction   --");
                println!("Seller's balance before transaction : {:?}", seller);
                println!("Buyer's balance before transaction : {:?}", buyer);
                let assets = self.offer().assets();
                seller.del_assets(&assets);
                seller.increase(price);
                seller.decrease(self.get_fee());
                let assets = self.offer().assets();
                buyer.add_assets(assets);
                buyer.decrease(price);
                println!("Seller's balance after transaction : {:?}", seller);
                println!("Buyer's balance after transaction : {:?}", buyer);
                WalletSchema::map(view, |mut schema| {
                    schema.wallets().put(self.buyer(), buyer);
                    schema.wallets().put(self.offer().seller(), seller);
                });
                TxStatus::Success
            } else {
                TxStatus::Fail
            };
            TxStatusSchema::map(
                view,
                |mut schema| schema.set_status(&self.hash(), tx_status),
            );
        }
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
    use super::TxTrade;
    use exonum::blockchain::Transaction;
    use exonum::storage::{Database, MemoryDB};
    use service::asset::{Asset, AssetID};
    use service::schema::wallet::WalletSchema;
    use service::wallet::Wallet;

    fn get_json() -> String {
        r#"{
            "body": {
                "buyer": "f2ab7abcae9363496ccc458a30ec0a58200d9890a12fdfeca35010da6b276e19",
                "offer": {
                "seller": "dedb2438fca19f04d2236d3005db0f28caa014f34caf98e23634cb49aef1c307",
                "assets": [
                    {
                    "id": "67e5504410b1426f9247bb680e5fe0c8",
                    "amount": 5
                    },
                    {
                    "id": "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8",
                    "amount": 7
                    }
                ],
                "price": "88"
                },
                "seed": "4",
                "seller_signature": "4e7d8d57fdc5c102b241d4e7a8d1228658c1de62a9334fa4e70776759268d67f9cdd8c4d20f7db8b226422c644bf442b0e28d9cbecece7753656c92915b02c06"
            },
            "network_id": 0,
            "protocol_version": 0,
            "service_id": 2,
            "message_id": 5,
            "signature": "aac7ce5fee4fca99fb66c978b94f42ba899834fa6b840491ab5c9245967b5a07bda688c0da52876258ee25d63dc1278cf97a6a90e84c8cb3880b5d6d3e606b06"
        }"#.to_string()
    }

    #[test]
    fn test_convert_from_json() {
        let tx: TxTrade = ::serde_json::from_str(&get_json()).unwrap();
        assert!(tx.verify());
        assert_eq!(5, tx.offer().assets()[0].amount());
        assert_eq!("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8", tx.offer().assets()[1].id().to_string());
        assert_eq!(88, tx.offer().price());
    }

    #[test]
    fn positive_trade_test() {
        let tx: TxTrade = ::serde_json::from_str(&get_json()).unwrap();

        let db = Box::new(MemoryDB::new());
        let fork = &mut db.fork();

        let assetid1 = AssetID::from_str("67e5504410b1426f9247bb680e5fe0c8").unwrap();
        let assetid2 = AssetID::from_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap();
        let seller = Wallet::new(
            tx.offer().seller(),
            tx.get_fee(),
            vec![
                Asset::new(assetid1, 10),
                Asset::new(assetid2, 7),
            ],
        );
        let buyer = Wallet::new(tx.buyer(), 3000, vec![]);
        WalletSchema::map(fork, |mut schema| {
            schema.wallets().put(tx.offer().seller(), seller);
            schema.wallets().put(tx.buyer(), buyer);
        });

        tx.execute(fork);

        let participants = WalletSchema::map(fork, |mut shema| {
            (shema.wallet(tx.offer().seller()), shema.wallet(tx.buyer()))
        });
        if let (Some(seller), Some(buyer)) = participants {
            assert_eq!(2912, buyer.balance());
            assert_eq!(88, seller.balance());
            assert_eq!(
                vec![Asset::new(assetid1, 5), ],
                seller.assets()
            );
            assert_eq!(
                vec![
                    Asset::new(assetid1, 5),
                    Asset::new(assetid2, 7),
                ],
                buyer.assets()
            );
        } else {
            panic!("Something wrong");
        }
    }

    #[test]
    fn exchange_info_test() {
        let tx: TxTrade = ::serde_json::from_str(&get_json()).unwrap();
        assert_eq!(tx.get_fee(), tx.info()["tx_fee"]);
    }
}
