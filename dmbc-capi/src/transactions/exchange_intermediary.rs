use crypto::{PublicKey, SecretKey, Signature};

use assets::AssetBundle;
use transactions::components::service::SERVICE_ID;
use transactions::components::Intermediary;

use error::{Error, ErrorKind};

/// Transaction ID.
pub const EXCHANGE_INTERMEDIARY_ID: u16 = 602;

evo_encoding_struct! {
    struct ExchangeOfferIntermediary {
        intermediary:     Intermediary,

        sender:           &PublicKey,
        sender_assets:    Vec<AssetBundle>,
        sender_value:     u64,

        recipient:        &PublicKey,
        recipient_assets: Vec<AssetBundle>,

        fee_strategy:     u8,
        seed:             u64,
        memo:        &str,
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
    seed: u64,
    memo: String
}

impl ExchangeOfferIntermediaryWrapper {
    pub fn new(
        intermediary: Intermediary,
        sender: &PublicKey,
        sender_value: u64,
        recipient: &PublicKey,
        fee_strategy: u8,
        seed: u64,
        memo: &str,
    ) -> Self {
        ExchangeOfferIntermediaryWrapper {
            intermediary: intermediary,

            sender: *sender,
            sender_assets: Vec::new(),
            sender_value: sender_value,

            recipient: *recipient,
            recipient_assets: Vec::new(),
            fee_strategy: fee_strategy,
            seed: seed,
            memo: memo.to_string()
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
            self.seed,
            &self.memo.as_str()
        )
    }
}

evo_message! {
    /// `exchange_intermediary` transaction.
    struct ExchangeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_INTERMEDIARY_ID;

        offer:                  ExchangeOfferIntermediary,
        sender_signature:       &Signature,
        intermediary_signature: &Signature,
    }
}

#[derive(Clone, Debug)]
pub struct ExchangeIntermediaryWrapper {
    offer: ExchangeOfferIntermediary,
    sender_signature: Signature,
    intermediary_signature: Signature,
}

impl ExchangeIntermediaryWrapper {
    pub fn new(
        offer: ExchangeOfferIntermediary,
        sender_signature: &Signature,
        intermediary_signature: &Signature,
    ) -> Self {
        ExchangeIntermediaryWrapper {
            offer: offer,
            sender_signature: *sender_signature,
            intermediary_signature: *intermediary_signature,
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
            &self.sender_signature,
            &self.intermediary_signature,
            &SecretKey::zero(),
        )
    }
}
