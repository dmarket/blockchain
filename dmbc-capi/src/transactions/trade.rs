use exonum::crypto::{PublicKey, SecretKey, Signature};

use assets::TradeAsset;
use transactions::components::service::SERVICE_ID;

use error::{Error, ErrorKind};

/// Transaction ID.
pub const TRADE_ID: u16 = 501;

encoding_struct! {
    struct TradeOffer {
        buyer: &PublicKey,
        seller: &PublicKey,
        assets: Vec<TradeAsset>,

        fee_strategy: u8,
    }
}

#[derive(Clone, Debug)]
pub struct TradeOfferWrapper {
    buyer: PublicKey,
    seller: PublicKey,
    assets: Vec<TradeAsset>,

    fee_strategy: u8,
}

impl TradeOfferWrapper {
    pub fn new(seller: &PublicKey, buyer: &PublicKey, fee_strategy: u8) -> Self {
        TradeOfferWrapper {
            buyer: *buyer,
            seller: *seller,
            assets: Vec::new(),
            fee_strategy: fee_strategy,
        }
    }

    pub fn from_ptr<'a>(
        builder: *mut TradeOfferWrapper,
    ) -> Result<&'a mut TradeOfferWrapper, Error> {
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

    pub fn unwrap(&self) -> TradeOffer {
        TradeOffer::new(
            &self.buyer,
            &self.seller,
            self.assets.clone(),
            self.fee_strategy,
        )
    }
}

message! {
    /// `trade` transaction.
    struct Trade {
        const TYPE = SERVICE_ID;
        const ID = TRADE_ID;

        offer:              TradeOffer,
        seed:               u64,
        seller_signature:   &Signature,
    }
}

#[derive(Clone, Debug)]
pub struct TradeWrapper {
    offer: TradeOffer,
    seed: u64,
    seller_signature: Signature,
}

impl TradeWrapper {
    pub fn new(offer: TradeOffer, seed: u64, signature: &Signature) -> Self {
        TradeWrapper {
            offer: offer,
            seed: seed,
            seller_signature: *signature,
        }
    }

    pub fn from_ptr<'a>(wrapper: *mut TradeWrapper) -> Result<&'a mut TradeWrapper, Error> {
        if wrapper.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "transaction isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *wrapper })
    }

    pub fn unwrap(&self) -> Trade {
        Trade::new(
            self.offer.clone(),
            self.seed,
            &self.seller_signature,
            &SecretKey::zero(),
        )
    }
}
