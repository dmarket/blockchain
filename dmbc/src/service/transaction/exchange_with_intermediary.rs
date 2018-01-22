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
use service::transaction::exchange::FeeStrategy;
use service::transaction::exchange::ExchangeFee;

use super::{SERVICE_ID, TX_EXCHANGE_WITH_INTERMEDIARY_ID};
use super::schema::asset::AssetSchema;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

encoding_struct! {
    struct Intermediary {
        const SIZE = 40;

        field wallet:       &PublicKey [0 => 32]
        field commision:    u64        [32 => 40]
    }
}

encoding_struct! {
    struct ExchangeOfferWithIntermediary {
        const SIZE = 105;

        field intermediary:           Intermediary [00 => 8]

        field sender:                 &PublicKey   [08 => 40]
        field sender_assets:          Vec<Asset>   [40 => 48]
        field sender_value:           u64          [48 => 56]

        field recipient:              &PublicKey   [56 => 88]
        field recipient_assets:       Vec<Asset>   [88 => 96]
        field recipient_value:        u64          [96 => 104]

        field fee_strategy:           u8           [104 => 105]
    }
}

message! {
    struct TxExchangeWithIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TX_EXCHANGE_WITH_INTERMEDIARY_ID;
        const SIZE = 152;

        field offer:                  ExchangeOfferWithIntermediary [0 => 8]
        field seed:                   u64                           [8 => 16]
        field sender_signature:       &Signature                    [16 => 80]
        field intermediary_signature: &Signature                    [80 => 144]
        field data_info:              &str                          [144 => 152]
    }
}

impl TxExchangeWithIntermediary {
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
        let (mut platform, mut sender, mut recipient, mut intermediary) =
            WalletSchema::map(view, |mut schema| {
                let platform_key = CurrencyService::get_platfrom_wallet();
                (
                    schema.wallet(&platform_key),
                    schema.wallet(self.offer().sender()),
                    schema.wallet(self.offer().recipient()),
                    schema.wallet(self.offer().intermediary().wallet()),
                )
            });

        let fee_strategy = FeeStrategy::from_u8(self.offer().fee_strategy()).unwrap();
        let fee = self.get_fee(view);

        // move coins from participant(s) to platform
        if !move_coins(
            view,
            &fee_strategy,
            &mut recipient,
            &mut sender,
            &mut intermediary,
            &mut platform,
            fee.transaction_fee(),
        ) {
            return TxStatus::Fail;
        }

        // initial point for db rollback, in case if transaction has failed
        view.checkpoint();

        // pay commison for the transaction to intermediary
        if !pay_commision(
            view,
            &fee_strategy,
            &mut recipient,
            &mut sender,
            &mut intermediary,
            self.offer().intermediary().commision(),
        ) {
            view.rollback();
            return TxStatus::Fail;
        }

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

        // send fee to creators of assets
        for (mut creator, fee) in fee.assets_fees() {
            println!("\tCreator {:?} will receive {}", creator.pub_key(), fee);
            if !move_coins(
                view,
                &fee_strategy,
                &mut recipient,
                &mut sender,
                &mut intermediary,
                &mut creator,
                fee,
            ) {
                view.rollback();
                return TxStatus::Fail;
            }
        }

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
            schema.wallets().put(sender.pub_key(), sender.clone());
            schema.wallets().put(recipient.pub_key(), recipient.clone());
        });

        TxStatus::Success
    }
}

impl Transaction for TxExchangeWithIntermediary {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        let mut keys_ok = *self.offer().sender() != *self.offer().recipient();
        keys_ok &= *self.offer().sender() != *self.offer().intermediary().wallet();
        keys_ok &= *self.offer().recipient() != *self.offer().intermediary().wallet();

        let fee_strategy_ok = FeeStrategy::from_u8(self.offer().fee_strategy()).is_some();

        let verify_recipient_ok = self.verify_signature(self.offer().recipient());
        let verify_sender_ok = verify(
            self.sender_signature(),
            &self.offer().raw,
            self.offer().sender(),
        );

