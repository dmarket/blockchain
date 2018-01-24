use exonum::storage::Fork;
use exonum::crypto::PublicKey;

use std::collections::BTreeMap;

use service::wallet::Wallet;
use service::asset::{Asset, MetaAsset, TradeAsset};
use service::configuration::Configuration;
use service::schema::asset::AssetSchema;
use service::schema::wallet::WalletSchema;

#[derive(PartialEq)]
pub enum FeeStrategy {
    Recipient = 1,
    Sender = 2,
    RecipientAndSender = 3,
    Intermediary = 4,
}

pub struct TxFees {
    transaction_fee: u64,
    assets_fees: BTreeMap<Wallet, u64>,
}

impl FeeStrategy {
    pub fn from_u8(value: u8) -> Option<FeeStrategy> {
        match value {
            1 => Some(FeeStrategy::Recipient),
            2 => Some(FeeStrategy::Sender),
            3 => Some(FeeStrategy::RecipientAndSender),
            4 => Some(FeeStrategy::Intermediary),
            _ => None,
        }
    }
}

impl TxFees {
    pub fn new(tx_fee: u64, fees: BTreeMap<Wallet, u64>) -> Self {
        TxFees {
            transaction_fee: tx_fee,
            assets_fees: fees,
        }
    }

    pub fn transaction_fee(&self) -> u64 {
        self.transaction_fee
    }

    pub fn amount(&self) -> u64 {
        self.transaction_fee() + self.assets_fees_total()
    }

    pub fn assets_fees(&self) -> BTreeMap<Wallet, u64> {
        self.assets_fees.clone()
    }

    pub fn assets_fees_total(&self) -> u64 {
        self.assets_fees.iter().fold(0, |acc, asset| acc + asset.1)
    }
}

pub fn calculate_fees_for_trade(view: &mut Fork, assets: Vec<TradeAsset>) -> TxFees {
    let mut assets_fees = BTreeMap::new();
    let fee_ratio = |price: u64, ratio: u64| (price as f64 / ratio as f64).round() as u64;

    for asset in assets {
        if let Some(info) = AssetSchema::map(view, |mut schema| schema.info(&asset.id())) {
            let trade_fee = info.fees().trade();
            let fee = trade_fee.tax() + fee_ratio(asset.total_price(), trade_fee.ratio());

            let creator = WalletSchema::map(view, |mut schema| schema.wallet(info.creator()));
            *assets_fees.entry(creator).or_insert(0) += fee;
        }
    }

    let tx_fee = Configuration::extract(view).fees().trade();
    TxFees::new(tx_fee, assets_fees)
}

pub fn calculate_fees_for_exchange(view: &mut Fork, assets: Vec<Asset>) -> TxFees {
    let mut assets_fees = BTreeMap::new();

    let fee_ratio = |count: u32, coef: u64| (count as f64 / coef as f64).round() as u64;
    for asset in assets {
        if let Some(info) = AssetSchema::map(view, |mut schema| schema.info(&asset.id())) {
            let exchange_fee = info.fees().exchange();
            let fee = exchange_fee.tax() + fee_ratio(asset.amount(), exchange_fee.ratio());

            let creator = WalletSchema::map(view, |mut schema| schema.wallet(info.creator()));
            *assets_fees.entry(creator).or_insert(0) += fee;
        }
    }

    let tx_fee = Configuration::extract(view).fees().exchange();
    TxFees::new(tx_fee, assets_fees)
}

pub fn calculate_fees_for_transfer(view: &mut Fork, assets: Vec<Asset>) -> TxFees {
    let mut assets_fees = BTreeMap::new();
    for asset in assets {
        if let Some(info) = AssetSchema::map(view, |mut schema| schema.info(&asset.id())) {
            let fee = info.fees().transfer().tax();

            let creator = WalletSchema::map(view, |mut schema| schema.wallet(info.creator()));
            *assets_fees.entry(creator).or_insert(0) += fee;
        }
    }

    let tx_fee = Configuration::extract(view).fees().transfer();
    TxFees::new(tx_fee, assets_fees)
}

pub fn calculate_fees_for_add_assets(
    view: &mut Fork,
    assets: Vec<MetaAsset>,
    creator_key: &PublicKey,
) -> TxFees {
    let mut assets_fees = BTreeMap::new();
    let configuration = Configuration::extract(view);
    let creator = WalletSchema::map(view, |mut schema| schema.wallet(creator_key));

    let count = assets
        .iter()
        .fold(0, |acc, asset| acc + asset.amount() as u64);

    let tx_fee = configuration.fees().add_asset();
    *assets_fees.entry(creator).or_insert(0) += configuration.fees().per_add_asset() * count;
    TxFees::new(tx_fee, assets_fees)
}

pub fn calculate_fees_for_del_assets(view: &mut Fork, _assets: Vec<Asset>) -> TxFees {
    let tx_fee = Configuration::extract(view).fees().del_asset();
    TxFees::new(tx_fee, BTreeMap::new())
}
