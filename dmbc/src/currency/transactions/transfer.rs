use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::AssetBundle;
use currency::error::Error;
use currency::status;

pub const TRANSFER_ID: u16 = 200;

message! {
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
        Err(Error::NotImplemented)
    }
}

impl Transaction for Transfer {
    fn verify(&self) -> bool {
        // TODO
        if cfg!(fuzzing) {
            return true;
        }

        false
    }

    fn execute(&self, view: &mut Fork) {
        // TODO
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
