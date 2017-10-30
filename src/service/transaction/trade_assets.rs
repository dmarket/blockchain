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
            self.offer().seller()
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
            } else { TxStatus::Fail };
            let mut tx_status_schema = TxStatusSchema{view: schema.view};
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


#[test]
fn test_trade_convert_from_json() {
    let json =
        r#"{
  "body": {
    "buyer": "83dbc25eea26578cfdae481b421b09faeb1b35b98451a30c9a6a33271503e61a",
    "offer": {
        "seller": "83dbc25eea26578cfdae481b421b09faeb1b35b98451a30c9a6a33271503e61a",
        "assets": [{"hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", "amount":2}, {"hash_id": "a8d5c97d-9978-4b0c-9947-7a95dcb31d0f", "amount":3}],
        "price": "10"
    },
    "seller_signature": "100c4bf9d50bd2da4af8d65b7b35847b0258d59d62b993311af4ce86049fa5de6712847db7b1a62d217e8c289bdf7b151552fac2404f965383c2c07fc39a5409",
    "seed": "123123123123"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 5,
  "signature": "100c4bf9d50bd2da4af8d65b7b35847b0258d59d62b993311af4ce86049fa5de6712847db7b1a62d217e8c289bdf7b151552fac2404f965383c2c07fc39a5409"
}"#;

    let _: TxTrade = ::serde_json::from_str(&json).unwrap();
}
