use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{Service, SERVICE_ID};
use currency::assets::AssetBundle;
use currency::transactions::components::Fees;
use currency::error::Error;
use currency::status;
use currency::wallet;

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
        let mut genesis = wallet::Schema(&*view).fetch(&Service::genesis_wallet());

        let fees = Fees::new_transfer(&*view,self.assets())?;

        // Collect the blockchain fee. Execution shall not continue if this fails.
        let mut wallet_from = wallet::Schema(&*view).fetch(self.from());
        fees.collect_to_genesis(&mut wallet_from, &mut genesis)?;

        wallet::Schema(&mut *view).store(self.from(), wallet_from);
        wallet::Schema(&mut *view).store(&Service::genesis_wallet(), genesis);

        // Operations bellow must either all succeed, or return an error without
        // saving anything to the database.

        // Process third party fees.
        let mut updated_wallets = fees.collect_to_third_party(view, self.from())?;

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
        if cfg!(fuzzing) {
            return true;
        }
        let wallets_ok = self.from() != self.to();
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
