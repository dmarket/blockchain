use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::TradeAsset;
use currency::transactions::components::Intermediary;
use currency::error::Error;
use currency::status;

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

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        // TODO
        let _ = view;
        Err(Error::NotImplemented)
    }
}

impl Transaction for TradeAskIntermediary {
    fn verify(&self) -> bool {
        // TODO
        if cfg!(fuzzing) {
            return true;
        }

        false
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
