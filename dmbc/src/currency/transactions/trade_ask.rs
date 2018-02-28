use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::TradeAsset;

pub const TRADE_ASK_ID: u16 = 501;

encoding_struct! {
    struct TradeAskOffer {
        const SIZE = 40;

        field seller: &PublicKey        [00 => 32]
        field assets: Vec<TradeAsset>   [32 => 40]
    }
}

message! {
    struct TradeAsk {
        const TYPE = SERVICE_ID;
        const ID = TRADE_ASK_ID;
        const SIZE = 120;

        field buyer:              &PublicKey    [00 => 32]
        field offer:              TradeAskOffer [32 => 40]
        field seed:               u64           [40 => 48]
        field seller_signature:   &Signature    [48 => 112]
        field data_info:          &str          [112 => 120]
    }
}

impl TradeAsk {
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }
}

impl Transaction for TradeAsk {
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
