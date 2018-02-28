use std::collections::HashMap;

use exonum::crypto::PublicKey;
use exonum::storage::Snapshot;

use currency::asset;
use currency::asset::{AssetBundle, TradeAsset, MetaAsset};
use currency::error::Error;
use currency::configuration::Configuration;

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum FeeStrategy {
    Recipient = 1,
    Sender = 2,
    RecipientAndSender = 3,
    Intermediary = 4,
}

impl FeeStrategy {
    pub fn try_from(value: u8) -> Option<Self> {
        match value {
            1 => Some(FeeStrategy::Recipient),
            2 => Some(FeeStrategy::Sender),
            3 => Some(FeeStrategy::RecipientAndSender),
            4 => Some(FeeStrategy::Intermediary),
            _ => None,
        }
    }
}

pub struct Fees {
    for_transaction: u64,
    for_assets: HashMap<PublicKey, u64>,
}

impl Fees {
    pub fn new<I>(for_transaction: u64, for_assets: I) -> Self
    where
        I: IntoIterator<Item=(PublicKey, u64)>
    {
        Fees {
            for_transaction,
            for_assets: for_assets.into_iter().collect(),
        }
    }

    pub fn new_add_assets<S, I>(view: S, creator: PublicKey, assets: I)
        -> Result<Fees, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item=MetaAsset>,
    {
        let fees_config = Configuration::extract(view.as_ref()).fees();

        let for_transaction = fees_config.add_asset();

        let per_asset = fees_config.per_add_asset();
        let assets_fee = assets
            .into_iter()
            .map(|meta| meta.amount() * per_asset)
            .sum();
        let for_assets = Some((creator, assets_fee)).into_iter().collect();

        let fees = Fees {
            for_transaction,
            for_assets,
        };

        Ok(fees)
    }

    pub fn new_trade<'a, S, I>(view: S, assets: I) -> Result<Fees, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item=&'a TradeAsset>,
        <I as IntoIterator>::IntoIter: Clone,
    {
        let view = view.as_ref();
        let assets = assets.into_iter();
        let assets_price : u64 = assets.clone().map(|ta| ta.total_price()).sum();
        let for_assets = assets
            .map(|ta| {
                let info = asset::Schema(view).fetch(&ta.id())
                    .ok_or_else(|| Error::AssetNotFound)?;

                let fee = info.fees().trade().for_price(assets_price);

                Ok((*info.creator(), fee))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let for_transaction = Configuration::extract(view).fees().trade();

        let fees = Fees {
            for_transaction,
            for_assets,
        };

        Ok(fees)
    }

    pub fn new_exchange<S, I>(view: S, assets: I) -> Result<Self, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item=AssetBundle>,
    {
        let view = view.as_ref();
        let for_assets = assets.into_iter()
            .map(|asset| {
                let info = asset::Schema(view).fetch(&asset.id())
                    .ok_or_else(|| Error::AssetNotFound)?;

                let fee = info.fees().exchange().tax();

                Ok((*info.creator(), fee))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let for_transaction = Configuration::extract(view).fees().exchange();

        let fees = Fees {
            for_transaction,
            for_assets,
        };

        Ok(fees)
    }

    pub fn new_transfer<S, I>(view: S, assets: I) -> Result<Self, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item=AssetBundle>,
    {
        let view = view.as_ref();
        let for_assets = assets.into_iter()
            .map(|asset| {
                let info = asset::Schema(view).fetch(&asset.id())
                    .ok_or_else(|| Error::AssetNotFound)?;

                let fee = info.fees().transfer().tax();

                Ok((*info.creator(), fee))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let for_transaction = Configuration::extract(view).fees().transfer();

        let fees = Fees {
            for_transaction,
            for_assets,
        };

        Ok(fees)
    }

    pub fn total(&self) -> u64 {
        self.for_transaction() + self.for_assets_total()
    }

    pub fn for_transaction(&self) -> u64 {
        self.for_transaction
    }

    pub fn for_assets(&self) -> HashMap<PublicKey, u64> {
        self.for_assets.clone()
    }

    pub fn for_assets_total(&self) -> u64 {
        self.for_assets.values().sum()
    }
}

