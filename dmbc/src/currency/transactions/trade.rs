use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{Service, SERVICE_ID};
use currency::assets::TradeAsset;
use currency::transactions::components::{FeeStrategy, ThirdPartyFees};
use currency::error::Error;
use currency::status;
use currency::wallet;
use currency::configuration::Configuration;

/// Transaction ID.
pub const TRADE_ID: u16 = 501;

encoding_struct! {
    struct TradeOffer {
        const SIZE = 73;

        field buyer: &PublicKey         [00 => 32]
        field seller: &PublicKey        [32 => 64]
        field assets: Vec<TradeAsset>   [64 => 72]

        field fee_strategy: u8          [72 => 73]
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
        info!("Processing tx: {:?}", self);

        let genesis_fee = Configuration::extract(view).fees().trade();

        let offer = self.offer();
        let fee_strategy =
            FeeStrategy::try_from(offer.fee_strategy()).expect("fee strategy must be valid");

        let mut genesis = wallet::Schema(&*view).fetch(&Service::genesis_wallet());
        // Collect the blockchain fee. Execution shall not continue if this fails.
        match fee_strategy {
            FeeStrategy::Recipient => {
                let mut buyer = wallet::Schema(&*view).fetch(offer.buyer());

                wallet::move_coins(&mut buyer, &mut genesis, genesis_fee)?;

                wallet::Schema(&mut *view).store(offer.buyer(), buyer);
            },
            FeeStrategy::Sender => {
                let mut seller = wallet::Schema(&*view).fetch(offer.seller());

                wallet::move_coins(&mut seller, &mut genesis, genesis_fee)?;

                wallet::Schema(&mut *view).store(offer.seller(), seller);
            },
            FeeStrategy::RecipientAndSender => {
                let mut buyer = wallet::Schema(&*view).fetch(offer.buyer());
                let mut seller = wallet::Schema(&*view).fetch(offer.seller());

                wallet::move_coins(&mut seller, &mut genesis, genesis_fee)?;
                wallet::move_coins(&mut buyer, &mut genesis, genesis_fee)?;

                wallet::Schema(&mut *view).store(offer.seller(), seller);
                wallet::Schema(&mut *view).store(offer.buyer(), buyer);
            }
            FeeStrategy::Intermediary => return Err(Error::InvalidTransaction),
        }

        wallet::Schema(&mut *view).store(&Service::genesis_wallet(), genesis);

        let fees = ThirdPartyFees::new_trade(&*view, &offer.assets())?;

        let mut wallet_buyer = wallet::Schema(&*view).fetch(offer.buyer());
        let mut wallet_seller = wallet::Schema(&*view).fetch(offer.seller());

        let total = offer.assets()
            .iter()
            .map(|asset| {asset.amount() * asset.price()})
            .sum();

        wallet::move_coins(&mut wallet_buyer, &mut wallet_seller, total)
            .or_else(|e| {
                wallet::Schema(&mut *view).store(&offer.seller(), wallet_seller.clone());
                wallet::Schema(&mut *view).store(&offer.buyer(), wallet_buyer.clone());

                Err(e)
            })
            .and_then(|_| {
                wallet::Schema(&mut *view).store(&offer.seller(), wallet_seller);
                wallet::Schema(&mut *view).store(&offer.buyer(), wallet_buyer);

                let mut updated_wallets = fees.collect(view, offer.seller())?;

                let mut wallet_seller = updated_wallets
                    .remove(&offer.seller())
                    .unwrap_or_else(|| wallet::Schema(&*view).fetch(&offer.seller()));
                let mut wallet_buyer = updated_wallets
                    .remove(&offer.buyer())
                    .unwrap_or_else(|| wallet::Schema(&*view).fetch(&offer.buyer()));
                let assets = offer.assets()
                    .into_iter()
                    .map(|a| a.to_bundle())
                    .collect::<Vec<_>>();

                wallet::move_assets(&mut wallet_seller, &mut wallet_buyer, &assets)?;

                updated_wallets.insert(*offer.seller(), wallet_seller);
                updated_wallets.insert(*offer.buyer(), wallet_buyer);

                // Save changes to the database.
                for (key, wallet) in updated_wallets {
                    wallet::Schema(&mut *view).store(&key, wallet);
                }

                Ok(())
            })?;

        Ok(())
    }
}

impl Transaction for Trade {
    fn verify(&self) -> bool {
        let wallets_ok = self.offer().buyer() != self.offer().seller();
        let fee_strategy_ok = match FeeStrategy::try_from(self.offer().fee_strategy()).unwrap() {
            FeeStrategy::Recipient | FeeStrategy::Sender | FeeStrategy::RecipientAndSender => true,
            _ => false,
        };

        if cfg!(fuzzing) {
            return wallets_ok && fee_strategy_ok;
        }

        let seller_verify_ok = crypto::verify(self.seller_signature(), &self.offer().raw, self.offer().seller());
        let buyer_verify_ok = self.verify_signature(&self.offer().buyer());

        wallets_ok && fee_strategy_ok && buyer_verify_ok && seller_verify_ok
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
