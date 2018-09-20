use crypto::{PublicKey, Signature};

use assets::TradeAsset;
use transactions::components::service::SERVICE_ID;
use transactions::components::Intermediary;

use error::{Error, ErrorKind};

/// Transaction ID.
pub const TRADE_INTERMEDIARY_ID: u16 = 502;

evo_encoding_struct! {
    struct TradeOfferIntermediary {
        intermediary: Intermediary,
        buyer:        &PublicKey,
        seller:       &PublicKey,
        assets:       Vec<TradeAsset>,

        fee_strategy: u8,
        seed: u64,
        memo: &str,
    }
}

#[derive(Clone, Debug)]
pub struct TradeOfferIntermediaryWrapper {
    intermediary: Intermediary,
    buyer: PublicKey,
    seller: PublicKey,
    assets: Vec<TradeAsset>,

    fee_strategy: u8,
    seed: u64,
    memo: String,
}

impl TradeOfferIntermediaryWrapper {
    pub fn new(
        intermediary: Intermediary,
        seller: &PublicKey,
        buyer: &PublicKey,
        fee_strategy: u8,
        seed: u64,
        memo: &str
    ) -> Self {
        TradeOfferIntermediaryWrapper {
            intermediary: intermediary,
            buyer: *buyer,
            seller: *seller,
            assets: Vec::new(),
            fee_strategy: fee_strategy,
            seed: seed, 
            memo: memo.to_string()
        }
    }

    pub fn from_ptr<'a>(
        builder: *mut TradeOfferIntermediaryWrapper,
    ) -> Result<&'a mut TradeOfferIntermediaryWrapper, Error> {
        if builder.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "Offer isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *builder })
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
            self.fee_strategy,
            self.seed,
            &self.memo.as_str()
        )
    }
}

evo_message! {
    /// `trade_intermediary` transaction.
    struct TradeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TRADE_INTERMEDIARY_ID;

        offer:                  TradeOfferIntermediary,
        seller_signature:       &Signature,
        intermediary_signature: &Signature,
    }
}

#[derive(Clone, Debug)]
pub struct TradeIntermediaryWrapper {
    offer: TradeOfferIntermediary,
    seller_signature: Signature,
    intermediary_signature: Signature
}

impl TradeIntermediaryWrapper {
    pub fn new(
        offer: TradeOfferIntermediary,
        seller_signature: &Signature,
        intermediary_signature: &Signature,
    ) -> Self {
        TradeIntermediaryWrapper {
            offer: offer,
            seller_signature: *seller_signature,
            intermediary_signature: *intermediary_signature,
        }
    }

    pub fn from_ptr<'a>(
        wrapper: *mut TradeIntermediaryWrapper,
    ) -> Result<&'a mut TradeIntermediaryWrapper, Error> {
        if wrapper.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "transaction isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *wrapper })
    }

    pub fn unwrap(&self) -> TradeIntermediary {
        TradeIntermediary::new(
            self.offer.clone(),
            &self.seller_signature,
            &self.intermediary_signature
        )
    }
}
