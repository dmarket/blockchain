use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::assets::AssetBundle;
use currency::SERVICE_ID;

pub const EXCHANGE_ID: u16 = 601;

encoding_struct! {
    struct ExchangeOffer {
        const SIZE = 89;

        field sender:           &PublicKey       [00 => 32]
        field sender_assets:    Vec<AssetBundle> [32 => 40]
        field sender_value:     u64              [40 => 48]

        field recipient:        &PublicKey       [48 => 80]
        field recipient_assets: Vec<AssetBundle> [80 => 88]

        field fee_strategy:     u8               [88 => 89]
    }
}

message! {
    struct Exchange {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_ID;
        const SIZE = 88;

        field offer:             ExchangeOffer     [00 => 8]
        field seed:              u64               [8 => 16]
        field sender_signature:  &Signature        [16 => 80]
        field data_info:         &str              [80 => 88]
    }
}

impl Exchange {
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }
}

impl Transaction for Exchange {
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

