use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::SERVICE_ID;
use currency::error::Error;
use currency::status;

pub const MINING_ID: u16 = 700;

message! {
    struct Mining {
        const TYPE = SERVICE_ID;
        const ID = MINING_ID;
        const SIZE = 40;

        field pub_key:     &PublicKey  [00 => 32]
        field seed:        u64         [32 => 40]
    }
}

impl Mining {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        // TODO
        let _ = view;
        Err(Error::NotImplemented)
    }
}

impl Transaction for Mining {
    fn verify(&self) -> bool {
        // TODO
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
        json!({})
    }
}
