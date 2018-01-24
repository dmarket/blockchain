extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::{verify, PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::CurrencyService;
use service::asset::Asset;
use service::wallet::Wallet;
use service::transaction::utils;
use service::transaction::utils::Intermediary;
use service::transaction::fee::{calculate_fees_for_exchange, FeeStrategy, TxFees};

use super::{SERVICE_ID, TX_EXCHANGE_WITH_INTERMEDIARY_ID};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

encoding_struct! {
    struct ExchangeOfferWithIntermediary {
        const SIZE = 97;

        field intermediary:           Intermediary [00 => 8]

        field sender:                 &PublicKey   [08 => 40]
        field sender_assets:          Vec<Asset>   [40 => 48]
        field sender_value:           u64          [48 => 56]

        field recipient:              &PublicKey   [56 => 88]
        field recipient_assets:       Vec<Asset>   [88 => 96]

        field fee_strategy:           u8           [96 => 97]
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

    pub fn get_fee(&self, view: &mut Fork) -> TxFees {
        let exchange_assets = [
            &self.offer().sender_assets()[..],
            &self.offer().recipient_assets()[..],
        ].concat();

        calculate_fees_for_exchange(view, exchange_assets)
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
        if !pay(
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

        if !utils::transfer_coins(
            view,
            &mut sender,
            &mut recipient,
            self.offer().sender_value(),
        ) {
            view.rollback();
            return TxStatus::Fail;
        }

        if !utils::exchange_assets(
            view,
            &mut sender,
            &mut recipient,
            &self.offer().sender_assets(),
            &self.offer().recipient_assets(),
        ) {
            view.rollback();
            return TxStatus::Fail;
        }

        println!("Sender's balance before transaction : {:?}", sender);
        println!("Recipient's balance before transaction : {:?}", recipient);

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
    // move coins from participant(s) to fee receiver
    match *strategy {
        FeeStrategy::Recipient => utils::transfer_coins(view, recipient, coins_receiver, coins),
        FeeStrategy::Sender => utils::transfer_coins(view, sender, coins_receiver, coins),
        FeeStrategy::RecipientAndSender => {
            let (recipient_half, sender_half) = utils::split_coins(coins);
            let recipient_ok =
                utils::transfer_coins(view, recipient, coins_receiver, recipient_half);
            let sender_ok = utils::transfer_coins(view, sender, coins_receiver, sender_half);

            sender_ok && recipient_ok
        }
        FeeStrategy::Intermediary => {
            utils::transfer_coins(view, intermediary, coins_receiver, coins)
        }
    }
}

fn pay(
    view: &mut Fork,
    strategy: &FeeStrategy,
    recipient: &mut Wallet,
    sender: &mut Wallet,
    intermediary: &mut Wallet,
    commision: u64,
) -> bool {
    if *strategy != FeeStrategy::Intermediary {
        return false;
    }

    match *strategy {
        FeeStrategy::Recipient => {
            return utils::transfer_coins(view, recipient, intermediary, commision);
        }
        FeeStrategy::Sender => {
            return utils::transfer_coins(view, sender, intermediary, commision);
        }
        FeeStrategy::RecipientAndSender => {
            let (recipient_half, sender_half) = utils::split_coins(commision);
            let recipient_ok = utils::transfer_coins(view, recipient, intermediary, recipient_half);
            let sender_ok = utils::transfer_coins(view, sender, intermediary, sender_half);
            return recipient_ok && sender_ok;
        }
        FeeStrategy::Intermediary => true,
    }
}
