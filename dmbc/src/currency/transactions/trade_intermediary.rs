use std::collections::HashMap;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use prometheus::{IntCounter, Histogram};

use currency::assets::TradeAsset;
use currency::error::Error;
use currency::status;
use currency::transactions::components::Intermediary;
use currency::transactions::components::{FeeStrategy, FeesCalculator, ThirdPartyFees};
use currency::wallet;
use currency::SERVICE_ID;
use currency::service::CONFIGURATION;

/// Transaction ID.
pub const TRADE_INTERMEDIARY_ID: u16 = 502;

encoding_struct! {
    struct TradeOfferIntermediary {
        intermediary: Intermediary,
        buyer:        &PublicKey,
        seller:       &PublicKey,
        assets:       Vec<TradeAsset>,

        fee_strategy: u8,
    }
}

message! {
    /// `trade_intermediary` transaction.
    struct TradeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TRADE_INTERMEDIARY_ID;

        offer:                  TradeOfferIntermediary,
        seed:                   u64,
        seller_signature:       &Signature,
        intermediary_signature: &Signature,
        data_info:              &str,
    }
}

impl FeesCalculator for TradeIntermediary {
    fn calculate_fees(&self, view: &mut Fork) -> Result<HashMap<PublicKey, u64>, Error> {
        let offer = self.offer();
        let genesis_fees = CONFIGURATION.read().unwrap().fees();
        let mut fees = ThirdPartyFees::new_trade(&*view, &offer.assets())?;
        fees.add_fee(
            offer.intermediary().wallet(),
            offer.intermediary().commission()
        );
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

impl TradeIntermediary {
    fn payers(&self, fee_strategy: &FeeStrategy, fee: u64) -> Result<Vec<(PublicKey, u64)>, Error> {
        let offer = self.offer();
        let payers = match *fee_strategy {
            FeeStrategy::Recipient => vec![(*offer.buyer(), fee)],
            FeeStrategy::Sender => vec![(*offer.seller(), fee)],
            FeeStrategy::RecipientAndSender => {
                vec![(*offer.seller(), fee / 2), (*offer.buyer(), fee / 2)]
            }
            FeeStrategy::Intermediary => vec![(*offer.intermediary().wallet(), fee)],
        };
        Ok(payers)
    }

    /// Raw bytes of the offer.
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn can_move_assets(&self, view: &mut Fork) -> Result<(), Error> {
        let mut wallet_buyer = wallet::Schema(&*view).fetch(self.offer().buyer());
        let mut wallet_seller = wallet::Schema(&*view).fetch(self.offer().seller());

        let assets = self.offer()
                    .assets()
                    .into_iter()
                    .map(|a| a.to_bundle())
                    .collect::<Vec<_>>();

        wallet::move_assets(&mut wallet_seller, &mut wallet_buyer, &assets)?;

        Ok(())
    }

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);

        let genesis_fees = CONFIGURATION.read().unwrap().fees();

        let offer = self.offer();

        let fee_strategy =
            FeeStrategy::try_from(offer.fee_strategy()).expect("fee strategy must be valid");

        let total = offer
            .assets()
            .iter()
            .map(|asset| asset.amount() * asset.price())
            .sum::<u64>();

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
            FeeStrategy::Intermediary => {
                let mut intermediary = wallet::Schema(&*view).fetch(offer.intermediary().wallet());

                wallet::move_coins(&mut intermediary, &mut genesis, genesis_fees.trade())?;

                wallet::Schema(&mut *view).store(offer.intermediary().wallet(), intermediary);
            }
        }

        wallet::Schema(&mut *view).store(genesis_fees.recipient(), genesis);

        let mut fees = ThirdPartyFees::new_trade(&*view, &offer.assets())?;
        fees.add_fee(
            offer.intermediary().wallet(),
            offer.intermediary().commission(),
        );

        self.can_move_assets(view)?;

        let mut wallet_buyer = wallet::Schema(&*view).fetch(self.offer().buyer());
        let mut wallet_seller = wallet::Schema(&*view).fetch(self.offer().seller());

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
                    FeeStrategy::Intermediary => fees.collect(view, offer.intermediary().wallet())?,
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
        "dmbc_transaction_trade_intermediary_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_trade_intermediary_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_trade_intermediary_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_trade_intermediary_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_trade_intermediary_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
    static ref EXECUTE_DURATION: Histogram = register_histogram!(
        "dmbc_transaction_trade_intermediary_execute_duration_seconds",
        "Duration of transaction execution."
    ).unwrap();
}

impl Transaction for TradeIntermediary {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

        let offer = self.offer();

        let wallets_ok = offer.seller() != offer.buyer()
            && offer.intermediary().wallet() != offer.seller()
            && offer.intermediary().wallet() != offer.buyer();
        let fee_strategy_ok = FeeStrategy::try_from(offer.fee_strategy()).is_some();

        if cfg!(fuzzing) {
            return wallets_ok && fee_strategy_ok;
        }

        let buyer_ok = self.verify_signature(offer.buyer());

        let seller_ok = crypto::verify(self.seller_signature(), &offer.raw, offer.seller());
        let intermediary_ok = crypto::verify(
            self.intermediary_signature(),
            &offer.raw,
            offer.intermediary().wallet(),
        );

        if wallets_ok && fee_strategy_ok && buyer_ok && seller_ok && intermediary_ok {
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
