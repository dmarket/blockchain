use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::SERVICE_ID;

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

impl Transaction for Mining {
    fn verify(&self) -> bool {
        unimplemented!()
    }

    fn execute(&self, view: &mut Fork) {
        let _ = view;
        unimplemented!()
    }

    fn info(&self) -> serde_json::Value {
        unimplemented!()
    }
}
