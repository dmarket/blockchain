use std::collections::HashMap;

use exonum::crypto::PublicKey;
use exonum::storage::{Snapshot, Fork};

use currency::assets;
use currency::assets::{AssetBundle, TradeAsset, MetaAsset};
use currency::error::Error;
use currency::configuration::Configuration;
use currency::wallet;
use currency::wallet::Wallet;

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
    pub to_genesis: u64,
    pub to_third_party: HashMap<PublicKey, u64>,
}

impl Fees {
    pub fn new<I>(to_genesis: u64, to_third_party: I) -> Self
    where
        I: IntoIterator<Item=(PublicKey, u64)>
    {
        Fees {
            to_genesis,
            to_third_party: to_third_party.into_iter().collect(),
        }
    }

    pub fn new_add_assets<S, I>(view: S, creator: PublicKey, assets: I)
        -> Result<Fees, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item=MetaAsset>,
    {
        let fees_config = Configuration::extract(view.as_ref()).fees();

        let to_genesis = fees_config.add_asset();

        let per_asset = fees_config.per_add_asset();
        let assets_fee = assets
            .into_iter()
            .map(|meta| meta.amount() * per_asset)
            .sum();
        let to_third_party = Some((creator, assets_fee)).into_iter().collect();

        let fees = Fees {
            to_genesis,
            to_third_party,
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
        let mut to_third_party = HashMap::new();

        for asset in assets {
            let info = assets::Schema(view).fetch(&asset.id())
                .ok_or_else(|| Error::AssetNotFound)?;

            let fee = info.fees().trade().for_price(assets_price);
            to_third_party
                .entry(*info.creator())
                .and_modify(|prev_fee| {
                    *prev_fee += fee;
                })
                .or_insert(fee);
        }

        let to_genesis = Configuration::extract(view).fees().trade();

        let fees = Fees {
            to_genesis,
            to_third_party,
        };

        Ok(fees)
    }

    pub fn new_exchange<S, I>(view: S, assets: I) -> Result<Self, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item=AssetBundle>,
    {
        let view = view.as_ref();
        let mut to_third_party = HashMap::new();

        for asset in assets {
            let info = assets::Schema(view).fetch(&asset.id())
                    .ok_or_else(|| Error::AssetNotFound)?;

            let fee = info.fees().exchange().tax();
            to_third_party
                .entry(*info.creator())
                .and_modify(|prev_fee| {
                    *prev_fee += fee;
                })
                .or_insert(fee);
        }

        let to_genesis = Configuration::extract(view).fees().exchange();

        let fees = Fees {
            to_genesis,
            to_third_party,
        };

        Ok(fees)
    }

    pub fn new_transfer<S, I>(view: S, assets: I) -> Result<Self, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item=AssetBundle>,
    {
        let view = view.as_ref();
        let mut to_third_party = HashMap::new();

        for asset in assets {
            let info = assets::Schema(view).fetch(&asset.id())
                    .ok_or_else(|| Error::AssetNotFound)?;

            let fee = info.fees().transfer().tax();
            to_third_party
                .entry(*info.creator())
                .and_modify(|prev_fee| {
                    *prev_fee += fee;
                })
                .or_insert(fee);
        }

        let to_genesis = Configuration::extract(view).fees().transfer();

        let fees = Fees {
            to_genesis,
            to_third_party,
        };

        Ok(fees)
    }

    pub fn total(&self) -> u64 {
        self.to_genesis + self.to_third_party_total()
    }

    pub fn to_third_party_total(&self) -> u64 {
        self.to_third_party.values().sum()
    }

    pub fn add_fee(&mut self, key: &PublicKey, fee: u64) {
        self.to_third_party
            .entry(*key)
            .and_modify(|prev_fee| {
                *prev_fee += fee;
            })
            .or_insert(fee);
    }

    pub fn collect_to_genesis(
        &mut self,
        payer: &mut Wallet,
        genesis: &mut Wallet,
    ) -> Result<(), Error> {
        wallet::move_coins(payer, genesis, self.to_genesis)?;

        Ok(())
    }

    pub fn collect_to_genesis_2(
        &mut self,
        payer_1: &mut Wallet,
        payer_2: &mut Wallet,
        genesis: &mut Wallet,
    ) -> Result<(), Error> {
        wallet::move_coins(payer_1, genesis, self.to_genesis / 2)?;
        wallet::move_coins(payer_2, genesis, self.to_genesis / 2)?;

        Ok(())
    }

    pub fn collect_to_third_party(
        &mut self,
        view: &Fork,
        payer_key: &PublicKey,
    ) -> Result<HashMap<PublicKey, Wallet>, Error> {
        let mut payer = wallet::Schema(&*view).fetch(&payer_key);

        let mut updated_wallets = self.to_third_party.iter()
            .filter(|&(key, _)| key != payer_key)
            .map(|(key, fee)| {
                let mut wallet = wallet::Schema(&*view).fetch(key);

                wallet::move_coins(&mut payer, &mut wallet, *fee)?;

                Ok((*key, wallet))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        updated_wallets.entry(*payer_key).or_insert(payer);

        Ok(updated_wallets)
    }

    pub fn collect_to_third_party_2(
        &mut self,
        view: &mut Fork,
        payer_key_1: &PublicKey,
        payer_key_2: &PublicKey,
    ) -> Result<HashMap<PublicKey, Wallet>, Error> {
        let mut payer_1 = wallet::Schema(&*view).fetch(&payer_key_1);
        let mut payer_2 = wallet::Schema(&*view).fetch(&payer_key_2);

        let mut updated_wallets = self.to_third_party.iter()
            .map(|(key, fee)| {
                let mut wallet = wallet::Schema(&*view).fetch(&key);
                if key != payer_key_1 {
                    wallet::move_coins(&mut payer_1, &mut wallet, fee / 2)?;
                }
                if key != payer_key_2 {
                    wallet::move_coins(&mut payer_2, &mut wallet, fee / 2)?;
                }
                Ok((*key, wallet))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        updated_wallets.entry(*payer_key_1).or_insert(payer_1);
        updated_wallets.entry(*payer_key_2).or_insert(payer_2);

        Ok(updated_wallets)
    }
}

