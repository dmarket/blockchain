use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::SERVICE_ID;
use currency::asset::AssetBundle;

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

impl Transaction for Transfer {
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
