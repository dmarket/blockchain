use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;
use prometheus::Counter;

use currency::{Service, SERVICE_ID};
use currency::assets::AssetBundle;
use currency::transactions::components::ThirdPartyFees;
use currency::error::Error;
use currency::status;
use currency::wallet;
use currency::configuration::Configuration;

/// Transaction ID.
pub const TRANSFER_ID: u16 = 200;

message! {
    /// `transfer` transaction.
    struct Transfer {
        const TYPE = SERVICE_ID;
        const ID = TRANSFER_ID;
        const SIZE = 96;

        field from:      &PublicKey       [00 => 32]
        field to:        &PublicKey       [32 => 64]
        field amount:    u64              [64 => 72]
        field assets:    Vec<AssetBundle> [72 => 80]
        field seed:      u64              [80 => 88]
        field data_info: &str             [88 => 96]
    }
}

impl Transfer {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        let genesis_fee = Configuration::extract(view).fees().transfer();

        let mut genesis = wallet::Schema(&*view).fetch(&Service::genesis_wallet());

        // Collect the blockchain fee. Execution shall not continue if this fails.
        let mut wallet_from = wallet::Schema(&*view).fetch(self.from());
        wallet::move_coins(&mut wallet_from, &mut genesis, genesis_fee)?;

        wallet::Schema(&mut *view).store(self.from(), wallet_from);
        wallet::Schema(&mut *view).store(&Service::genesis_wallet(), genesis);

        let fees = ThirdPartyFees::new_transfer(&*view,self.assets())?;

        // Operations bellow must either all succeed, or return an error without
        // saving anything to the database.

        // Process third party fees.
        let mut updated_wallets = fees.collect(view, self.from())?;

        // Process the main transaction.
        let mut wallet_from = updated_wallets
            .remove(&self.from())
            .unwrap_or_else(|| wallet::Schema(&*view).fetch(&self.from()));

        let mut wallet_to = updated_wallets
            .remove(&self.to())
            .unwrap_or_else(|| wallet::Schema(&*view).fetch(&self.to()));

        //wallet::Schema(&*view).fetch(self.to());
        wallet::move_coins(&mut wallet_from, &mut wallet_to, self.amount())?;
        wallet::move_assets(&mut wallet_from, &mut wallet_to, &self.assets())?;

        updated_wallets.insert(*self.from(), wallet_from);
        updated_wallets.insert(*self.to(), wallet_to);

        // Save changes to the database.
        for (key, wallet) in updated_wallets {
            wallet::Schema(&mut *view).store(&key, wallet);
        }

        Ok(())
    }
}

lazy_static! {
    static ref VERIFY_COUNT: Counter = register_counter!(
        "dmbc_transaction_transfer_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_transfer_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: Counter = register_counter!(
        "dmbc_transaction_transfer_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_transfer_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: Counter = register_counter!(
        "dmbc_transaction_transfer_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
}

impl Transaction for Transfer {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

        let wallets_ok = self.from() != self.to();

        if cfg!(fuzzing) {
            return wallets_ok;
        }

        let verify_ok = self.verify_signature(&self.from());

        if wallets_ok && verify_ok {
            VERIFY_SUCCESS_COUNT.inc();
            true
        } else {
            false
        }
    }

    fn execute(&self, view: &mut Fork) {
        EXECUTE_COUNT.inc();

        let result = self.process(view);

        if let &Ok(_) = &result {
            EXECUTE_SUCCESS_COUNT.inc();
        }

        status::Schema(view).store(self.hash(), result);

        EXECUTE_FINISH_COUNT.inc();
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
