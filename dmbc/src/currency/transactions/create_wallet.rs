use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::SERVICE_ID;
use currency::error::Error;
use currency::wallet;
use currency::wallet::Wallet;
use currency::status;

pub const CREATE_WALLET_ID: u16 = 100;
const INITIAL_BALANCE: u64 = 1_00000000;

message! {
    struct CreateWallet {
        const TYPE = SERVICE_ID;
        const ID = CREATE_WALLET_ID;
        const SIZE = 32;

        field pub_key:     &PublicKey  [00 => 32]
    }
}

impl CreateWallet {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        let wallet = Wallet::new(INITIAL_BALANCE, Vec::new());
        wallet::Schema(&mut *view).store(&self.pub_key(), wallet);
        Ok(())
    }
}

impl Transaction for CreateWallet {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return true;
        }

        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!{[]}
    }
}
