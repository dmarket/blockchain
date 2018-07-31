use std::collections::HashMap;

use exonum::blockchain::Transaction;
use exonum::crypto::PublicKey;
use exonum::messages::Message;
use exonum::storage::Fork;
use prometheus::{IntCounter, Histogram};

use currency::assets::{TradeAsset, AssetBundle};
use currency::error::Error;
use currency::status;
use currency::transactions::components::{FeesCalculator, ThirdPartyFees};
use currency::wallet;
use currency::offers;
use currency::SERVICE_ID;
use currency::service::CONFIGURATION;

/// Transaction ID.
pub const ASK_OFFER_ID: u16 = 701;

message! {
    /// `AskOffer` transaction.
    struct AskOffer {
        const TYPE = SERVICE_ID;
        const ID = ASK_OFFER_ID;

        pub_key:      &PublicKey,
        asset:        TradeAsset,
        seed:         u64,
        data_info:    &str,
    }
}

impl FeesCalculator for AskOffer {
    fn calculate_fees(&self, view: &mut Fork) -> Result<HashMap<PublicKey, u64>, Error> {
        let genesis_fees = CONFIGURATION.read().unwrap().fees();
        let fees = ThirdPartyFees::new_trade(&*view, &[self.asset()])?;

        let mut fees_table = HashMap::new();
        if genesis_fees.recipient() != self.pub_key() {
            fees_table.insert(*self.pub_key(), genesis_fees.trade());
        }

        for (pub_key, fee) in fees.0 {
            if pub_key != *self.pub_key() {
                *fees_table.entry(*self.pub_key()).or_insert(0) += fee;
            }
        }

        Ok(fees_table)
    }
}

impl AskOffer {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        let genesis_fees = CONFIGURATION.read().unwrap().fees();

        let mut genesis = wallet::Schema(&*view).fetch(genesis_fees.recipient());

        // Collect the blockchain fee. Execution shall not continue if this fails.
        let mut wallet_from = wallet::Schema(&*view).fetch(self.pub_key());
        wallet::move_coins(&mut wallet_from, &mut genesis, genesis_fees.trade())?;

        wallet::Schema(&mut *view).store(self.pub_key(), wallet_from.clone());
        wallet::Schema(&mut *view).store(genesis_fees.recipient(), genesis);

        wallet_from.remove_coins(self.asset().amount() * self.asset().price())?;
        let mut open_offers = offers::Schema(&mut *view).fetch(&self.asset().id());
        let closed_bids =  open_offers.close_bid(self.asset().price(), self.asset().amount());
        if closed_bids.len() == 0 {
            let ask = offers::Offer::new(self.pub_key(), self.asset().amount(), &self.hash());
            open_offers.add_ask(self.asset().price(), ask);
        } else {
            let mut closed_amount:u64 = 0;
            #[derive(Debug)]
            struct UpdateData {
                amount: u64,
                coins: u64,
            }

            let mut update_wallets: HashMap<PublicKey, UpdateData> = HashMap::new(); // HashMap[wallet] = amount
            for offer in closed_bids.iter(){
                closed_amount += offer.amount;
                let v = update_wallets.entry(offer.wallet).or_insert(UpdateData{amount:0,coins:0});
                let amount = v.amount + offer.amount;
//                    let coins = v.coins + offer.amount * self.asset().price();
                let coins = v.coins + offer.amount * offer.price;
                *v = UpdateData{ amount, coins};
            }


            for (pk, data) in update_wallets.iter() {
                let w = wallet::Schema(&*view).fetch(pk);
                wallet::Schema(&mut *view).store(pk, wallet::Wallet::new(w.balance() + data.coins, w.assets()));
            }
            if self.asset().amount() > closed_amount {
                let ask_amount = self.asset().amount() - closed_amount;
                let ask = offers::Offer::new(self.pub_key(), ask_amount, &self.hash());
                open_offers.add_ask(self.asset().price(), ask);
            }
            if closed_amount > 0 {
                let asset = AssetBundle::new(self.asset().id(), closed_amount);
                wallet_from.add_assets(vec![asset]);
            }
        }

        offers::Schema(&mut *view).store(&self.asset().id(), open_offers);
        wallet::Schema(&mut *view).store(self.pub_key(), wallet_from);

        Ok(())
    }
}

lazy_static! {
    static ref VERIFY_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_ask_offer_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_ask_offer_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_ask_offer_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_ask_offer_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_ask_offer_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
    static ref EXECUTE_DURATION: Histogram = register_histogram!(
        "dmbc_transaction_ask_offer_execute_duration_seconds",
        "Duration of transaction execution."
    ).unwrap();
}

impl Transaction for AskOffer {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

        if cfg!(fuzzing) {
            return true;
        }

        let overflow = match self.asset().price().checked_mul(self.asset().amount()) {
            None => false,
            Some(_) => true,
        };

        let verify_ok = self.verify_signature(&self.pub_key());

        if verify_ok && overflow {
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
