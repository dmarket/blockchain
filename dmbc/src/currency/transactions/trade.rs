use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::SERVICE_ID;
use currency::assets::TradeAsset;
use currency::error::Error;
use currency::status;

/// Transaction ID.
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
    /// `trade` transaction.
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
    /// Raw bytes of the offer.
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        // TODO
        let _ = view;
        Err(Error::NotImplemented)
    }
}

impl Transaction for Trade {
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
