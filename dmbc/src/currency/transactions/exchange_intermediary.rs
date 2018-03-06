use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{SERVICE_ID, Service};
use currency::assets::AssetBundle;
use currency::transactions::components::{Intermediary, FeeStrategy, Fees};
use currency::error::Error;
use currency::status;
use currency::wallet;

pub const EXCHANGE_INTERMEDIARY_ID: u16 = 602;

encoding_struct! {
    struct ExchangeOfferIntermediary {
        const SIZE = 97;

        field intermediary:     Intermediary     [00 =>  8]

        field sender:           &PublicKey       [08 => 40]
        field sender_assets:    Vec<AssetBundle> [40 => 48]
        field sender_value:     u64              [48 => 56]

        field recipient:        &PublicKey       [56 => 88]
        field recipient_assets: Vec<AssetBundle> [88 => 96]

        field fee_strategy:     u8               [96 => 97]
    }
}

message! {
    struct ExchangeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_INTERMEDIARY_ID;
        const SIZE = 152;

        field offer:                  ExchangeOfferIntermediary [0 => 8]
        field seed:                   u64                       [8 => 16]
        field sender_signature:       &Signature                [16 => 80]
        field intermediary_signature: &Signature                [80 => 144]
        field data_info:              &str                      [144 => 152]
    }
}

impl ExchangeIntermediary {
    pub fn offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        let offer = self.offer();

        let fee_strategy = FeeStrategy::try_from(offer.fee_strategy())
            .expect("fee strategy must be valid");

        let mut fees = Fees::new_exchange(
            &*view,
            offer.sender_assets().into_iter()
                 .chain(offer.recipient_assets().into_iter()),
        )?;

        // Insert intermediary as one of third party fees.
        fees.add_fee(offer.intermediary().wallet(), offer.intermediary().commission());

        let mut genesis = wallet::Schema(&*view).fetch(&Service::genesis_wallet());
        
        // Collect the blockchain fee. Execution shall not continue if this fails.
        match fee_strategy {
            FeeStrategy::Recipient => {
                let mut recipient = wallet::Schema(&*view).fetch(offer.recipient());

                fees.collect_to_genesis(&mut recipient, &mut genesis)?;

                wallet::Schema(&mut*view).store(offer.recipient(), recipient);
            }
            FeeStrategy::Sender => {
                let mut sender = wallet::Schema(&*view).fetch(offer.sender());

                fees.collect_to_genesis(&mut sender, &mut genesis)?;

                wallet::Schema(&mut*view).store(offer.sender(), sender);
            }
            FeeStrategy::RecipientAndSender => {
                let mut recipient = wallet::Schema(&*view).fetch(offer.recipient());
                let mut sender    = wallet::Schema(&*view).fetch(offer.sender());

                fees.collect_to_genesis_2(&mut sender, &mut recipient, &mut genesis)?;

                wallet::Schema(&mut*view).store(offer.sender(),       sender);
                wallet::Schema(&mut*view).store(offer.recipient(), recipient);
            }
            FeeStrategy::Intermediary => {
                let mut intermediary = wallet::Schema(&*view).fetch(offer.intermediary().wallet());

                fees.collect_to_genesis(&mut intermediary, &mut genesis)?;

                wallet::Schema(&mut*view).store(offer.intermediary().wallet(), intermediary);
            }
        }

        wallet::Schema(&mut*view).store(&Service::genesis_wallet(), genesis);

        // Operations bellow must either all succeed, or return an error without
        // saving anything to the database.

        // Process third party fees.
        let mut updated_wallets = match fee_strategy {
            FeeStrategy::Recipient => fees.collect_to_third_party(view, offer.recipient())?,
            FeeStrategy::Sender => fees.collect_to_third_party(view, offer.sender())?,
            FeeStrategy::RecipientAndSender => fees.collect_to_third_party_2(view, offer.sender(), offer.recipient())?,
            FeeStrategy::Intermediary => fees.collect_to_third_party(view, offer.intermediary().wallet())?,
        };

        // Process the main transaction.
        let mut sender = updated_wallets.remove(&offer.sender()).unwrap_or_else(|| {
            wallet::Schema(&*view).fetch(&offer.sender())
        });
        let mut recipient = updated_wallets.remove(&offer.recipient()).unwrap_or_else(|| {
            wallet::Schema(&*view).fetch(&offer.recipient())
        });

        wallet::move_coins(&mut sender, &mut recipient, offer.sender_value())?;
        wallet::move_assets(&mut sender, &mut recipient, &offer.sender_assets())?;
        wallet::move_assets(&mut recipient, &mut sender, &offer.recipient_assets())?;

        updated_wallets.insert(*offer.sender(), sender);
        updated_wallets.insert(*offer.recipient(), recipient);

        // Save changes to the database.
        for (key, wallet) in updated_wallets {
            wallet::Schema(&mut*view).store(&key, wallet);
        }

        Ok(())
    }
}

impl Transaction for ExchangeIntermediary {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return true;
        }

        let offer = self.offer();

        let wallets_ok = offer.sender() != offer.recipient()
                      && offer.intermediary().wallet() != offer.sender()
                      && offer.intermediary().wallet() != offer.recipient();
        let fee_strategy_ok = FeeStrategy::try_from(offer.fee_strategy()).is_some();
        let recipient_ok = self.verify_signature(offer.recipient());
        let sender_ok = crypto::verify(self.sender_signature(), &offer.raw, offer.sender());
        let intermediary_ok = crypto::verify(self.intermediary_signature(), &offer.raw, offer.intermediary().wallet());

        wallets_ok && fee_strategy_ok && recipient_ok && sender_ok && intermediary_ok
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}

