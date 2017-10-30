extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::{PublicKey, Signature, verify};
use exonum::messages::Message;
use serde_json::Value;

use super::{SERVICE_ID, TX_EXCHANGE_ID};
use super::wallet::Asset;
use super::schema::wallet::WalletSchema;
use super::schema::transaction_status::{TxStatusSchema, TxStatus};

encoding_struct! {
    struct ExchangeOffer {
        const SIZE = 97;

        field sender:                 &PublicKey   [00 => 32]
        field sender_assets:          Vec<Asset>   [32 => 40]
        field sender_value:           u64          [40 => 48]

        field recipient:              &PublicKey   [48 => 80]
        field recipient_assets:       Vec<Asset>   [80 => 88]
        field recipient_value:        u64          [88 => 96]

        field fee_strategy:           u8           [96 => 97]
    }
}

message! {
    struct TxExchange {
        const TYPE = SERVICE_ID;
        const ID = TX_EXCHANGE_ID;
        const SIZE = 80;

        field offer:             ExchangeOffer     [00 => 8]
        field seed:              u64               [8 => 16]
        field sender_signature:  &Signature        [16 => 80]
    }
}
impl TxExchange {
    pub fn get_offer_raw(&self) -> Vec<u8> {
        self.offer().raw
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
        let mut schema = WalletSchema { view };
        let mut tx_status = TxStatus::Fail;
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
                tx_status = TxStatus::Success;
            }
        }
        let mut tx_status_schema = TxStatusSchema{view: schema.view};
        tx_status_schema.set_status(&self.hash(), tx_status);
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
            "tx_fee": 0,

        })
    }

}


#[test]
fn test_convert_from_json() {
    let json =
        r#"{
  "body": {
    "offer": {
      "sender": "b52ed23433b2eb2377177ea658cd32d73de2641d3acfe9f9a33b7716c0480558",
      "sender_assets": [{"hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f","amount": 5},{"hash_id": "a8d5c97d-9978-4111-9947-7a95dcb31d0f","amount": 7}],
      "sender_value": "37",
      "recipient": "f0198865f6c249dad503abee4a06b59f8bc4f9ff31600fde8cc43b7229ef3207",
      "recipient_assets": [{"hash_id": "a8d5c97d-9978-cccc-9947-7a95dcb31d0f","amount": 1}],
      "recipient_value": "0",
      "fee_strategy": 1
    },
    "seed": "216",
    "sender_signature": "96e934b718d50ca915e07561b36d15aa2c45f00fb1aad887ae33e3b9403c6197d0cc44ffd1f3353b5b9f7ad7bd8b750b49836401b313b1da0957ea833d8e3f03"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 6,
  "signature": "284b1e08e69d7dc622501c66cbc71841e1ae9a13a7a9e77d2411f2c571c2c6afa176510b8404573da922ae5488c91aa1163c7c9669fd2181fa31925825780a03"
}"#;

    let _: TxExchange = ::serde_json::from_str(&json).unwrap();
}
