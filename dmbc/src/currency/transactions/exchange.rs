use std::collections::HashMap;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use prometheus::{IntCounter, Histogram};

use currency::assets::AssetBundle;
use currency::error::Error;
use currency::status;
use currency::transactions::components::{FeeStrategy, FeesCalculator, ThirdPartyFees};
use currency::wallet;
use currency::SERVICE_ID;
use currency::service::CONFIGURATION;

/// Transaction ID.
pub const EXCHANGE_ID: u16 = 601;

encoding_struct! {
    struct ExchangeOffer {
        sender:           &PublicKey,
        sender_assets:    Vec<AssetBundle>,
        sender_value:     u64,

        recipient:        &PublicKey,
        recipient_assets: Vec<AssetBundle>,

        fee_strategy:     u8,
    }
}

message! {
    /// `exchange` transaction.
    struct Exchange {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_ID;

        offer:             ExchangeOffer,
        seed:              u64,
        sender_signature:  &Signature,
        data_info:         &str,
    }
}

impl FeesCalculator for Exchange {
    fn calculate_fees(&self, view: &mut Fork) -> Result<HashMap<PublicKey, u64>, Error> {
        let offer = self.offer();
        let genesis_fees = CONFIGURATION.lock().unwrap().fees();
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

        let payers = self.payers(&fee_strategy, genesis_fees.exchange())?;
        for (payer_key, fee) in payers {
            if genesis_fees.recipient() != &payer_key {
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

impl Exchange {
    fn payers(&self, fee_strategy: &FeeStrategy, fee: u64) -> Result<Vec<(PublicKey, u64)>, Error> {
        let offer = self.offer();
        let payers = match *fee_strategy {
            FeeStrategy::Recipient => vec![(*offer.recipient(), fee)],
            FeeStrategy::Sender => vec![(*offer.sender(), fee)],
            FeeStrategy::RecipientAndSender => {
                vec![(*offer.sender(), fee / 2), (*offer.recipient(), fee / 2)]
            }
            FeeStrategy::Intermediary => return Err(Error::InvalidTransaction),
        };
        Ok(payers)
    }

    /// Get raw bytes of the offer.
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);

        let genesis_fees = CONFIGURATION.lock().unwrap().fees();

        let offer = self.offer();

        let fee_strategy =
            FeeStrategy::try_from(offer.fee_strategy()).expect("fee strategy must be valid");

        let mut genesis = wallet::Schema(&*view).fetch(genesis_fees.recipient());

        // Collect the blockchain fee. Execution shall not continue if this fails.
        match fee_strategy {
            FeeStrategy::Recipient => {
                let mut recipient = wallet::Schema(&*view).fetch(offer.recipient());

                wallet::move_coins(&mut recipient, &mut genesis, genesis_fees.exchange())?;

                wallet::Schema(&mut *view).store(offer.recipient(), recipient);
            }
            FeeStrategy::Sender => {
                let mut sender = wallet::Schema(&*view).fetch(offer.sender());

                wallet::move_coins(&mut sender, &mut genesis, genesis_fees.exchange())?;

                wallet::Schema(&mut *view).store(offer.sender(), sender);
            }
            FeeStrategy::RecipientAndSender => {
                let mut recipient = wallet::Schema(&*view).fetch(offer.recipient());
                let mut sender = wallet::Schema(&*view).fetch(offer.sender());

                wallet::move_coins(&mut recipient, &mut genesis, genesis_fees.exchange() / 2)?;
                wallet::move_coins(&mut sender, &mut genesis, genesis_fees.exchange() / 2)?;

                wallet::Schema(&mut *view).store(offer.sender(), sender);
                wallet::Schema(&mut *view).store(offer.recipient(), recipient);
            }
            FeeStrategy::Intermediary => return Err(Error::InvalidTransaction),
        }

        wallet::Schema(&mut *view).store(genesis_fees.recipient(), genesis);

        let fees = ThirdPartyFees::new_exchange(
            &*view,
            offer
                .sender_assets()
                .into_iter()
                .chain(offer.recipient_assets().into_iter()),
        )?;

        // Operations bellow must either all succeed, or return an error without
        // saving anything to the database.

        // Process third party fees.
        let mut updated_wallets = match fee_strategy {
            FeeStrategy::Recipient => fees.collect(view, offer.recipient())?,
            FeeStrategy::Sender => fees.collect(view, offer.sender())?,
            FeeStrategy::RecipientAndSender => {
                fees.collect2(view, offer.sender(), offer.recipient())?
            }
            FeeStrategy::Intermediary => unreachable!(),
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
    static ref VERIFY_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_exchange_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_exchange_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_exchange_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_exchange_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_exchange_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
    static ref EXECUTE_DURATION: Histogram = register_histogram!(
        "dmbc_transaction_exchange_execute_duration_seconds",
        "Duration of transaction execution."
    ).unwrap();
}

impl Transaction for Exchange {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

        let offer = self.offer();

        let wallets_ok = offer.sender() != offer.recipient();
        let fee_strategy_ok = match FeeStrategy::try_from(offer.fee_strategy()).unwrap() {
            FeeStrategy::Recipient | FeeStrategy::Sender | FeeStrategy::RecipientAndSender => true,
            _ => false,
        };

        if cfg!(fuzzing) {
            return wallets_ok && fee_strategy_ok;
        }

        let recipient_ok = self.verify_signature(offer.recipient());
        let sender_ok = crypto::verify(self.sender_signature(), &offer.raw, offer.sender());

        if wallets_ok && fee_strategy_ok && recipient_ok && sender_ok {
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
}
