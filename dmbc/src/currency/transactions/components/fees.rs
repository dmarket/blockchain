//! Transaction fees.

use std::collections::HashMap;

use exonum::crypto::PublicKey;
use exonum::storage::{Fork, Snapshot};

use currency::Service;
use currency::assets;
use currency::assets::{AssetBundle, MetaAsset, TradeAsset};
use currency::error::Error;
use currency::configuration::Configuration;
use currency::wallet;
use currency::wallet::Wallet;

/// For exchange transactions, determines who shall pay the fees.
#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum FeeStrategy {
    /// Recipient pays.
    Recipient = 1,
    /// Sender pays.
    Sender = 2,
    /// Recipient and sender share paying the fee.
    RecipientAndSender = 3,
    /// Intermediary pays.
    Intermediary = 4,
}

impl FeeStrategy {
    /// Try converting from an u8. To be replaced when the `TryFrom` trait
    /// is stabilised.
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

/// Transaction fees.
pub struct Fees {
    /// Fixed fee to pay for processing the transaction.
    pub to_genesis: u64,
    /// Fees to be paid to third parties, e.g. creators of assets exchanged
    /// in a thransaction.
    pub to_third_party: HashMap<PublicKey, u64>,
}

impl Fees {
    /// Create fees with specified data.
    pub fn new<I>(to_genesis: u64, to_third_party: I) -> Self
    where
        I: IntoIterator<Item = (PublicKey, u64)>,
    {
        Fees {
            to_genesis,
            to_third_party: to_third_party.into_iter().collect(),
        }
    }

    /// Create `Fees` for an `add_assets` transaction.
    pub fn new_add_assets<S, I>(view: S, assets: I) -> Result<Fees, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item = MetaAsset>,
    {
        let fees_config = Configuration::extract(view.as_ref()).fees();

        let to_genesis = fees_config.add_asset();

        let per_asset = fees_config.per_add_asset();
        let assets_fee = assets
            .into_iter()
            .map(|meta| meta.amount() * per_asset)
            .sum();
        let to_third_party = Some((Service::genesis_wallet(), assets_fee))
            .into_iter()
            .collect();

        let fees = Fees {
            to_genesis,
            to_third_party,
        };

        Ok(fees)
    }

    /// Create `Fees` for `trade` transactions.
    pub fn new_trade<'a, S, I>(view: S, assets: I) -> Result<Fees, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item = &'a TradeAsset>,
        <I as IntoIterator>::IntoIter: Clone,
    {
        let view = view.as_ref();
        let assets = assets.into_iter();
        let assets_price: u64 = assets.clone().map(|ta| ta.total_price()).sum();
        let mut to_third_party = HashMap::new();

        for asset in assets {
            let info = assets::Schema(view)
                .fetch(&asset.id())
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

    /// Create `Fees` for `exchange` transactions.
    pub fn new_exchange<S, I>(view: S, assets: I) -> Result<Self, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item = AssetBundle>,
    {
        let view = view.as_ref();
        let mut to_third_party = HashMap::new();

        for asset in assets {
            let info = assets::Schema(view)
                .fetch(&asset.id())
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

    /// Create `Fees` for `transfer` transactions.
    pub fn new_transfer<S, I>(view: S, assets: I) -> Result<Self, Error>
    where
        S: AsRef<Snapshot>,
        I: IntoIterator<Item = AssetBundle>,
    {
        let view = view.as_ref();
        let mut to_third_party = HashMap::new();

        for asset in assets {
            let info = assets::Schema(view)
                .fetch(&asset.id())
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

    /// Total amount of fees that must be paid in order for the transaction
    /// to succeed.
    pub fn total(&self) -> u64 {
        self.to_genesis + self.to_third_party_total()
    }

    /// Total amound that needs to be paid to third party wallets.
    pub fn to_third_party_total(&self) -> u64 {
        self.to_third_party.values().sum()
    }


    pub fn total_for_wallet(&self, pub_key: &PublicKey ) -> u64 {
        self.to_third_party
            .iter()
            .filter_map(|(key, fee)| if key != pub_key {Some(fee)} else {None} )
            .sum()
    }

    /// Add a new fee to the list of third party payments.
    pub fn add_fee(&mut self, key: &PublicKey, fee: u64) {
        self.to_third_party
            .entry(*key)
            .and_modify(|prev_fee| {
                *prev_fee += fee;
            })
            .or_insert(fee);
    }

    /// Collect fees to genesis wallet.
    ///
    /// # Errors
    /// Returns `InsufficientFunds` if the payer is unable to pay the fees.
    pub fn collect_to_genesis(
        &self,
        payer: &mut Wallet,
        genesis: &mut Wallet,
    ) -> Result<(), Error> {
        wallet::move_coins(payer, genesis, self.to_genesis)?;

        Ok(())
    }

    /// Split the fee payment to genesis wallet between two wallets.
    pub fn collect_to_genesis_2(
        &self,
        payer_1: &mut Wallet,
        payer_2: &mut Wallet,
        genesis: &mut Wallet,
    ) -> Result<(), Error> {
        wallet::move_coins(payer_1, genesis, self.to_genesis / 2)?;
        wallet::move_coins(payer_2, genesis, self.to_genesis / 2)?;

        Ok(())
    }

    /// Collect fees to third party wallets.
    /// 
    /// Returns a list of wallets modified by fee withdrawal.
    /// This list must usually not be committed or discarded before
    /// the transaction has otherwise successfully executed.
    ///
    /// # Errors
    /// Returns `InsufficientFunds` if the payer is unable to pay the fees.
    pub fn collect_to_third_party(
        &self,
        view: &Fork,
        payer_key: &PublicKey,
    ) -> Result<HashMap<PublicKey, Wallet>, Error> {
        let mut payer = wallet::Schema(&*view).fetch(&payer_key);

        let mut updated_wallets = self.to_third_party
            .iter()
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

    /// Split fees to third party wallets between two payers.
    pub fn collect_to_third_party_2(
        &self,
        view: &mut Fork,
        payer_key_1: &PublicKey,
        payer_key_2: &PublicKey,
    ) -> Result<HashMap<PublicKey, Wallet>, Error> {
        let mut payer_1 = wallet::Schema(&*view).fetch(&payer_key_1);
        let mut payer_2 = wallet::Schema(&*view).fetch(&payer_key_2);

        let mut updated_wallets = self.to_third_party
            .iter()
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
