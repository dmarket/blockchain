use std::collections::HashMap;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use prometheus::{IntCounter, Histogram};
use serde_json;

use currency::assets::TradeAsset;
use currency::configuration::Configuration;
use currency::error::Error;
use currency::status;
use currency::transactions::components::{FeeStrategy, FeesCalculator, ThirdPartyFees};
use currency::wallet;
use currency::SERVICE_ID;

/// Transaction ID.
pub const TRADE_ID: u16 = 501;

encoding_struct! {
    struct TradeOffer {
        const SIZE = 73;

        field buyer: &PublicKey         [00 => 32]
        field seller: &PublicKey        [32 => 64]
        field assets: Vec<TradeAsset>   [64 => 72]

        field fee_strategy: u8          [72 => 73]
    }
}

message! {
    /// `trade` transaction.
    struct Trade {
        const TYPE = SERVICE_ID;
        const ID = TRADE_ID;
        const SIZE = 80;

        field offer:              TradeOffer    [00 => 8]
        field seed:               u64           [8 => 16]
        field seller_signature:   &Signature    [16 => 80]
    }
}

impl FeesCalculator for Trade {
    fn calculate_fees(&self, view: &mut Fork) -> Result<HashMap<PublicKey, u64>, Error> {
        let offer = self.offer();
        let genesis_fees = Configuration::extract(view).fees();
        let fees = ThirdPartyFees::new_trade(&*view, &offer.assets())?;
        let fee_strategy =
            FeeStrategy::try_from(offer.fee_strategy()).expect("fee strategy must be valid");

        let mut fees_table = HashMap::new();

        let payers = self.payers(&fee_strategy, genesis_fees.trade())?;
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

impl Trade {
    fn payers(&self, fee_strategy: &FeeStrategy, fee: u64) -> Result<Vec<(PublicKey, u64)>, Error> {
        let offer = self.offer();
        let payers = match *fee_strategy {
            FeeStrategy::Recipient => vec![(*offer.buyer(), fee)],
            FeeStrategy::Sender => vec![(*offer.seller(), fee)],
            FeeStrategy::RecipientAndSender => {
                vec![(*offer.seller(), fee / 2), (*offer.buyer(), fee / 2)]
            }
            FeeStrategy::Intermediary => return Err(Error::InvalidTransaction),
        };
        Ok(payers)
    }

    /// Raw bytes of the offer.
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);

        let genesis_fees = Configuration::extract(view).fees();

        let offer = self.offer();
        let fee_strategy =
            FeeStrategy::try_from(offer.fee_strategy()).expect("fee strategy must be valid");

        let mut genesis = wallet::Schema(&*view).fetch(genesis_fees.recipient());
        // Collect the blockchain fee. Execution shall not continue if this fails.
        match fee_strategy {
            FeeStrategy::Recipient => {
                let mut buyer = wallet::Schema(&*view).fetch(offer.buyer());

                wallet::move_coins(&mut buyer, &mut genesis, genesis_fees.trade())?;

                wallet::Schema(&mut *view).store(offer.buyer(), buyer);
            }
            FeeStrategy::Sender => {
                let mut seller = wallet::Schema(&*view).fetch(offer.seller());

                wallet::move_coins(&mut seller, &mut genesis, genesis_fees.trade())?;

                wallet::Schema(&mut *view).store(offer.seller(), seller);
            }
            FeeStrategy::RecipientAndSender => {
                let mut buyer = wallet::Schema(&*view).fetch(offer.buyer());
                let mut seller = wallet::Schema(&*view).fetch(offer.seller());

                wallet::move_coins(&mut seller, &mut genesis, genesis_fees.trade() / 2)?;
                wallet::move_coins(&mut buyer, &mut genesis, genesis_fees.trade() / 2)?;

                wallet::Schema(&mut *view).store(offer.seller(), seller);
                wallet::Schema(&mut *view).store(offer.buyer(), buyer);
            }
            FeeStrategy::Intermediary => return Err(Error::InvalidTransaction),
        }

        wallet::Schema(&mut *view).store(genesis_fees.recipient(), genesis);

        let fees = ThirdPartyFees::new_trade(&*view, &offer.assets())?;

        let mut wallet_buyer = wallet::Schema(&*view).fetch(offer.buyer());
        let mut wallet_seller = wallet::Schema(&*view).fetch(offer.seller());

        let total = offer
            .assets()
            .iter()
            .map(|asset| asset.amount() * asset.price())
            .sum();

        wallet::move_coins(&mut wallet_buyer, &mut wallet_seller, total)
            .or_else(|e| {
                wallet::Schema(&mut *view).store(&offer.seller(), wallet_seller.clone());
                wallet::Schema(&mut *view).store(&offer.buyer(), wallet_buyer.clone());

                Err(e)
            })
            .and_then(|_| {
                wallet::Schema(&mut *view).store(&offer.seller(), wallet_seller);
                wallet::Schema(&mut *view).store(&offer.buyer(), wallet_buyer);

                let mut updated_wallets = match fee_strategy {
                    FeeStrategy::Recipient => fees.collect(view, offer.buyer())?,
                    FeeStrategy::Sender => fees.collect(view, offer.seller())?,
                    FeeStrategy::RecipientAndSender => {
                        fees.collect2(view, offer.seller(), offer.buyer())?
                    }
                    FeeStrategy::Intermediary => HashMap::<PublicKey, wallet::Wallet>::new(),
                };

                let mut wallet_seller = updated_wallets
                    .remove(&offer.seller())
                    .unwrap_or_else(|| wallet::Schema(&*view).fetch(&offer.seller()));
                let mut wallet_buyer = updated_wallets
                    .remove(&offer.buyer())
                    .unwrap_or_else(|| wallet::Schema(&*view).fetch(&offer.buyer()));
                let assets = offer
                    .assets()
                    .into_iter()
                    .map(|a| a.to_bundle())
                    .collect::<Vec<_>>();

                wallet::move_assets(&mut wallet_seller, &mut wallet_buyer, &assets)?;

                updated_wallets.insert(*offer.seller(), wallet_seller);
                updated_wallets.insert(*offer.buyer(), wallet_buyer);

                // Save changes to the database.
                for (key, wallet) in updated_wallets {
                    wallet::Schema(&mut *view).store(&key, wallet);
                }

                Ok(())
            })?;

        Ok(())
    }
}

lazy_static! {
    static ref VERIFY_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_trade_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_trade_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_trade_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_trade_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_trade_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
    static ref EXECUTE_DURATION: Histogram = register_histogram!(
        "dmbc_transaction_trade_execute_duration_seconds",
        "Duration of transaction execution."
    ).unwrap();
}

impl Transaction for Trade {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

        let wallets_ok = self.offer().buyer() != self.offer().seller();
        let fee_strategy_ok = match FeeStrategy::try_from(self.offer().fee_strategy()).unwrap() {
            FeeStrategy::Recipient | FeeStrategy::Sender | FeeStrategy::RecipientAndSender => true,
            _ => false,
        };

        if cfg!(fuzzing) {
            return wallets_ok && fee_strategy_ok;
        }

        let seller_verify_ok = crypto::verify(
            self.seller_signature(),
            &self.offer().raw,
            self.offer().seller(),
        );
        let buyer_verify_ok = self.verify_signature(&self.offer().buyer());

        if wallets_ok && fee_strategy_ok && buyer_verify_ok && seller_verify_ok {
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
