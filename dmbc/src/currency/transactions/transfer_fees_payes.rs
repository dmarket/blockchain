use std::collections::HashMap;

use exonum::blockchain::Transaction;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use prometheus::{IntCounter, Histogram};

use currency::assets::AssetBundle;
use currency::error::Error;
use currency::status;
use currency::transactions::components::{FeesCalculator, ThirdPartyFees};
use currency::transactions::components::{mask_for, has_permission, Permissions};
use currency::wallet;
use currency::SERVICE_ID;
use currency::service::{CONFIGURATION, PERMISSIONS};

/// Transaction ID.
pub const TRANSFER_FEES_PAYER_ID: u16 = 201;


encoding_struct! {
    struct TransferOffer {

        from:       &PublicKey,
        to:         &PublicKey,
        fees_payer: &PublicKey,

        amount:     u64,
        assets:     Vec<AssetBundle>,
        seed:       u64,
        data_info:  &str,
    }
}
message! {
    /// `transfer` transaction.
    struct TransferWithFeesPayer {
        const TYPE = SERVICE_ID;
        const ID = TRANSFER_FEES_PAYER_ID;

        offer:                TransferOffer,
        fees_payer_signature: &Signature,
    }
}

impl Permissions for TransferWithFeesPayer {
    fn is_authorized(&self) -> bool {
        let permissions = PERMISSIONS.read().unwrap();
        let global_mask = CONFIGURATION.read().unwrap().permissions().global_permission_mask();
        let tx_mask = mask_for(TRANSFER_FEES_PAYER_ID);

        match permissions.get(self.offer().from()) {
            Some(mask) => { 
                if !has_permission(*mask, tx_mask) {
                    return false;
                }
            },
            None => ()
        }

        match permissions.get(self.offer().to()) {
            Some(mask) => {
                if !has_permission(*mask, tx_mask) {
                    return false;
                }
            },
            None => ()
        }

        match permissions.get(self.offer().fees_payer()) {
            Some(mask) => {
                if !has_permission(*mask, tx_mask) {
                    return false;
                }
            },
            None => ()
        }

        has_permission(global_mask, tx_mask)
    }
}

impl FeesCalculator for TransferWithFeesPayer {
    fn calculate_fees(&self, view: &mut Fork) -> Result<HashMap<PublicKey, u64>, Error> {
        let genesis_fees = CONFIGURATION.read().unwrap().fees();
        let fees = ThirdPartyFees::new_transfer(&*view, self.offer().assets())?;

        let mut fees_table = HashMap::new();
        if genesis_fees.recipient() != self.offer().fees_payer() {
            fees_table.insert(*self.offer().fees_payer(), genesis_fees.transfer());
        }

        for (pub_key, fee) in fees.0 {
            if pub_key != *self.offer().fees_payer() {
                *fees_table.entry(*self.offer().fees_payer()).or_insert(0) += fee;
            }
        }

        Ok(fees_table)
    }
}

impl TransferWithFeesPayer {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        let genesis_fees = CONFIGURATION.read().unwrap().fees();

        let mut genesis = wallet::Schema(&*view).fetch(genesis_fees.recipient());

        // Collect the blockchain fee. Execution shall not continue if this fails.
        let mut fees_payer = wallet::Schema(&*view).fetch(self.offer().fees_payer());
        wallet::move_coins(&mut fees_payer, &mut genesis, genesis_fees.transfer())?;

        wallet::Schema(&mut *view).store(self.offer().fees_payer(), fees_payer);
        wallet::Schema(&mut *view).store(genesis_fees.recipient(), genesis);

        let fees = ThirdPartyFees::new_transfer(&*view, self.offer().assets())?;

        // Operations bellow must either all succeed, or return an error without
        // saving anything to the database.

        // Process third party fees.
        let mut updated_wallets = fees.collect(view, self.offer().fees_payer())?;

        // Process the main transaction.
        let mut wallet_from = updated_wallets
            .remove(&self.offer().from())
            .unwrap_or_else(|| wallet::Schema(&*view).fetch(&self.offer().from()));

        let mut wallet_to = updated_wallets
            .remove(&self.offer().to())
            .unwrap_or_else(|| wallet::Schema(&*view).fetch(&self.offer().to()));

        //wallet::Schema(&*view).fetch(self.to());
        wallet::move_coins(&mut wallet_from, &mut wallet_to, self.offer().amount())?;
        wallet::move_assets(&mut wallet_from, &mut wallet_to, &self.offer().assets())?;

        updated_wallets.insert(*self.offer().from(), wallet_from);
        updated_wallets.insert(*self.offer().to(), wallet_to);

        // Save changes to the database.
        for (key, wallet) in updated_wallets {
            wallet::Schema(&mut *view).store(&key, wallet);
        }

        Ok(())
    }
}

lazy_static! {
    static ref VERIFY_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_transfer_fees_payer_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_transfer_fees_payer_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_transfer_fees_payer_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_transfer_fees_payer_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: IntCounter = register_int_counter!(
        "dmbc_transaction_transfer_fees_payer_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
    static ref EXECUTE_DURATION: Histogram = register_histogram!(
        "dmbc_transaction_transfer_fees_payer_execute_duration_seconds",
        "Duration of transaction execution."
    ).unwrap();
}

impl Transaction for TransferWithFeesPayer {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

        let wallets_ok = (self.offer().from() != self.offer().to())
            && (self.offer().from() != self.offer().fees_payer());

        if cfg!(fuzzing) {
            return wallets_ok;
        }

        if !self.is_authorized() {
            return false;
        }

        let verify_ok = self.verify_signature(&self.offer().from());

        if wallets_ok && verify_ok {
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
