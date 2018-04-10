use std::collections::HashMap;

use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;
use prometheus::{Counter, Histogram};

use currency::{Service, SERVICE_ID};
use currency::assets::AssetBundle;
use currency::transactions::components::{FeeStrategy, ThirdPartyFees, Intermediary, FeesCalculator};
use currency::error::Error;
use currency::status;
use currency::wallet;
use currency::configuration::Configuration;

/// Transaction ID.
pub const EXCHANGE_INTERMEDIARY_ID: u16 = 602;

encoding_struct! {
    struct ExchangeOfferIntermediary {
        const SIZE = 97;

        field intermediary:     Intermediary     [00 =>  8]

        field sender:           &PublicKey       [08 => 40]
        field sender_assets:    Vec<AssetBundle> [40 => 48]
        field sender_value:     u64              [48 => 56]

        field recipient:        &PublicKey       [56 => 88]
        field recipient_assets: Vec<AssetBundle> [88 => 96]

        field fee_strategy:     u8               [96 => 97]
    }
}

message! {
    /// `exchange_intermediary` transaction.
    struct ExchangeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_INTERMEDIARY_ID;
        const SIZE = 152;

        field offer:                  ExchangeOfferIntermediary [0 => 8]
        field seed:                   u64                       [8 => 16]
        field sender_signature:       &Signature                [16 => 80]
        field intermediary_signature: &Signature                [80 => 144]
        field data_info:              &str                      [144 => 152]
    }
}

impl FeesCalculator for ExchangeIntermediary {
    fn calculate_fees(&self, view: &mut Fork) -> Result<HashMap<PublicKey, u64>, Error> {
        let offer = self.offer();
        let genesis_fee = Configuration::extract(view).fees().exchange();
        let fees = ThirdPartyFees::new_exchange(
            &*view,
            offer
                .sender_assets()
                .into_iter()
                .chain(offer.recipient_assets().into_iter()),
        )?;
        let fee_strategy =
            FeeStrategy::try_from(offer.fee_strategy()).expect("fee strategy must be valid");

        let mut fees_table = HashMap::new();

        let payers = self.payers(&fee_strategy, genesis_fee)?;
        for (payer_key, fee) in payers {
            if Service::genesis_wallet() != payer_key {
                fees_table.insert(payer_key, fee);
            }
        }

        for (receiver_key, fee) in fees.0 {
            let payers = self.payers(&fee_strategy, fee)?;
            
            for (payer_key, fee) in payers {
                if payer_key != receiver_key {
                    *fees_table.entry(payer_key).or_insert(0) += fee;
                }
            }
        }

        Ok(fees_table)
    }
}

impl ExchangeIntermediary {
    fn payers(&self, fee_strategy: &FeeStrategy, fee: u64) -> Result<Vec<(PublicKey, u64)>, Error> {
        let offer = self.offer();
        let payers = match *fee_strategy {
            FeeStrategy::Recipient => vec![(*offer.recipient(), fee)],
            FeeStrategy::Sender => vec![(*offer.sender(), fee)],
            FeeStrategy::RecipientAndSender => vec![(*offer.sender(), fee/2), 
                                                    (*offer.recipient(), fee/2)],
            FeeStrategy::Intermediary => vec![(*offer.intermediary().wallet(), fee)],
        };
        Ok(payers)
    }

