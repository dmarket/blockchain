use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{Service, SERVICE_ID};
use currency::assets::TradeAsset;
use currency::transactions::components::Fees;
use currency::error::Error;
use currency::status;
use currency::wallet;

/// Transaction ID.
pub const TRADE_ID: u16 = 501;

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


        let mut wallet_buyer = wallet::Schema(&*view).fetch(self.offer().buyer());
        let mut wallet_seller = wallet::Schema(&*view).fetch(self.offer().seller());
        let mut wallet_genesis = wallet::Schema(&*view).fetch(&Service::genesis_wallet());

        let fees = Fees::new_trade(&*view,&self.offer().assets())?;

        // Collect the blockchain fee. Execution shall not continue if this fails.

        Ok(())
    }
}

impl Transaction for Trade {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return true;
        }

        let wallets_ok = self.offer().buyer() != self.offer().seller();
        let seller_verify_ok = crypto::verify(self.seller_signature(), &self.offer().raw, self.offer().seller());
        let buyer_verify_ok = self.verify_signature(&self.offer().buyer());

        wallets_ok && buyer_verify_ok && seller_verify_ok
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
