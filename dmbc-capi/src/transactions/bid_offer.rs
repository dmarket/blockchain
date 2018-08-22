use exonum::crypto::{PublicKey, SecretKey};

use assets::TradeAsset;
use transactions::components::service::SERVICE_ID;

use error::{Error, ErrorKind};

/// Transaction ID.
pub const BID_OFFER_ID: u16 = 700;

message! {
    /// `BidOffer` transaction.
    struct BidOffer {
        const TYPE = SERVICE_ID;
        const ID = BID_OFFER_ID;

        pub_key:      &PublicKey,
        asset:        TradeAsset,
        seed:         u64,
        data_info:    &str,
    }
}

#[derive(Clone, Debug)]
pub struct BidOfferWrapper {
    pub_key:      PublicKey,
    asset:        TradeAsset,
    seed:         u64,
    data_info:    String,
}

impl BidOfferWrapper {
    pub fn new(
        pub_key: &PublicKey,
        asset: TradeAsset,
        seed: u64,
        data_info: &str
    ) -> Self {
        BidOfferWrapper {
            pub_key: *pub_key,
            asset: asset.clone(),
            seed: seed,
            data_info: data_info.to_string()
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
            self.data_info.as_str(),
            &SecretKey::zero()
        )
    }
}