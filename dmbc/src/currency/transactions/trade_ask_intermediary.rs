use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::TradeAsset;
use currency::transactions::components::Intermediary;

pub const TRADE_ASK_INTERMEDIARY_ID: u16 = 503;

encoding_struct! {
    struct TradeAskOfferIntermediary {
        const SIZE = 48;

        field intermediary: Intermediary [00 => 08]
        field seller: &PublicKey         [08 => 40]
        field assets: Vec<TradeAsset>    [40 => 48]
    }
}

message! {
    struct TradeAskIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TRADE_ASK_INTERMEDIARY_ID;
        const SIZE = 184;

        field buyer:                  &PublicKey                     [00 => 32]
        field offer:                  TradeAskOfferIntermediary      [32 => 40]
        field seed:                   u64                            [40 => 48]
        field seller_signature:       &Signature                     [48 => 112]
        field intermediary_signature: &Signature                     [112 => 176]
        field data_info:              &str                           [176 => 184]
    }
}

impl TradeAskIntermediary {
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }
}

impl Transaction for TradeAskIntermediary {
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
