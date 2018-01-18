extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::{verify, PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use std::collections::BTreeMap;

use service::asset::Asset;
use service::wallet::Wallet;
use service::configuration::Configuration;

use super::{SERVICE_ID, TX_EXCHANGE_ID};
use super::schema::asset::AssetSchema;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

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

pub enum FeeStrategy {
    Recipient = 1,
    Sender = 2,
    RecipientAndSender = 3,
    Intermediary = 4,
}

pub struct ExchangeFee {
    transaction_fee: u64,
    assets_fees: BTreeMap<Wallet, u64>,
}

message! {
    struct TxExchange {
        const TYPE = SERVICE_ID;
        const ID = TX_EXCHANGE_ID;
        const SIZE = 88;

        field offer:             ExchangeOffer     [00 => 8]
        field seed:              u64               [8 => 16]
        field sender_signature:  &Signature        [16 => 80]
        field data_info:         &str              [80 => 88]
    }
}

impl TxExchange {
    pub fn get_offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    pub fn get_fee(&self, view: &mut Fork) -> ExchangeFee {
        let exchange_assets = [
            &self.offer().sender_assets()[..],
            &self.offer().recipient_assets()[..],
        ].concat();

        let mut assets_fees = BTreeMap::new();

        let fee_ratio = |count: u32, coef: u64| (count as f64 / coef as f64).round() as u64;
        for asset in exchange_assets {
            if let Some(info) = AssetSchema::map(view, |mut schema| schema.info(&asset.id())) {
                let exchange_fee = info.fees().exchange();
                let fee = exchange_fee.tax() + fee_ratio(asset.amount(), exchange_fee.ratio());

                let creator = WalletSchema::map(view, |mut schema| schema.wallet(info.creator()));
                *assets_fees.entry(creator).or_insert(0) += fee;
            }
        }

        let tx_fee = Configuration::extract(view).fees().exchange();
        ExchangeFee::new(tx_fee, assets_fees)
    }
}

impl Transaction for TxExchange {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        let keys_ok = *self.offer().sender() != *self.offer().recipient();
        let fee_strategy_ok = FeeStrategy::from_u8(self.offer().fee_strategy()).is_some();
        let verify_recipient_ok = self.verify_signature(self.offer().recipient());
        let verify_sender_ok = verify(
            self.sender_signature(),
            &self.offer().raw,
            self.offer().sender(),
        );

        keys_ok && fee_strategy_ok && verify_recipient_ok && verify_sender_ok
    }

    fn execute(&self, view: &mut Fork) {
        let mut tx_status = TxStatus::Fail;
        WalletSchema::map(view, |mut schema| {
            let mut sender = schema.wallet(self.offer().sender());
            let mut recipient = schema.wallet(self.offer().recipient());
            if sender.balance() >= self.offer().sender_value()
                && sender.is_assets_in_wallet(&self.offer().sender_assets())
                && recipient.balance() >= self.offer().recipient_value()
                && recipient.is_assets_in_wallet(&self.offer().recipient_assets())
            {
                println!("--   Exchange transaction   --");
                println!("Sender's balance before transaction : {:?}", sender);
                println!("Recipient's balance before transaction : {:?}", recipient);

                sender.decrease(self.offer().sender_value());
                recipient.increase(self.offer().sender_value());

                sender.increase(self.offer().recipient_value());
                recipient.decrease(self.offer().recipient_value());

                sender.del_assets(&self.offer().sender_assets());
                recipient.add_assets(&self.offer().sender_assets());

                sender.add_assets(&self.offer().recipient_assets());
                recipient.del_assets(&self.offer().recipient_assets());

                println!("Sender's balance before transaction : {:?}", sender);
                println!("Recipient's balance before transaction : {:?}", recipient);
                let mut wallets = schema.wallets();
                wallets.put(self.offer().sender(), sender);
                wallets.put(self.offer().recipient(), recipient);

                tx_status = TxStatus::Success;
            }
        });

        TxStatusSchema::map(view, |mut db| db.set_status(&self.hash(), tx_status))
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
        })
    }
}

impl FeeStrategy {
    fn from_u8(value: u8) -> Option<FeeStrategy> {
        match value {
            1 => Some(FeeStrategy::Recipient),
            2 => Some(FeeStrategy::Sender),
            3 => Some(FeeStrategy::RecipientAndSender),
            4 => Some(FeeStrategy::Intermediary),
            _ => None,
        }
    }
}

impl ExchangeFee {
    pub fn new(tx_fee: u64, fees: BTreeMap<Wallet, u64>) -> Self {
        ExchangeFee {
            transaction_fee: tx_fee,
            assets_fees: fees,
        }
    }

    pub fn amount(&self) -> u64 {
        let mut amount = self.transaction_fee;
        amount += self.assets_fees.iter().fold(0, |acc, asset| acc + asset.1);
        amount
    }

    pub fn assets_fees(&self) -> BTreeMap<Wallet, u64> {
        self.assets_fees.clone()
    }
}
