use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::SERVICE_ID;
use currency::asset::TradeAsset;
use currency::transactions::components::Intermediary;

pub const TRADE_INTERMEDIARY_ID: u16 = 504;

encoding_struct! {
    struct TradeOfferIntermediary {
        const SIZE = 80;

        field intermediary: Intermediary [00 => 08]
        field buyer: &PublicKey          [08 => 40]
        field seller: &PublicKey         [40 => 72]
        field assets: Vec<TradeAsset>    [72 => 80]
    }
}

message! {
    struct TradeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TRADE_INTERMEDIARY_ID;
        const SIZE = 152;

        field offer:              TradeOfferIntermediary     [00 => 08]
        field seed:               u64                        [08 => 16]
        field seller_signature:   &Signature                 [16 => 80]
        field intermediary_signature: &Signature             [80 => 144]
        field data_info:          &str                       [144 => 152]
    }
}

impl TradeIntermediary {
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }
}

impl Transaction for TradeIntermediary {
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
