use crypto::{PublicKey, Signature};

use assets::AssetBundle;
use transactions::components::service::SERVICE_ID;

use error::{Error, ErrorKind};
/// Transaction ID.
pub const TRANSFER_FEES_PAYER_ID: u16 = 201;

evo_encoding_struct! {
    struct TransferOffer {

        from:       &PublicKey,
        to:         &PublicKey,
        fees_payer: &PublicKey,

        amount:     u64,
        assets:     Vec<AssetBundle>,
        seed:       u64,
        memo:       &str,
    }
}

#[derive(Debug, Clone)]
pub struct TransferOfferWrapper {

    from:       PublicKey,
    to:         PublicKey,
    fees_payer: PublicKey,

    amount:     u64,
    assets:     Vec<AssetBundle>,
    seed:       u64,
    memo:       String,
}

impl TransferOfferWrapper {
    pub fn new(
        from: &PublicKey, 
        to: &PublicKey, 
        fees_payer: &PublicKey, 
        amount: u64, 
        seed: u64, 
        memo: &str
    ) -> Self {
        TransferOfferWrapper {
            from: *from,
            to: *to,
            fees_payer: *fees_payer,
            amount: amount,
            assets: Vec::new(),
            seed: seed,
            memo: memo.to_string(),
        }
    }

    pub fn from_ptr<'a>(
        builder: *mut TransferOfferWrapper,
    ) -> Result<&'a mut TransferOfferWrapper, Error> {
        if builder.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "Offer isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *builder })
    }

    pub fn add_asset(&mut self, asset: AssetBundle) {
        self.assets.push(asset);
    }

    pub fn unwrap(&self) -> TransferOffer {
        TransferOffer::new(
            &self.from,
            &self.to,
            &self.fees_payer,
            self.amount,
            self.assets.clone(),
            self.seed,
            self.memo.as_str()
        )
    }
}

evo_message! {
    /// `transfer` transaction.
    struct TransferWithFeesPayer {
        const TYPE = SERVICE_ID;
        const ID = TRANSFER_FEES_PAYER_ID;

        offer:                TransferOffer,
        fees_payer_signature: &Signature,
    }
}

#[derive(Clone, Debug)]
pub struct TransferWithFeesPayerWrapper {
    offer: TransferOffer,
    fees_payer_signature: Signature,
}

impl TransferWithFeesPayerWrapper {
    pub fn new(offer: TransferOffer, fees_payer_signature: &Signature) -> Self {
        TransferWithFeesPayerWrapper {
            offer: offer,
            fees_payer_signature: *fees_payer_signature
        }
    }

    pub fn from_ptr<'a>(
        wrapper: *mut TransferWithFeesPayerWrapper,
    ) -> Result<&'a mut TransferWithFeesPayerWrapper, Error> {
        if wrapper.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "transaction isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *wrapper })
    }

    pub fn unwrap(&self) -> TransferWithFeesPayer {
        TransferWithFeesPayer::new(
            self.offer.clone(),
            &self.fees_payer_signature
        )
    }
}