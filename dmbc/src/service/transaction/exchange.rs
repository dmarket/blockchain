extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::{verify, PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use std::cmp;

use service::CurrencyService;
use service::asset::Asset;
use service::wallet::Wallet;
use service::transaction::fee::{calculate_fee_for_exchange, FeeStrategy, TradeExchangeFee};

use super::{SERVICE_ID, TX_EXCHANGE_ID};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

encoding_struct! {
    struct ExchangeOffer {
        const SIZE = 89;

        field sender:                 &PublicKey   [00 => 32]
        field sender_assets:          Vec<Asset>   [32 => 40]
        field sender_value:           u64          [40 => 48]

        field recipient:              &PublicKey   [48 => 80]
        field recipient_assets:       Vec<Asset>   [80 => 88]

        field fee_strategy:           u8           [88 => 89]
    }
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

    pub fn get_fee(&self, view: &mut Fork) -> TradeExchangeFee {
        let exchange_assets = [
            &self.offer().sender_assets()[..],
            &self.offer().recipient_assets()[..],
        ].concat();

        calculate_fee_for_exchange(view, exchange_assets)
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        let (mut platform, mut sender, mut recipient) = WalletSchema::map(view, |mut schema| {
            let platform_key = CurrencyService::get_platfrom_wallet();
            (
                schema.wallet(&platform_key),
                schema.wallet(self.offer().sender()),
                schema.wallet(self.offer().recipient()),
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
            &mut platform,
            fee.transaction_fee(),
        ) {
            return TxStatus::Fail;
        }

        // initial point for db rollback, in case if transaction has failed
        view.checkpoint();

        // check if recipient and sender have mentioned assets/coins for exchange
        // fail if not
        let recipient_assets_ok = recipient.is_assets_in_wallet(&self.offer().recipient_assets());
        let sender_assets_ok = sender.is_assets_in_wallet(&self.offer().sender_assets());
        let sender_value_ok = sender.balance() >= self.offer().sender_value();

        if !recipient_assets_ok || !sender_assets_ok || !sender_value_ok {
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
                &mut creator,
                fee,
            ) {
                view.rollback();
                return TxStatus::Fail;
            }
        }

        sender.decrease(self.offer().sender_value());
        recipient.increase(self.offer().sender_value());

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
    pub fn from_u8(value: u8) -> Option<FeeStrategy> {
        match value {
            1 => Some(FeeStrategy::Recipient),
            2 => Some(FeeStrategy::Sender),
            3 => Some(FeeStrategy::RecipientAndSender),
            4 => Some(FeeStrategy::Intermediary),
            _ => None,
        }
    }
}

fn split_coins(coins: u64) -> (u64, u64) {
    let first_half = (coins as f64 / 2.0).ceil() as u64;
    let second_half = coins - first_half;
    (first_half, second_half)
}

fn move_coins(
    view: &mut Fork,
    strategy: &FeeStrategy,
    recipient: &mut Wallet,
    sender: &mut Wallet,
    coins_receiver: &mut Wallet,
    coins: u64,
) -> bool {
    // check if participant(s) have enough coins
    if !sufficient_funds(strategy, recipient, sender, coins) {
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
            let (recipient_half, sender_half) = split_coins(coins);
            recipient.decrease(recipient_half);
            sender.decrease(sender_half);
            coins_receiver.increase(recipient_half);
            coins_receiver.increase(sender_half);
        }
        _ => return false,
    }

    // store changes
    WalletSchema::map(view, |mut schema| {
        schema.wallets().put(recipient.pub_key(), recipient.clone());
        schema.wallets().put(sender.pub_key(), sender.clone());
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
    coins: u64,
) -> bool {
    // helper
    let can_pay_both = |a: u64, b: u64| {
        let min = cmp::min(a, b);
        let (a_half, b_half) = split_coins(coins);
        a_half <= min && b_half <= min
    };

    // check if participant(s) have enough coins to pay platform
    match *strategy {
        FeeStrategy::Recipient => recipient.balance() >= coins,
        FeeStrategy::Sender => sender.balance() >= coins,
        FeeStrategy::RecipientAndSender => can_pay_both(recipient.balance(), sender.balance()),
        _ => false,
    }
}
