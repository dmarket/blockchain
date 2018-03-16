use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{SERVICE_ID, Service};
use currency::assets::TradeAsset;
use currency::transactions::components::Intermediary;
use currency::transactions::components::Fees;
use currency::error::Error;
use currency::status;
use currency::wallet;

/// Transaction ID.
pub const TRADE_INTERMEDIARY_ID: u16 = 502;

encoding_struct! {
    struct TradeOfferIntermediary {
        const SIZE = 80;

        field intermediary: Intermediary       [00 => 08]
        field buyer:        &PublicKey         [08 => 40]
        field seller:       &PublicKey         [40 => 72]
        field assets:       Vec<TradeAsset>    [72 => 80]
    }
}

message! {
    /// `trade_intermediary` transaction.
    struct TradeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TRADE_INTERMEDIARY_ID;
        const SIZE = 152;

        field offer:                  TradeOfferIntermediary     [00 => 08]
        field seed:                   u64                        [08 => 16]
        field seller_signature:       &Signature                 [16 => 80]
        field intermediary_signature: &Signature                 [80 => 144]
        field data_info:              &str                       [144 => 152]
    }
}

impl TradeIntermediary {
    /// Raw bytes of the offer.
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        let mut wallet_buyer = wallet::Schema(&*view).fetch(self.offer().buyer());
        let mut wallet_seller = wallet::Schema(&*view).fetch(self.offer().seller());
        let mut wallet_genesis = wallet::Schema(&*view).fetch(&Service::genesis_wallet());

        let mut fees = Fees::new_trade(&*view,&self.offer().assets())?;
        fees.add_fee(
            self.offer().intermediary().wallet(),
            self.offer().intermediary().commission(),
        );

        let total = self.offer().assets()
            .iter()
            .map(|asset| {asset.amount() * asset.price()})
            .sum::<u64>();

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
                updated_wallets.insert(*self.offer().buyer(), wallet_buyer);

                // Save changes to the database.
                for (key, wallet) in updated_wallets {
                    wallet::Schema(&mut *view).store(&key, wallet);
                }

                Ok(())
            })?;

        Ok(())
    }
}

impl Transaction for TradeIntermediary {
    fn verify(&self) -> bool {
        let offer = self.offer();

        let wallets_ok = offer.seller() != offer.buyer()
            && offer.intermediary().wallet() != offer.seller()
            && offer.intermediary().wallet() != offer.buyer();

        if cfg!(fuzzing) {
            return true;
        }

        let buyer_ok = self.verify_signature(offer.buyer());

        let seller_ok = crypto::verify(
            self.seller_signature(),
            &offer.raw,
            offer.seller()
        );
        let intermediary_ok = crypto::verify(
            self.intermediary_signature(),
            &offer.raw,
            offer.intermediary().wallet(),
        );

        wallets_ok && buyer_ok && seller_ok && intermediary_ok
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
