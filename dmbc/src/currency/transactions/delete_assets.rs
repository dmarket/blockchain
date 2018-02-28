use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::AssetBundle;
use currency::error::Error;
use currency::status;

pub const DELETE_ASSETS_ID: u16 = 400;

message! {
    struct DeleteAssets {
        const TYPE = SERVICE_ID;
        const ID = DELETE_ASSETS_ID;
        const SIZE = 48;

        field pub_key:     &PublicKey       [00 => 32]
        field assets:      Vec<AssetBundle> [32 => 40]
        field seed:        u64              [40 => 48]
    }
}

impl DeleteAssets {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        // TODO
        Err(Error::NotImplemented)
    }
}

impl Transaction for DeleteAssets {
    fn verify(&self) -> bool {
        // TODO
        if cfg!(fuzzing) {
            return true;
        }

        self.verify_signature(self.pub_key())
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

