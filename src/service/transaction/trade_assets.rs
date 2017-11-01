extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::{PublicKey, Signature, verify};
use exonum::messages::Message;
use serde_json::Value;


use super::{SERVICE_ID, TX_TRADE_ASSETS_ID};
use super::wallet::Asset;
use super::schema::wallet::WalletSchema;
use super::schema::transaction_status::{TxStatusSchema, TxStatus};

pub const FEE_FOR_TRADE: f64 = 0.025;

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
            verify(
                self.seller_signature(),
                &self.offer().raw,
                self.offer().seller(),
            )
    }

    pub fn get_offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn get_fee(&self) -> u64 {
        ((self.offer().price() as f64) * FEE_FOR_TRADE).round() as u64
    }
}

impl Transaction for TxTrade {
    fn verify(&self) -> bool {
        self.offer_verify() && self.verify_signature(self.buyer())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = WalletSchema { view };
        let buyer = schema.wallet(self.buyer());
        let seller = schema.wallet(self.offer().seller());
        if let (Some(mut buyer), Some(mut seller)) = (buyer, seller) {
            let price = self.offer().price();
            let assets = self.offer().assets();
            println!("Buyer {:?} => Seller {:?}", buyer, seller);
            let tx_status = if (buyer.balance() >= price) && seller.in_wallet_assets(assets) {
                println!("--   Trade transaction   --");
                println!("Seller's balance before transaction : {:?}", seller);
                println!("Buyer's balance before transaction : {:?}", buyer);
                let assets = self.offer().assets();
                seller.del_assets(assets);
                seller.increase(price - self.get_fee());
                let assets = self.offer().assets();
                buyer.add_assets(assets);
                buyer.decrease(price);
                println!("Seller's balance after transaction : {:?}", seller);
                println!("Buyer's balance after transaction : {:?}", buyer);
                let mut wallets = schema.wallets();
                wallets.put(self.buyer(), buyer);
                wallets.put(self.offer().seller(), seller);
                TxStatus::Success
            } else {
                TxStatus::Fail
            };
            let mut tx_status_schema = TxStatusSchema { view: schema.view };
            tx_status_schema.set_status(&self.hash(), tx_status);
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
use exonum::storage::{MemoryDB, Database};
#[cfg(test)]
use service::wallet::Wallet;

#[cfg(test)]
fn get_json() -> String {
    r#"{
  "body": {
    "buyer": "f2ab7abcae9363496ccc458a30ec0a58200d9890a12fdfeca35010da6b276e19",
    "offer": {
      "seller": "dedb2438fca19f04d2236d3005db0f28caa014f34caf98e23634cb49aef1c307",
      "assets": [
        {
          "hash_id": "a4826063-d7bb-57a3-a119-3ba03a51b7fa",
          "amount": 5
        },
        {
          "hash_id": "a007f130-ceea-5939-b616-3aaf7185a164",
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
    assert_eq!("a007f130-ceea-5939-b616-3aaf7185a164", tx.offer().assets()[1].hash_id());
    assert_eq!(88, tx.offer().price());
}

#[test]
fn positive_trade_test() {
    let tx: TxTrade = ::serde_json::from_str(&get_json()).unwrap();

    let db = Box::new(MemoryDB::new());
    let mut wallet_schema = WalletSchema { view: &mut db.fork() };

    let seller = Wallet::new(
        tx.offer().seller(),
        0,
        vec![
            Asset::new("a4826063-d7bb-57a3-a119-3ba03a51b7fa", 10),
            Asset::new("a007f130-ceea-5939-b616-3aaf7185a164", 7),
        ],
    );
    let buyer = Wallet::new(tx.buyer(), 100, vec![]);

    wallet_schema.wallets().put(tx.offer().seller(), seller);
    wallet_schema.wallets().put(tx.buyer(), buyer);

    tx.execute(&mut wallet_schema.view);

    let seller = wallet_schema.wallet(tx.offer().seller());
    let buyer = wallet_schema.wallet(tx.buyer());
    if let (Some(seller), Some(buyer)) = (seller, buyer) {
        assert_eq!(12, buyer.balance());
        assert_eq!(88 - tx.get_fee(), seller.balance());
        assert_eq!(
            vec![Asset::new("a4826063-d7bb-57a3-a119-3ba03a51b7fa", 5),],
            seller.assets()
        );
        assert_eq!(
            vec![
                Asset::new("a4826063-d7bb-57a3-a119-3ba03a51b7fa", 5),
                Asset::new("a007f130-ceea-5939-b616-3aaf7185a164", 7),
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