    /// Get raw bytes of the offer.
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);

        let genesis_fee = Configuration::extract(view).fees().exchange();

        let offer = self.offer();

        let fee_strategy =
            FeeStrategy::try_from(offer.fee_strategy()).expect("fee strategy must be valid");

        let mut genesis = wallet::Schema(&*view).fetch(&Service::genesis_wallet());

        // Collect the blockchain fee. Execution shall not continue if this fails.
        match fee_strategy {
            FeeStrategy::Recipient => {
                let mut recipient = wallet::Schema(&*view).fetch(offer.recipient());

                wallet::move_coins(&mut recipient, &mut genesis, genesis_fee)?;

                wallet::Schema(&mut *view).store(offer.recipient(), recipient);
            }
            FeeStrategy::Sender => {
                let mut sender = wallet::Schema(&*view).fetch(offer.sender());

                wallet::move_coins(&mut sender, &mut genesis, genesis_fee)?;

                wallet::Schema(&mut *view).store(offer.sender(), sender);
            }
            FeeStrategy::RecipientAndSender => {
                let mut recipient = wallet::Schema(&*view).fetch(offer.recipient());
                let mut sender = wallet::Schema(&*view).fetch(offer.sender());

                wallet::move_coins(&mut recipient, &mut genesis, genesis_fee / 2)?;
                wallet::move_coins(&mut sender, &mut genesis, genesis_fee / 2)?;

                wallet::Schema(&mut *view).store(offer.sender(), sender);
                wallet::Schema(&mut *view).store(offer.recipient(), recipient);
            }
            FeeStrategy::Intermediary => {
                let mut intermediary = wallet::Schema(&*view).fetch(offer.intermediary().wallet());

                wallet::move_coins(&mut intermediary, &mut genesis, genesis_fee)?;

                wallet::Schema(&mut *view).store(offer.intermediary().wallet(), intermediary);
            }
        }

        wallet::Schema(&mut *view).store(&Service::genesis_wallet(), genesis);

        let mut fees = ThirdPartyFees::new_exchange(
            &*view,
            offer
                .sender_assets()
                .into_iter()
                .chain(offer.recipient_assets().into_iter()),
        )?;

        // Insert intermediary as one of third party fees.
        fees.add_fee(
            offer.intermediary().wallet(),
            offer.intermediary().commission(),
        );

        // Operations bellow must either all succeed, or return an error without
        // saving anything to the database.

        // Process third party fees.
        let mut updated_wallets = match fee_strategy {
            FeeStrategy::Recipient => fees.collect(view, offer.recipient())?,
            FeeStrategy::Sender => fees.collect(view, offer.sender())?,
            FeeStrategy::RecipientAndSender => {
                fees.collect2(view, offer.sender(), offer.recipient())?
            }
            FeeStrategy::Intermediary => {
                fees.collect(view, offer.intermediary().wallet())?
            }
        };

        // Process the main transaction.
        let mut sender = updated_wallets
            .remove(&offer.sender())
            .unwrap_or_else(|| wallet::Schema(&*view).fetch(&offer.sender()));
        let mut recipient = updated_wallets
            .remove(&offer.recipient())
            .unwrap_or_else(|| wallet::Schema(&*view).fetch(&offer.recipient()));

        wallet::move_coins(&mut sender, &mut recipient, offer.sender_value())?;
        wallet::move_assets(&mut sender, &mut recipient, &offer.sender_assets())?;
        wallet::move_assets(&mut recipient, &mut sender, &offer.recipient_assets())?;

        updated_wallets.insert(*offer.sender(), sender);
        updated_wallets.insert(*offer.recipient(), recipient);

        // Save changes to the database.
        for (key, wallet) in updated_wallets {
            wallet::Schema(&mut *view).store(&key, wallet);
        }

        Ok(())
    }
}

lazy_static! {
    static ref VERIFY_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_intermediary_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_intermediary_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_intermediary_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_intermediary_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: Counter = register_counter!(
        "dmbc_transaction_exchange_intermediary_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
    static ref EXECUTE_DURATION: Histogram = register_histogram!(
        "dmbc_transaction_exchange_intermediary_execute_duration_seconds",
        "Duration of transaction execution."
    ).unwrap();
}

impl Transaction for ExchangeIntermediary {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

        let offer = self.offer();

        let wallets_ok = offer.sender() != offer.recipient()
            && offer.intermediary().wallet() != offer.sender()
            && offer.intermediary().wallet() != offer.recipient();
        let fee_strategy_ok = FeeStrategy::try_from(offer.fee_strategy()).is_some();

        if cfg!(fuzzing) {
            return wallets_ok && fee_strategy_ok;
        }

        let recipient_ok = self.verify_signature(offer.recipient());
        let sender_ok = crypto::verify(self.sender_signature(), &offer.raw, offer.sender());
        let intermediary_ok = crypto::verify(
            self.intermediary_signature(),
            &offer.raw,
            offer.intermediary().wallet(),
        );

        if wallets_ok && fee_strategy_ok && recipient_ok && sender_ok && intermediary_ok {
            VERIFY_SUCCESS_COUNT.inc();
            true
        } else {
            false
        }
    }

    fn execute(&self, view: &mut Fork) {
        EXECUTE_COUNT.inc();
        let timer = EXECUTE_DURATION.start_timer();

        let result = self.process(view);

        if let &Ok(_) = &result {
            EXECUTE_SUCCESS_COUNT.inc();
        }

        status::Schema(view).store(self.hash(), result);

        timer.observe_duration();
        EXECUTE_FINISH_COUNT.inc();
    }

    fn info(&self) -> serde_json::Value {
        json!(self)
    }
}

