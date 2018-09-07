use exonum::crypto::{PublicKey, SecretKey};

use assets::TradeAsset;
use transactions::components::service::SERVICE_ID;

use error::{Error, ErrorKind};

/// Transaction ID.
pub const BID_OFFER_ID: u16 = 700;

evo_message! {
    /// `BidOffer` transaction.
    struct BidOffer {
        const TYPE = SERVICE_ID;
        const ID = BID_OFFER_ID;

        pub_key:      &PublicKey,
        asset:        TradeAsset,
        seed:         u64,
        memo:         &str,
    }
}

#[derive(Clone, Debug)]
pub struct BidOfferWrapper {
    pub_key:      PublicKey,
    asset:        TradeAsset,
    seed:         u64,
    memo:         String,
}

impl BidOfferWrapper {
    pub fn new(
        pub_key: &PublicKey,
        asset: TradeAsset,
        seed: u64,
        memo: &str
    ) -> Self {
        BidOfferWrapper {
            pub_key: *pub_key,
            asset: asset.clone(),
            seed: seed,
            memo: memo.to_string()
        }
    }

    pub fn from_ptr<'a>(
        builder: *mut BidOfferWrapper,
    ) -> Result<&'a mut BidOfferWrapper, Error> {
        if builder.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "Offer isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *builder })
    }

    pub fn unwrap(&self) -> BidOffer {
        BidOffer::new(
            &self.pub_key,
            self.asset.clone(),
            self.seed,
            self.memo.as_str(),
            &SecretKey::zero()
        )
    }
}