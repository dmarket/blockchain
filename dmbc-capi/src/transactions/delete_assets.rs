use crypto::{PublicKey, SecretKey};

use assets::AssetBundle;
use transactions::components::service::SERVICE_ID;

use error::{Error, ErrorKind};

/// Transaction ID.
pub const DELETE_ASSETS_ID: u16 = 400;

evo_message! {
    /// `delete_assets` transaction.
    struct DeleteAssets {
        const TYPE = SERVICE_ID;
        const ID = DELETE_ASSETS_ID;

        pub_key:     &PublicKey,
        assets:      Vec<AssetBundle>,
        seed:        u64,
    }
}

#[derive(Clone, Debug)]
pub struct DeleteAssetsWrapper {
    public_key: PublicKey,
    assets: Vec<AssetBundle>,
    seed: u64,
}

impl DeleteAssetsWrapper {
    pub fn new(public_key: &PublicKey, seed: u64) -> Self {
        DeleteAssetsWrapper {
            public_key: *public_key,
            assets: Vec::new(),
            seed: seed,
        }
    }

    pub fn from_ptr<'a>(
        wrapper: *mut DeleteAssetsWrapper,
    ) -> Result<&'a mut DeleteAssetsWrapper, Error> {
        if wrapper.is_null() {
            return Err(Error::new(ErrorKind::Text(
                "wrapper isn't initialized".to_string(),
            )));
        }
        Ok(unsafe { &mut *wrapper })
    }

    pub fn add_asset(&mut self, asset: AssetBundle) {
        self.assets.push(asset);
    }

    pub fn unwrap(&self) -> DeleteAssets {
        DeleteAssets::new(
            &self.public_key,
            self.assets.clone(),
            self.seed,
            &SecretKey::zero(),
        )
    }
}
