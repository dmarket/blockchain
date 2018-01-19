extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::{verify, PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use std::collections::BTreeMap;
use std::cmp;

use service::CurrencyService;
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

#[derive(PartialEq)]
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

    fn process(&self, view: &mut Fork) -> TxStatus {
        let platform_key = CurrencyService::get_platfrom_wallet();
        let mut platform = WalletSchema::map(view, |mut schema| schema.wallet(&platform_key));
        let mut sender = WalletSchema::map(view, |mut schema| schema.wallet(self.offer().sender()));
        let mut recipient =
            WalletSchema::map(view, |mut schema| schema.wallet(self.offer().recipient()));

        let fee_strategy = FeeStrategy::from_u8(self.offer().fee_strategy()).unwrap();
        let fee = self.get_fee(view);

        // helper
        let can_pay_both = |a: u64, b: u64| {
            let min = cmp::min(a, b) as f64;
            let half = (fee.transaction_fee() as f64 / 2.0).ceil();
            half <= min
        };

        // get fee for platform
        let sufficient_funds = match fee_strategy {
            FeeStrategy::Recipient => recipient.balance() >= fee.transaction_fee(),
            FeeStrategy::Sender => sender.balance() >= fee.transaction_fee(),
            FeeStrategy::RecipientAndSender => can_pay_both(recipient.balance(), sender.balance()),
            _ => false,
        };
        // if participant(s) doesn't have enough coins, than fail.
        if !sufficient_funds {
            return TxStatus::Fail;
        }

        // move coins from participant(s) to platform
        match fee_strategy {
            FeeStrategy::Recipient => {
                recipient.decrease(fee.transaction_fee());
                platform.increase(fee.transaction_fee());
            }
            FeeStrategy::Sender => {
                sender.decrease(fee.transaction_fee());
                platform.increase(fee.transaction_fee());
            }
            FeeStrategy::RecipientAndSender => {
                let half = (fee.transaction_fee() as f64 / 2.0).ceil() as u64;
                recipient.decrease(half);
                sender.decrease(half);
                platform.increase(half);
                platform.increase(half);
            }
            _ => return TxStatus::Fail,
        }

        // store changes
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(self.offer().sender(), sender.clone());
            schema
                .wallets()
                .put(self.offer().recipient(), recipient.clone());
            schema.wallets().put(&platform_key, platform.clone());
        });

        // initial point for db rollback, in case if transaction has failed
        view.checkpoint();

        // check if recipient and sender have mentioned assets/coins for exchange
        // fail if not
        let recipient_assets_ok = recipient.is_assets_in_wallet(&self.offer().recipient_assets());
        let sender_assets_ok = sender.is_assets_in_wallet(&self.offer().sender_assets());
        let recipient_value_ok = recipient.balance() >= self.offer().recipient_value();
        let sender_value_ok = sender.balance() >= self.offer().sender_value();

        if !recipient_assets_ok || !sender_assets_ok || !recipient_value_ok || !sender_value_ok {
            view.rollback();
            return TxStatus::Fail;
        }

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

        // store changes
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(self.offer().sender(), sender.clone());
            schema
                .wallets()
                .put(self.offer().recipient(), recipient.clone());
        });

        TxStatus::Success
    }
}

impl Transaction for TxExchange {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        let keys_ok = *self.offer().sender() != *self.offer().recipient();
        // Fee Strategy cannot be intermediary
        let fee_strategy_ok = match FeeStrategy::from_u8(self.offer().fee_strategy()) {
            Some(fee_strategy) => fee_strategy != FeeStrategy::Intermediary,
            None => false,
        };
        let verify_recipient_ok = self.verify_signature(self.offer().recipient());
        let verify_sender_ok = verify(
            self.sender_signature(),
            &self.offer().raw,
            self.offer().sender(),
        );

        keys_ok && fee_strategy_ok && verify_recipient_ok && verify_sender_ok
    }

    fn execute(&self, view: &mut Fork) {
        let tx_status = self.process(view);
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

    pub fn transaction_fee(&self) -> u64 {
        self.transaction_fee
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
