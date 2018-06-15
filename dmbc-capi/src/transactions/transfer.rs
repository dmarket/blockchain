use exonum::crypto::{PublicKey, SecretKey};

use assets::AssetBundle;
use transactions::components::service::SERVICE_ID;

use error::{Error, ErrorKind};

/// Transaction ID.
pub const TRANSFER_ID: u16 = 200;

message! {
    /// `transfer` transaction.
    struct Transfer {
        const TYPE = SERVICE_ID;
        const ID = TRANSFER_ID;

        from:      &PublicKey,
        to:        &PublicKey,
        amount:    u64,
        assets:    Vec<AssetBundle>,
        seed:      u64,
        data_info: &str,
    }
}

#[derive(Debug, Clone)]
pub struct TransferWrapper {
    from:      PublicKey,
    to:        PublicKey,
    amount:    u64,
    assets:    Vec<AssetBundle>,
    seed:      u64,
    data_info: String,
}

impl TransferWrapper {
    pub fn new(from: &PublicKey, to: &PublicKey, amount: u64, seed: u64, data_info: &str) -> Self {
        TransferWrapper {
            from: *from,
            to: *to,
            amount: amount,
            assets: Vec::new(),
            seed: seed,
            data_info: data_info.to_string()
        }
    }

    pub fn from_ptr<'a>(wrapper: *mut TransferWrapper) -> Result<&'a mut TransferWrapper, Error> {
        if wrapper.is_null() {
            return Err(
                Error::new(
                    ErrorKind::Text("wrapper isn't initialized".to_string())
                )
            );
        }
        Ok( unsafe { &mut *wrapper } )
    }

    pub fn add_asset(&mut self, asset: AssetBundle) {
        self.assets.push(asset);
    }

    pub fn unwrap(&self) -> Transfer {
        Transfer::new(
            &self.from,
            &self.to, 
            self.amount,
            self.assets.clone(), 
            self.seed,
            &self.data_info,
            &SecretKey::zero(),
        )
    }
}