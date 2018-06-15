use exonum::crypto::{PublicKey, SecretKey};

use assets::MetaAsset;
use transactions::components::service::SERVICE_ID;

use error::{Error, ErrorKind};

pub const ADD_ASSETS_ID: u16 = 300;

message!{
    /// `add_assets` transaction.
    struct AddAssets {
        const TYPE = SERVICE_ID;
        const ID = ADD_ASSETS_ID;

        pub_key:     &PublicKey,
        meta_assets: Vec<MetaAsset>,
        seed:        u64,
    }
}

#[derive(Clone, Debug)]
pub struct AddAssetWrapper {
    pub_key: PublicKey,
    meta_assets: Vec<MetaAsset>,
    seed: u64,
}

impl AddAssetWrapper {
    pub fn new(public_key: &PublicKey, seed: u64) -> Self {
        AddAssetWrapper {
            pub_key: *public_key,
            meta_assets: Vec::new(),
            seed: seed,
        }
    }

    pub fn from_ptr<'a>(wrapper: *mut AddAssetWrapper) -> Result<&'a mut AddAssetWrapper, Error> {
        if wrapper.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "wrapper isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *wrapper })
    }

    pub fn add_asset(&mut self, asset: MetaAsset) {
        self.meta_assets.push(asset);
    }

    pub fn unwrap(&self) -> AddAssets {
        AddAssets::new(
            &self.pub_key,
            self.meta_assets.clone(),
            self.seed,
            &SecretKey::zero(),
        )
    }
}
