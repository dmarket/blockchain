use std::collections::HashMap;

use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::AssetBundle;
use currency::transactions::components::{FeesCalculator, ThirdPartyFees};
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

impl FeesCalculator for Transfer {
    fn calculate_fees(&self, view: &mut Fork) -> Result<HashMap<PublicKey, u64>, Error> {
        let genesis_fees = Configuration::extract(view).fees();
        let fees = ThirdPartyFees::new_transfer(&*view,self.assets())?;

        let mut fees_table = HashMap::new();
        if genesis_fees.recipient() != self.from() {
            fees_table.insert(*self.from(), genesis_fees.transfer());
        }

        for (pub_key, fee) in fees.0 {
            if pub_key != *self.from() {
                *fees_table.entry(*self.from()).or_insert(0) += fee;
            }
        }


        Ok(fees_table)
    }
}

impl Transfer {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        let genesis_fees = Configuration::extract(view).fees();

        let mut genesis = wallet::Schema(&*view).fetch(genesis_fees.recipient());

        // Collect the blockchain fee. Execution shall not continue if this fails.
        let mut wallet_from = wallet::Schema(&*view).fetch(self.from());
        wallet::move_coins(&mut wallet_from, &mut genesis, genesis_fees.transfer())?;

        wallet::Schema(&mut *view).store(self.from(), wallet_from);
        wallet::Schema(&mut *view).store(genesis_fees.recipient(), genesis);

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

impl Transaction for Transfer {
    fn verify(&self) -> bool {
        let wallets_ok = self.from() != self.to();

        if cfg!(fuzzing) {
            return wallets_ok;
        }

        let verify_ok = self.verify_signature(&self.from());

        wallets_ok && verify_ok
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
