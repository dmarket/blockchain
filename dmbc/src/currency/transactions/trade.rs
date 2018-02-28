use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::TradeAsset;

pub const TRADE_ID: u16 = 502;

encoding_struct! {
    struct TradeOffer {
        const SIZE = 72;

        field buyer: &PublicKey         [00 => 32]
        field seller: &PublicKey        [32 => 64]
        field assets: Vec<TradeAsset>   [64 => 72]
    }
}

message! {
    struct Trade {
        const TYPE = SERVICE_ID;
        const ID = TRADE_ID;
        const SIZE = 80;

        field offer:              TradeOffer    [00 => 8]
        field seed:               u64           [8 => 16]
        field seller_signature:   &Signature    [16 => 80]
    }
}

impl Trade {
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }
}

impl Transaction for Trade {
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
