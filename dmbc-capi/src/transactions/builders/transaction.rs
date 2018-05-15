use exonum::crypto::{PublicKey, SecretKey};

use assets::{Fees, MetaAsset, AssetBundle};
use transactions::add_assets::AddAssets;
use transactions::delete_assets::DeleteAssets;
use transactions::transfer::Transfer;

use error::{Error, ErrorKind};

pub struct Builder {
    network_id: u8,
    protocol_version: u8,
    service_id: u16,
}

pub trait TransactionBuilder<T> {
    fn build(&self) -> Result<T, Error>;
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

    pub fn tx_add_assets(self) -> AddAssetBuilder {
        AddAssetBuilder::new(self.into())
    }

    pub fn tx_delete_assets(self) -> DelAssetBuilder {
        DelAssetBuilder::new(self.into())
    }

    pub fn tx_transfer(self) -> TransferBuilder {
        TransferBuilder::new(self.into())
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

    fn validate(&self) -> Result<(), Error> {
        match self.public_key {
            Some(_) => Ok(()),
            None => Err(Error::new(ErrorKind::Text("Public key isn't set".to_string()))),
        }
    }
}

impl TransactionBuilder<AddAssets> for AddAssetBuilder {
    fn build(&self) -> Result<AddAssets, Error> {
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

    fn validate(&self) -> Result<(), Error> {
        match self.public_key {
            Some(_) => Ok(()),
            None => Err(Error::new(ErrorKind::Text("Public key isn't set".to_string()))),
        }
    }
}

impl TransactionBuilder<DeleteAssets> for DelAssetBuilder {
    fn build(&self) -> Result<DeleteAssets, Error> {
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
}

#[derive(Clone, Debug)]
pub struct TransferBuilder {
    meta: TransactionMetadata,
    from: Option<PublicKey>,
    to: Option<PublicKey>,
    amount: u64,
    assets: Vec<AssetBundle>,
    seed: u64,
    data_info: Option<String>,
}

impl TransferBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TransferBuilder {
            meta,
            from: None,
            to: None,
            amount: 0,
            assets: Vec::new(),
            seed: 0,
            data_info: None,
        }
    }

    pub fn from(&mut self, public_key: PublicKey) {
        self.from = Some(public_key);
    }

    pub fn to(&mut self, public_key: PublicKey) {
        self.to = Some(public_key);
    }

    pub fn amount(&mut self, amount: u64) {
        self.amount = amount
    }

    pub fn add_asset(&mut self, asset: AssetBundle) {
        self.assets.push(asset);
    }

    pub fn seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    pub fn data_info(&mut self, data_info: &str) {
        self.data_info = Some(data_info.to_string());
    }

    fn verify(&self) -> Result<(), Error> {
        match (self.from, self.to) {
            (Some(_), Some(_)) => Ok(()),
            (None, _) => Err(Error::new(ErrorKind::Text("`from` public key isn't set".to_string()))),
            (_, None) => Err(Error::new(ErrorKind::Text("`to` public key isn't set".to_string()))),
        }
    }
}

impl TransactionBuilder<Transfer> for TransferBuilder {
    fn build(&self) -> Result<Transfer, Error> {
        self.verify()?;

        Ok(
            Transfer::new(
                &self.from.unwrap(),
                &self.to.unwrap(),
                self.amount,
                self.assets.clone(),
                self.seed,
                &self.data_info.clone().unwrap_or_default(),
                &SecretKey::zero(),
            )
        )
    }
}