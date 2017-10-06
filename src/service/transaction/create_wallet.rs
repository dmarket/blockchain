extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use serde_json::Value;

use super::{SERVICE_ID, TX_CREATE_WALLET_ID, INIT_BALANCE};
use service::wallet::{Wallet, Asset};
use service::schema::wallet::WalletSchema;

message! {
    struct TxCreateWallet {
        const TYPE = SERVICE_ID;
        const ID = TX_CREATE_WALLET_ID;
        const SIZE = 32;

        field pub_key:     &PublicKey  [00 => 32]
    }
}

impl Transaction for TxCreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = WalletSchema { view };
        if schema.wallet(self.pub_key()).is_none() {
            let assets: Vec<Asset> = vec![];
            let wallet = Wallet::new(self.pub_key(), INIT_BALANCE, assets);
            println!("Create the wallet: {:?}", wallet);
            schema.wallets().put(self.pub_key(), wallet)
        }
    }

    fn info(&self) -> Value {
        json!(self)
    }

}

#[test]
fn test_convert_from_json() {
    let json =
        r#"{
  "body": {
    "pub_key": "83dbc25eea26578cfdae481b421b09faeb1b35b98451a30c9a6a33271503e61a"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 1,
  "signature": "100c4bf9d50bd2da4af8d65b7b35847b0258d59d62b993311af4ce86049fa5de6712847db7b1a62d217e8c289bdf7b151552fac2404f965383c2c07fc39a5409"
}"#;

    let tx_create_wallet: TxCreateWallet = ::serde_json::from_str(&json).unwrap();
}
