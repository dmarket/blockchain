use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::AssetBundle;

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

impl Transaction for DeleteAssets {
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