        let verify_intermediary_ok = verify(
            self.intermediary_signature(),
            &self.offer().raw,
            self.offer().intermediary().wallet(),
        );

        keys_ok && fee_strategy_ok && verify_recipient_ok && verify_sender_ok
            && verify_intermediary_ok
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

fn move_coins(
    view: &mut Fork,
    strategy: &FeeStrategy,
    recipient: &mut Wallet,
    sender: &mut Wallet,
    intermediary: &mut Wallet,
    coins_receiver: &mut Wallet,
    coins: u64,
) -> bool {
    // check if participant(s) have enough coins to pay fee
    if !sufficient_funds(strategy, recipient, sender, intermediary, coins) {
        return false;
    }
    // move coins from participant(s) to fee receiver
    match *strategy {
        FeeStrategy::Recipient => {
            recipient.decrease(coins);
            coins_receiver.increase(coins);
        }
        FeeStrategy::Sender => {
            sender.decrease(coins);
            coins_receiver.increase(coins);
        }
        FeeStrategy::RecipientAndSender => {
            let half = (coins as f64 / 2.0).ceil() as u64;
            recipient.decrease(half);
            sender.decrease(half);
            coins_receiver.increase(half);
            coins_receiver.increase(half);
        }
        FeeStrategy::Intermediary => {
            intermediary.decrease(coins);
            coins_receiver.increase(coins);
        }
    }

    // store changes
    WalletSchema::map(view, |mut schema| {
        schema.wallets().put(recipient.pub_key(), recipient.clone());
        schema.wallets().put(sender.pub_key(), sender.clone());
        schema
            .wallets()
            .put(intermediary.pub_key(), intermediary.clone());
        schema
            .wallets()
            .put(coins_receiver.pub_key(), coins_receiver.clone());
    });
    true
}

fn sufficient_funds(
    strategy: &FeeStrategy,
    recipient: &Wallet,
    sender: &Wallet,
    intermediary: &Wallet,
    coins: u64,
) -> bool {
    // helper
    let can_pay_both = |a: u64, b: u64| {
        let min = cmp::min(a, b) as f64;
        let half = (coins as f64 / 2.0).ceil();
        half <= min
    };

    // check if participant(s) have enough coins to pay platform fee
    match *strategy {
        FeeStrategy::Recipient => recipient.balance() >= coins,
        FeeStrategy::Sender => sender.balance() >= coins,
        FeeStrategy::RecipientAndSender => can_pay_both(recipient.balance(), sender.balance()),
        FeeStrategy::Intermediary => intermediary.balance() >= coins,
    }
}

fn pay_commision(
    view: &mut Fork,
    strategy: &FeeStrategy,
    recipient: &mut Wallet,
    sender: &mut Wallet,
    intermediary: &mut Wallet,
    commision: u64,
) -> bool {
    let is_sufficient_funds =
        sufficient_funds(strategy, recipient, sender, intermediary, commision);
    if *strategy != FeeStrategy::Intermediary && !is_sufficient_funds {
        return false;
    }

    match *strategy {
        FeeStrategy::Recipient => {
            recipient.decrease(commision);
            intermediary.increase(commision);
        }
        FeeStrategy::Sender => {
            sender.decrease(commision);
            intermediary.increase(commision);
        }
        FeeStrategy::RecipientAndSender => {
            let half = (commision as f64 / 2.0).ceil() as u64;
            recipient.decrease(half);
            sender.decrease(half);
            intermediary.increase(half);
            intermediary.increase(half);
        }
        FeeStrategy::Intermediary => (),
    }

    // store changes
    WalletSchema::map(view, |mut schema| {
        schema.wallets().put(recipient.pub_key(), recipient.clone());
        schema.wallets().put(sender.pub_key(), sender.clone());
        schema
            .wallets()
            .put(intermediary.pub_key(), intermediary.clone());
    });
    true
}
