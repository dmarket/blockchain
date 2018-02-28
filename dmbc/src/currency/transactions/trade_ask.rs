use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::TradeAsset;
use currency::error::Error;
use currency::status;

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

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }
}

impl Transaction for TradeAsk {
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
