extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use serde_json::Value;


use super::{SERVICE_ID, TX_TRADE_ASSETS_ID};
use service::wallet::Asset;
use service::schema::wallet::WalletSchema;

pub const FEE_FOR_TRADE: u64 = 1;

encoding_struct! {
    struct Offer {
        const SIZE = 112;

        field seller:              &PublicKey   [00 => 32]
        field assets:              Vec<Asset>   [32 => 40]
        field price:               u64          [40 => 48]
        field signature:           &Signature   [48 => 112]
    }
}

message! {
    struct TxTrade {
        const TYPE = SERVICE_ID;
        const ID = TX_TRADE_ASSETS_ID;
        const SIZE = 48;

        field buyer:         &PublicKey  [00 => 32]
        field offer:         Offer       [32 => 40]
        field seed:          u64         [40 => 48]
    }
}

encoding_struct! {
    struct OfferData {
        const SIZE = 48;

        field seller:              &PublicKey   [00 => 32]
        field assets:              Vec<Asset>   [32 => 40]
        field price:               u64          [40 => 48]
    }
}

impl TxTrade {
    fn offer_verify(&self) -> bool {
        let offer_data = OfferData::new(
            self.offer().seller(),
            self.offer().assets(),
            self.offer().price()
        );
        crypto::verify(
            self.offer().signature(),
            &offer_data.raw,
            self.offer().seller()
        )
    }
}

impl Transaction for TxTrade {
    fn verify(&self) -> bool {
        self.offer_verify() &&
        self.verify_signature(self.buyer())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = WalletSchema { view };
        let buyer = schema.wallet(self.buyer());
        let seller = schema.wallet(self.offer().seller());
        if let (Some(mut buyer), Some(mut seller)) = (buyer, seller) {
            let price = self.offer().price();
            let assets = self.offer().assets();
            println!("{:?} => {:?}", buyer, seller);

            if (buyer.balance() >= price + FEE_FOR_TRADE) && seller.in_wallet_assets(assets) {
                println!("--   Trade transaction   --");
                println!("Seller's balance before transaction : {:?}", seller);
                println!("Buyer's balance before transaction : {:?}", buyer);
                let assets = self.offer().assets();
                seller.del_assets(assets);
                seller.increase(price);
                let assets = self.offer().assets();
                buyer.add_assets(assets);
                buyer.decrease(price + FEE_FOR_TRADE);
                println!("Seller's balance after transaction : {:?}", seller);
                println!("Buyer's balance after transaction : {:?}", buyer);
                let mut wallets = schema.wallets();
                wallets.put(self.buyer(), buyer);
                wallets.put(self.offer().seller(), seller);
            }
        }
    }

    fn info(&self) -> Value {
        json!(self)
    }

}


#[test]
fn test_trade_convert_from_json() {
    let json =
        r#"{
  "body": {
    "buyer": "83dbc25eea26578cfdae481b421b09faeb1b35b98451a30c9a6a33271503e61a",
    "offer": {
        "seller": "83dbc25eea26578cfdae481b421b09faeb1b35b98451a30c9a6a33271503e61a",
        "assets": [{"hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", "amount":2}, {"hash_id": "a8d5c97d-9978-4b0c-9947-7a95dcb31d0f", "amount":3}],
        "price": "10",
        "signature": "100c4bf9d50bd2da4af8d65b7b35847b0258d59d62b993311af4ce86049fa5de6712847db7b1a62d217e8c289bdf7b151552fac2404f965383c2c07fc39a5409"
    },
    "seed": "123123123123"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 5,
  "signature": "100c4bf9d50bd2da4af8d65b7b35847b0258d59d62b993311af4ce86049fa5de6712847db7b1a62d217e8c289bdf7b151552fac2404f965383c2c07fc39a5409"
}"#;

    let tx_trade: TxTrade = ::serde_json::from_str(&json).unwrap();
}