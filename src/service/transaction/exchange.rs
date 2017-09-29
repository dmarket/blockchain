extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;

use super::{SERVICE_ID, TX_EXCHANGE_ID};
//use service::wallet::Wallet;
use service::schema::currency::CurrencySchema;

message! {
    struct TxExchange {
        const TYPE = SERVICE_ID;
        const ID = TX_EXCHANGE_ID;
        const SIZE = 80;

        field from:        &PublicKey  [00 => 32]
        field to:          &PublicKey  [32 => 64]
        field amount:      u64         [64 => 72]
        field seed:        u64         [72 => 80]
    }
}

impl Transaction for TxExchange {
    fn verify(&self) -> bool {
        (*self.from() != *self.to()) && self.verify_signature(self.from())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = CurrencySchema { view };
        let sender = schema.wallet(self.from());
        let receiver = schema.wallet(self.to());
        if let (Some(mut sender), Some(mut receiver)) = (sender, receiver) {
            let amount = self.amount();
            if sender.balance() >= amount {
                sender.decrease(amount);
                receiver.increase(amount);
                println!("Transfer between wallets: {:?} => {:?}", sender, receiver);
                let mut wallets = schema.wallets();
                wallets.put(self.from(), sender);
                wallets.put(self.to(), receiver);
            }
        }
    }
}


#[test]
fn test_convert_from_json() {
    let json =
        r#"{
  "body": {
    "seller_body": {
        "pub_key": "83dbc25eea26578cfdae481b421b09faeb1b35b98451a30c9a6a33271503e61a"
        "asset": {"hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", "amount":2}
        "value": "10",
        "signature": "100c4bf9d50bd2da4af8d65b7b35847b0258d59d62b993311af4ce86049fa5de6712847db7b1a62d217e8c289bdf7b151552fac2404f965383c2c07fc39a5409"
    },
    "pub_key": "83dbc25eea26578cfdae481b421b09faeb1b35b98451a30c9a6a33271503e61a"
    "seed": "123123123123"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 1,
  "signature": "100c4bf9d50bd2da4af8d65b7b35847b0258d59d62b993311af4ce86049fa5de6712847db7b1a62d217e8c289bdf7b151552fac2404f965383c2c07fc39a5409"
}"#;

    let tx_create_wallet: TxCreateWallet = ::serde_json::from_str(&json).unwrap();
}