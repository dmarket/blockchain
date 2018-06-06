use exonum::crypto::{PublicKey, Signature, SecretKey};

use assets::TradeAsset;
use transactions::components::Intermediary;
use transactions::components::service::SERVICE_ID;

use error::{Error, ErrorKind};

/// Transaction ID.
pub const TRADE_INTERMEDIARY_ID: u16 = 502;

encoding_struct! {
    struct TradeOfferIntermediary {
        intermediary: Intermediary,
        buyer:        &PublicKey,
        seller:       &PublicKey,
        assets:       Vec<TradeAsset>,

        fee_strategy: u8,
    }
}

#[derive(Clone, Debug)]
pub struct TradeOfferIntermediaryWrapper {
    intermediary: Intermediary,
    buyer:        PublicKey,
    seller:       PublicKey,
    assets:       Vec<TradeAsset>,

    fee_strategy: u8,
}

impl TradeOfferIntermediaryWrapper {
    pub fn new(intermediary: Intermediary, seller: &PublicKey, buyer: &PublicKey, fee_strategy: u8) -> Self {
        TradeOfferIntermediaryWrapper {
            intermediary: intermediary,
            buyer: *buyer,
            seller: *seller,
            assets: Vec::new(),
            fee_strategy: fee_strategy,
        }
    }

    pub fn from_ptr<'a>(builder: *mut TradeOfferIntermediaryWrapper) -> Result<&'a mut TradeOfferIntermediaryWrapper, Error> {
        if builder.is_null() {
            return Err(
                Error::new(
                    ErrorKind::Text("Offer isn't initialized".to_string())
                )
            );
        }
        Ok( unsafe { &mut *builder } )
    }

    pub fn add_asset(&mut self, asset: TradeAsset) {
         self.assets.push(asset);
    }

    pub fn unwrap(&self) -> TradeOfferIntermediary {
        TradeOfferIntermediary::new(
            self.intermediary.clone(),
            &self.buyer, 
            &self.seller, 
            self.assets.clone(), 
            self.fee_strategy
        )
    }
}

message! {
    /// `trade_intermediary` transaction.
    struct TradeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TRADE_INTERMEDIARY_ID;

        offer:                  TradeOfferIntermediary,
        seed:                   u64,
        seller_signature:       &Signature,
        intermediary_signature: &Signature,
        data_info:              &str,
    }
}

#[derive(Clone, Debug)]
pub struct TradeIntermediaryWrapper {
    offer:                  TradeOfferIntermediary,
    seed:                   u64,
    seller_signature:       Signature,
    intermediary_signature: Signature,
    data_info:              String,
}

impl TradeIntermediaryWrapper {
    pub fn new(offer: TradeOfferIntermediary, seed: u64, seller_signature: &Signature, intermediary_signature: &Signature, data_info: &str) -> Self {
        TradeIntermediaryWrapper {
            offer: offer,
            seed: seed,
            seller_signature: *seller_signature,
            intermediary_signature: *intermediary_signature,
            data_info: data_info.to_string()
        }
    }

    pub fn from_ptr<'a>(wrapper: *mut TradeIntermediaryWrapper) -> Result<&'a mut TradeIntermediaryWrapper, Error> {
        if wrapper.is_null() {
            return Err(
                Error::new(
                    ErrorKind::Text("transaction isn't initialized".to_string())
                )
            );
        }
        Ok( unsafe { &mut *wrapper } )
    }

    pub fn unwrap(&self) -> TradeIntermediary {
        TradeIntermediary::new(
            self.offer.clone(),
            self.seed,
            &self.seller_signature,
            &self.intermediary_signature,
            self.data_info.as_str(),
            &SecretKey::zero(),
        )
    }
}