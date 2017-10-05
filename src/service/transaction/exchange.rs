extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::{PublicKey, Signature, verify};
use exonum::messages::Message;
use serde_json::Value;

use super::{SERVICE_ID, TX_EXCHANGE_ID};
use service::wallet::Asset;
use service::schema::currency::CurrencySchema;

encoding_struct! {
    struct Offer {
        const SIZE = 104;

        field sender:                 &PublicKey   [00 => 32]
        field sender_assets:          Vec<Asset>   [32 => 40]
        field sender_value:           u64          [40 => 48]

        field recipient:              &PublicKey   [48 => 80]
        field recipient_assets:       Vec<Asset>   [88 => 96]
        field recipient_value:        u64          [96 => 104]
    }
}

message! {
    struct TxExchange {
        const TYPE = SERVICE_ID;
        const ID = TX_EXCHANGE_ID;
        const SIZE = 80;

        field offer:             Offer        [00 => 08]
        field seed:              u64          [08 => 16]
        field sender_signature:  &Signature   [16 => 80]
    }
}

impl Transaction for TxExchange {
    fn verify(&self) -> bool {
        *self.offer().sender() != *self.offer().recipient() &&
        self.verify_signature(self.offer().recipient()) &&
        verify(
            self.sender_signature(),
            &self.offer().raw,
            self.offer().sender()
        )

    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = CurrencySchema { view };
        let sender = schema.wallet(self.offer().sender());
        let recipient = schema.wallet(self.offer().recipient());
        if let (Some(mut sender), Some(mut recipient)) = (sender, recipient) {
            if sender.balance() >= self.offer().sender_value() &&
                sender.in_wallet_assets(self.offer().sender_assets()) &&
                recipient.balance() >= self.offer().recipient_value() &&
                recipient.in_wallet_assets(self.offer().recipient_assets())
            {
                println!("--   Exchange transaction   --");
                println!("Sender's balance before transaction : {:?}", sender);
                println!("Recipient's balance before transaction : {:?}", recipient);

                sender.decrease(self.offer().sender_value());
                recipient.increase(self.offer().sender_value());

                sender.increase(self.offer().recipient_value());
                recipient.decrease(self.offer().recipient_value());

                sender.del_assets(self.offer().sender_assets());
                recipient.add_assets(self.offer().sender_assets());

                sender.add_assets(self.offer().recipient_assets());
                recipient.del_assets(self.offer().recipient_assets());

                println!("Sender's balance before transaction : {:?}", sender);
                println!("Recipient's balance before transaction : {:?}", recipient);
                let mut wallets = schema.wallets();
                wallets.put(self.offer().sender(), sender);
                wallets.put(self.offer().recipient(), recipient);
            }
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
    "offer": {
        "sender": "83dbc25eea26578cfdae481b421b09faeb1b35b98451a30c9a6a33271503e61a",
        "sender_assets": [{"hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", "amount":2}, {"hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", "amount":2}],
        "sender_value": "10",
        "recipient": "83dbc25eea26578cfdae481b421b09faeb1b35b98451a30c9a6a33271503e61a",
        "recipient_assets": [{"hash_id": "a8d5c97d-9978-300b-9947-7a95dcb31d0f", "amount":2}, {"hash_id": "a8d5c97d-9978-310b-9947-7a95dcb31d0f", "amount":2}],
        "recipient_value": "13"
    },
    "seed": "123123123123",
    "sender_signature": "100c4bf9d50bd2da4af8d65b7b35847b0258d59d62b993311af4ce86049fa5de6712847db7b1a62d217e8c289bdf7b151552fac2404f965383c2c07fc39a5409"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 6,
  "signature": "100c4bf9d50bd2da4af8d65b7b35847b0258d59d62b993311af4ce86049fa5de6712847db7b1a62d217e8c289bdf7b151552fac2404f965383c2c07fc39a5409"
}"#;

    let tx_create_wallet: TxExchange = ::serde_json::from_str(&json).unwrap();
}