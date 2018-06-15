use exonum::crypto::{PublicKey, SecretKey, Signature};

use assets::AssetBundle;
use transactions::components::service::SERVICE_ID;
use transactions::components::Intermediary;

use error::{Error, ErrorKind};

/// Transaction ID.
pub const EXCHANGE_INTERMEDIARY_ID: u16 = 602;

encoding_struct! {
    struct ExchangeOfferIntermediary {
        intermediary:     Intermediary,

        sender:           &PublicKey,
        sender_assets:    Vec<AssetBundle>,
        sender_value:     u64,

        recipient:        &PublicKey,
        recipient_assets: Vec<AssetBundle>,

        fee_strategy:     u8,
    }
}

#[derive(Clone, Debug)]
pub struct ExchangeOfferIntermediaryWrapper {
    intermediary: Intermediary,

    sender: PublicKey,
    sender_assets: Vec<AssetBundle>,
    sender_value: u64,

    recipient: PublicKey,
    recipient_assets: Vec<AssetBundle>,

    fee_strategy: u8,
}

impl ExchangeOfferIntermediaryWrapper {
    pub fn new(
        intermediary: Intermediary,
        sender: &PublicKey,
        sender_value: u64,
        recipient: &PublicKey,
        fee_strategy: u8,
    ) -> Self {
        ExchangeOfferIntermediaryWrapper {
            intermediary: intermediary,

            sender: *sender,
            sender_assets: Vec::new(),
            sender_value: sender_value,

            recipient: *recipient,
            recipient_assets: Vec::new(),
            fee_strategy: fee_strategy,
        }
    }

    pub fn from_ptr<'a>(
        builder: *mut ExchangeOfferIntermediaryWrapper,
    ) -> Result<&'a mut ExchangeOfferIntermediaryWrapper, Error> {
        if builder.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "Offer isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *builder })
    }

    pub fn add_sender_asset(&mut self, asset: AssetBundle) {
        self.sender_assets.push(asset);
    }

    pub fn add_recipient_asset(&mut self, asset: AssetBundle) {
        self.recipient_assets.push(asset);
    }

    pub fn unwrap(&self) -> ExchangeOfferIntermediary {
        ExchangeOfferIntermediary::new(
            self.intermediary.clone(),
            &self.sender,
            self.sender_assets.clone(),
            self.sender_value,
            &self.recipient,
            self.recipient_assets.clone(),
            self.fee_strategy,
        )
    }
}

message! {
    /// `exchange_intermediary` transaction.
    struct ExchangeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_INTERMEDIARY_ID;

        offer:                  ExchangeOfferIntermediary,
        seed:                   u64,
        sender_signature:       &Signature,
        intermediary_signature: &Signature,
        data_info:              &str,
    }
}

#[derive(Clone, Debug)]
pub struct ExchangeIntermediaryWrapper {
    offer: ExchangeOfferIntermediary,
    seed: u64,
    sender_signature: Signature,
    intermediary_signature: Signature,
    data_info: String,
}

impl ExchangeIntermediaryWrapper {
    pub fn new(
        offer: ExchangeOfferIntermediary,
        seed: u64,
        sender_signature: &Signature,
        intermediary_signature: &Signature,
        data_info: &str,
    ) -> Self {
        ExchangeIntermediaryWrapper {
            offer: offer,
            seed: seed,
            sender_signature: *sender_signature,
            intermediary_signature: *intermediary_signature,
            data_info: data_info.to_string(),
        }
    }

    pub fn from_ptr<'a>(
        wrapper: *mut ExchangeIntermediaryWrapper,
    ) -> Result<&'a mut ExchangeIntermediaryWrapper, Error> {
        if wrapper.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "transactionx isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *wrapper })
    }

    pub fn unwrap(&self) -> ExchangeIntermediary {
        ExchangeIntermediary::new(
            self.offer.clone(),
            self.seed,
            &self.sender_signature,
            &self.intermediary_signature,
            self.data_info.as_str(),
            &SecretKey::zero(),
        )
    }
}
