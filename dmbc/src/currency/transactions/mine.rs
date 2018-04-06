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

/// Transaction ID.
pub const MINE_ID: u16 = 700;
const MINE_AMOUNT: u64 = 1_00000000;

message! {
    struct Mine {
        const TYPE = SERVICE_ID;
        const ID = MINE_ID;
        const SIZE = 40;

        field pub_key: &PublicKey [00 => 32]
        field seed:    u64        [32 => 40]
    }
}

impl Mine {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);
        let wallet = wallet::Schema(&*view).fetch(self.pub_key());
        let wallet = Wallet::new(wallet.balance() + MINE_AMOUNT, wallet.assets());
        wallet::Schema(&mut *view).store(&self.pub_key(), wallet);
        Ok(())
    }
}

impl Transaction for Mine {
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
        json!{self}
    }
}
