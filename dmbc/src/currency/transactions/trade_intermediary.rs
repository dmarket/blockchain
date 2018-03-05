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

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        // TODO
        let _ = view;
        Err(Error::NotImplemented)
    }
}

impl Transaction for TradeIntermediary {
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
