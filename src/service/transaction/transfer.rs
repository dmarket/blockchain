extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use service::wallet::Asset;

use service::transaction::{PER_ASSET_FEE, TRANSACTION_FEE};

use super::{SERVICE_ID, TX_TRANSFER_ID};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

message! {
    struct TxTransfer {
        const TYPE = SERVICE_ID;
        const ID = TX_TRANSFER_ID;
        const SIZE = 88;

        field from:        &PublicKey  [00 => 32]
        field to:          &PublicKey  [32 => 64]
        field amount:      u64         [64 => 72]
        field assets:      Vec<Asset>  [72 => 80]
        field seed:        u64         [80 => 88]
    }
}

impl TxTransfer {
    fn get_fee(&self) -> u64 {
        TRANSACTION_FEE + PER_ASSET_FEE * Asset::count(&self.assets())
    }
}

impl Transaction for TxTransfer {
    fn verify(&self) -> bool {
        (*self.from() != *self.to()) && self.verify_signature(self.from())
    }

    fn execute(&self, view: &mut Fork) {
        let sender = WalletSchema::map(view, |mut schema| schema.wallet(self.from()));
        let mut tx_status = TxStatus::Fail;
        if let Some(mut sender) = sender {
            let amount = self.amount();
            let update_amount = amount == 0 && sender.balance() >= self.get_fee() ||
                amount > 0 && sender.balance() >= amount + self.get_fee();
            let update_assets = self.assets().is_empty() ||
                !self.assets().is_empty() && sender.in_wallet_assets(&self.assets());
            if update_amount && update_assets {
                sender.decrease(amount + self.get_fee());
                sender.del_assets(&self.assets());
                WalletSchema::map(view, |mut schema| {
                    let mut receiver = schema.create_wallet(self.to());
                    receiver.increase(amount);
                    receiver.add_assets(self.assets());
                    println!("Transfer between wallets: {:?} => {:?}", sender, receiver);
                    schema.wallets().put(self.from(), sender);
                    schema.wallets().put(self.to(), receiver);
                });
                tx_status = TxStatus::Success;
            }
        }
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
use exonum::storage::{Database, MemoryDB};
#[cfg(test)]
use service::wallet::Wallet;


#[cfg(test)]
fn get_json() -> String {
    r#"{
  "body": {
    "from": "739fe1c8507aac54b5d4af116544fec304cf8b0f759d0bce39a7934630c0457e",
    "to": "c08575875170900ac946fc9c0c521bea3d61c138380512cc8d1f55ba27289d27",
    "amount": "3",
    "assets": [
      {
        "hash_id": "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f",
        "amount": 3
      }
    ],
    "seed": "123"
  },
  "network_id": 0,
  "protocol_version": 0,
  "service_id": 2,
  "message_id": 2,
  "signature": "4f9c0a9ddb32a1d8e61d3b656dec5786fb447c19362853ddac67a2c4f48c9ad65a377ee86a02727a27a35d16a14dea84f6920878ab82a6e850e8e7814bb64701"
}"#.to_string()
}

#[test]
fn test_convert_from_json() {
    let tx: TxTransfer = ::serde_json::from_str(&get_json()).unwrap();
    assert!(tx.verify());
    assert_eq!(
        Asset::new("a8d5c97d-9978-4b0b-9947-7a95dcb31d0f",3),
        tx.assets()[0]
    );
    assert_eq!(3, tx.amount());
}

#[test]
fn positive_send_staff_test() {
    let tx_transfer: TxTransfer = ::serde_json::from_str(&get_json()).unwrap();

    let db = Box::new(MemoryDB::new());
    let fork = &mut db.fork();

    let from = Wallet::new(
        tx_transfer.from(),
        2000,
        vec![
            Asset::new("a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", 100),
        ],
    );
    WalletSchema::map(fork, |mut schema| {
        schema.wallets().put(tx_transfer.from(), from);
    });

    tx_transfer.execute(fork);

    let participants = WalletSchema::map(fork, |mut schema| {
        (
            schema.wallet(tx_transfer.from()),
            schema.wallet(tx_transfer.to()),
        )
    });
    if let (Some(from), Some(to)) = participants {
        assert_eq!(994, from.balance());
        assert_eq!(3, to.balance());
        assert_eq!(
            vec![Asset::new("a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", 97), ],
            from.assets()
        );
        assert_eq!(
            vec![
                Asset::new("a8d5c97d-9978-4b0b-9947-7a95dcb31d0f", 3),
            ],
            to.assets()
        );
    } else {
        panic!("Something wrong!!!");
    }
}
#[test]
fn transfer_info_test() {
    let tx: TxTransfer = ::serde_json::from_str(&get_json()).unwrap();
    assert_eq!(tx.get_fee(), tx.info()["tx_fee"]);
}
