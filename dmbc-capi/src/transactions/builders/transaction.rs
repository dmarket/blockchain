use exonum::crypto::{PublicKey, SecretKey};

use assets::{Fees, MetaAsset, AssetBundle};
use transactions::add_assets::AddAssets;
use transactions::delete_assets::DeleteAssets;

use error::{Error, ErrorKind};

pub struct Builder {
    network_id: u8,
    protocol_version: u8,
    service_id: u16,
}

#[derive(Debug, Clone)]
struct TransactionMetadata {
    network_id: u8,
    protocol_version: u8,
    service_id: u16,
}

impl From<Builder> for TransactionMetadata {
    fn from(b: Builder) -> Self {
        TransactionMetadata {
            network_id: b.network_id,
            protocol_version: b.protocol_version,
            service_id: b.service_id,
        }
    }
}

impl Builder {
    pub fn new(network_id: u8, protocol_version: u8, service_id: u16) -> Self {
        Builder {
            network_id,
            protocol_version,
            service_id,
        }
    }

    pub fn tx_add_asset(self) -> AddAssetBuilder {
        AddAssetBuilder::new(self.into())
    }

    pub fn tx_delete_asset(self) -> DelAssetBuilder {
        DelAssetBuilder::new(self.into())
    }
}

#[derive(Debug, Clone)]
pub struct AddAssetBuilder {
    meta: TransactionMetadata,
    public_key: Option<PublicKey>,
    assets: Vec<MetaAsset>,
    seed: u64,
}

impl AddAssetBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        AddAssetBuilder {
            meta,
            public_key: None,
            assets: Vec::new(),
            seed: 0,
        }
    }

    pub fn public_key(&mut self, public_key: PublicKey) {
        self.public_key = Some(public_key);
    }

    pub fn add_asset(&mut self, name: &str, count: u64, fees: Fees, receiver_key: &PublicKey) {
        let asset = MetaAsset::new(&receiver_key, name, count, fees);
        self.add_asset_value(asset);
    }

    pub fn add_asset_value(&mut self, asset: MetaAsset) {
        self.assets.push(asset);
    }

    pub fn seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    pub fn build(&self) -> Result<AddAssets, Error> {
        self.validate()?;
        Ok(
            AddAssets::new(
                &self.public_key.unwrap(),
                self.assets.clone(),
                self.seed,
                &SecretKey::zero(),
            )
        )
    }

    fn validate(&self) -> Result<(), Error> {
        match self.public_key {
            Some(_) => Ok(()),
            None => Err(Error::new(ErrorKind::Text("Public key isn't set".to_string()))),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DelAssetBuilder {
    meta: TransactionMetadata,
    public_key: Option<PublicKey>,
    assets: Vec<AssetBundle>,
    seed: u64,
}

impl DelAssetBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        DelAssetBuilder {
            meta,
            public_key: None,
            assets: Vec::new(),
            seed: 0,
        }
    }

    pub fn public_key(&mut self, public_key: PublicKey) {
        self.public_key = Some(public_key);
    }

    pub fn add_asset(&mut self, asset: AssetBundle) {
        self.assets.push(asset);
    }

    pub fn seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    pub fn build(&self) -> Result<DeleteAssets, Error> {
        self.validate()?;
        Ok(
            DeleteAssets::new(
                &self.public_key.unwrap(),
                self.assets.clone(),
                self.seed,
                &SecretKey::zero(),
            )
        )
    }

    fn validate(&self) -> Result<(), Error> {
        match self.public_key {
            Some(_) => Ok(()),
            None => Err(Error::new(ErrorKind::Text("Public key isn't set".to_string()))),
        }
    }
}