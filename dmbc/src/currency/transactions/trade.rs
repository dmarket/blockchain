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
        let total = self.offer().assets()
            .iter()
            .map(|asset| {asset.amount() * asset.price()})
            .sum();
        println!("{:?}", &total);

        wallet::move_coins(&mut wallet_buyer, &mut wallet_seller, total)
            .or_else(|e| {
                fees.collect_to_genesis(&mut wallet_seller, &mut wallet_genesis)?;
                wallet::Schema(&mut *view).store(&self.offer().seller(), wallet_seller.clone());
                wallet::Schema(&mut *view).store(&self.offer().buyer(), wallet_buyer.clone());
                wallet::Schema(&mut *view).store(&Service::genesis_wallet(), wallet_genesis.clone());

                Err(e)
            })
            .and_then(|_| {
                fees.collect_to_genesis(&mut wallet_seller, &mut wallet_genesis)?;
                wallet::Schema(&mut *view).store(&self.offer().seller(), wallet_seller);
                wallet::Schema(&mut *view).store(&self.offer().buyer(), wallet_buyer);
                wallet::Schema(&mut *view).store(&Service::genesis_wallet(), wallet_genesis);

                let mut updated_wallets = fees.collect_to_third_party(view, self.offer().seller())?;

                let mut wallet_seller = updated_wallets
                    .remove(&self.offer().seller())
                    .unwrap_or_else(|| wallet::Schema(&*view).fetch(&self.offer().seller()));
                let mut wallet_buyer = updated_wallets
                    .remove(&self.offer().buyer())
                    .unwrap_or_else(|| wallet::Schema(&*view).fetch(&self.offer().buyer()));
                let assets = self.offer().assets()
                    .into_iter()
                    .map(|a| a.to_bundle())
                    .collect::<Vec<_>>();

                wallet::move_assets(&mut wallet_seller, &mut wallet_buyer, &assets)?;

                updated_wallets.insert(*self.offer().seller(), wallet_seller);

                // Save changes to the database.
                for (key, wallet) in updated_wallets {
                    wallet::Schema(&mut *view).store(&key, wallet);
                }

                Ok(())
            })?;



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
