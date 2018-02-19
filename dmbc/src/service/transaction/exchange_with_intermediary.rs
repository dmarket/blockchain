extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto::{verify, PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::CurrencyService;
use service::asset::Asset;
use service::wallet::Wallet;
use service::transaction::intermediary::Intermediary;
use service::transaction::fee::{FeeStrategy, TxFees};
use service::schema::wallet::WalletSchema;

use super::SERVICE_ID;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};

pub const TX_EXCHANGE_WITH_INTERMEDIARY_ID: u16 = 602;

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

        TxFees::for_exchange(view, exchange_assets)
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        let mut platform =
            WalletSchema::get_wallet(view, &CurrencyService::genesis_wallet_pub_key());
        let mut sender = WalletSchema::get_wallet(view, self.offer().sender());
        let mut recipient = WalletSchema::get_wallet(view, self.offer().recipient());
        let mut intermediary = WalletSchema::get_wallet(view, self.offer().intermediary().wallet());

        let fee_strategy = FeeStrategy::from_u8(self.offer().fee_strategy()).unwrap();
        let fee = self.get_fee(view);

        // Pay fee for tx execution
        if move_coins(
            view,
            &fee_strategy,
            &mut recipient,
            &mut sender,
            &mut intermediary,
            &mut platform,
            fee.transaction_fee(),
        ).is_err()
        {
            return TxStatus::Fail;
        }

        // initial point for db rollback, in case if transaction has failed

//        view.commit();
//        view.checkpoint();

        // pay commison for the transaction to intermediary
        if pay_commision(
            view,
            &fee_strategy,
            &mut recipient,
            &mut sender,
            &mut intermediary,
            self.offer().intermediary().commission(),
        ).is_err()
        {
//            view.rollback();
            return TxStatus::Fail;
        }

        println!("--   Exchange transaction   --");
        println!("Sender's balance before transaction : {:?}", sender);
        println!("Recipient's balance before transaction : {:?}", recipient);

        // send fee to creators of assets
        for (mut creator, fee) in fee.assets_fees() {
            println!("\tCreator {:?} will receive {}", creator.pub_key(), fee);
            if move_coins(
                view,
                &fee_strategy,
                &mut recipient,
                &mut sender,
                &mut intermediary,
                &mut creator,
                fee,
            ).is_err()
            {
//                view.rollback();
                return TxStatus::Fail;
            }
        }

        if WalletSchema::transfer_coins(
            view,
            &mut sender,
            &mut recipient,
            self.offer().sender_value(),
        ).is_err()
        {
//            view.rollback();
            return TxStatus::Fail;
        }

        if WalletSchema::exchange_assets(
            view,
            &mut sender,
            &mut recipient,
            &self.offer().sender_assets(),
            &self.offer().recipient_assets(),
        ).is_err()
        {
//            view.rollback();
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
    intermediary: &mut Wallet,
    coins_receiver: &mut Wallet,
    coins: u64,
) -> Result<(), ()> {
    // move coins from participant(s) to fee receiver
    match *strategy {
        FeeStrategy::Recipient => {
            WalletSchema::transfer_coins(view, recipient, coins_receiver, coins)
        }
        FeeStrategy::Sender => WalletSchema::transfer_coins(view, sender, coins_receiver, coins),
        FeeStrategy::RecipientAndSender => {
            let (recipient_half, sender_half) = split_coins(coins);
            let recipient_result =
                WalletSchema::transfer_coins(view, recipient, coins_receiver, recipient_half);
            let sender_result =
                WalletSchema::transfer_coins(view, sender, coins_receiver, sender_half);

            if recipient_result.is_ok() && sender_result.is_ok() {
                Ok(())
            } else {
                Err(())
            }
        }
        FeeStrategy::Intermediary => {
            WalletSchema::transfer_coins(view, intermediary, coins_receiver, coins)
        }
    }
}

fn pay_commision(
    view: &mut Fork,
    strategy: &FeeStrategy,
    recipient: &mut Wallet,
    sender: &mut Wallet,
    intermediary: &mut Wallet,
    commision: u64,
) -> Result<(), ()> {
    if *strategy != FeeStrategy::Intermediary {
        return Err(());
    }

    match *strategy {
        FeeStrategy::Recipient => {
            return WalletSchema::transfer_coins(view, recipient, intermediary, commision);
        }
        FeeStrategy::Sender => {
            return WalletSchema::transfer_coins(view, sender, intermediary, commision);
        }
        FeeStrategy::RecipientAndSender => {
            let (recipient_half, sender_half) = split_coins(commision);
            let recipient_result =
                WalletSchema::transfer_coins(view, recipient, intermediary, recipient_half);
            let sender_result =
                WalletSchema::transfer_coins(view, sender, intermediary, sender_half);
            if recipient_result.is_ok() && sender_result.is_ok() {
                Ok(())
            } else {
                Err(())
            }
        }
        FeeStrategy::Intermediary => Ok(()),
    }
}
