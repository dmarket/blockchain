extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use std::collections::BTreeMap;

use service::CurrencyService;
use service::asset::{Asset, TradeAsset};
use service::wallet::Wallet;
use service::configuration::Configuration;

use super::{SERVICE_ID, TX_TRADE_ASK_ASSETS_ID};
use super::schema::asset::AssetSchema;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

encoding_struct! {
    struct TradeAskOffer {
        const SIZE = 40;

        field seller: &PublicKey        [00 => 32]
        field assets: Vec<TradeAsset>   [32 => 40]
    }
}

message! {
    struct TxTradeAsk {
        const TYPE = SERVICE_ID;
        const ID = TX_TRADE_ASK_ASSETS_ID;
        const SIZE = 120;

        field buyer:              &PublicKey    [00 => 32]
        field offer:              TradeAskOffer [32 => 40]
        field seed:               u64           [40 => 48]
        field seller_signature:   &Signature    [48 => 112]
        field data_info:          &str          [112 => 120]
    }
}

pub struct TradeAskFee {
    transaction_fee: u64,
    assets_fees: BTreeMap<Wallet, u64>,
}

impl TxTradeAsk {
    pub fn get_fee(&self, view: &mut Fork) -> TradeAskFee {
        let mut assets_fees = BTreeMap::new();
        let fee_ratio = |price: u64, ratio: u64| (price as f64 / ratio as f64).round() as u64;

        for asset in self.offer().assets() {
            if let Some(info) = AssetSchema::map(view, |mut schema| schema.info(&asset.id())) {
                let trade_fee = info.fees().trade();
                let fee = trade_fee.tax() + fee_ratio(asset.total_price(), trade_fee.ratio());

                let creator = WalletSchema::map(view, |mut schema| schema.wallet(info.creator()));
                *assets_fees.entry(creator).or_insert(0) += fee;
            }
        }

        let tx_fee = Configuration::extract(view).fees().trade();
        TradeAskFee::new(tx_fee, assets_fees)
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        let (mut platform, mut buyer, mut seller) = WalletSchema::map(view, |mut schema| {
            let platform_key = CurrencyService::get_platfrom_wallet();
            (
                schema.wallet(&platform_key),
                schema.wallet(self.buyer()),
                schema.wallet(self.offer().seller()),
            )
        });

        let fee = self.get_fee(view);

        // Fail if not enough coins on seller balance
        if seller.balance() < fee.transaction_fee() {
            return TxStatus::Fail;
        }

        // Take coins for executing transaction
        seller.decrease(fee.transaction_fee());
        // put fee to platfrom wallet
        platform.increase(fee.transaction_fee());
        // store data
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(&seller.pub_key(), seller.clone());
            schema.wallets().put(&platform.pub_key(), platform.clone());
        });

        // initial point for db rollback, in case if transaction has failed
        view.checkpoint();

        // convert trade assets to assets stored on the blockchain
        let trade_assets = self.offer().assets();
        let assets = trade_assets
            .iter()
            .map(|x| x.clone().into())
            .collect::<Vec<Asset>>();
        println!("Buyer {:?} => Seller {:?}", buyer, seller);

        let offer_price = self.offer().total_price();
        let seller_assets_ok = seller.is_assets_in_wallet(&assets);
        let seller_balance_ok = seller.balance() >= fee.assets_fees_total();
        let buyer_balance_ok = buyer.balance() >= offer_price;

        if !seller_assets_ok || !seller_balance_ok || !buyer_balance_ok {
            view.rollback();
            return TxStatus::Fail;
        }

        println!("--   Ask/Bid transaction   --");
        println!("Seller's balance before transaction : {:?}", seller);
        println!("Buyer's balance before transaction : {:?}", buyer);

        seller.del_assets(&assets);
        seller.increase(offer_price);
        buyer.add_assets(&assets);
        buyer.decrease(offer_price);

        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(buyer.pub_key(), buyer.clone());
            schema.wallets().put(seller.pub_key(), seller.clone());
        });

        for (mut creator, fee) in fee.assets_fees() {
            println!("\tCreator {:?} will receive {}", creator.pub_key(), fee);
            seller.decrease(fee);
            creator.increase(fee);
            WalletSchema::map(view, |mut schema| {
                schema.wallets().put(creator.pub_key(), creator.clone());
                schema.wallets().put(seller.pub_key(), seller.clone());
            });
        }

        TxStatus::Success
    }
}

impl Transaction for TxTradeAsk {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }
        let keys_ok = *self.buyer() != *self.offer().seller();
        let verify_seller_ok = crypto::verify(
            self.seller_signature(),
            &self.offer().raw,
            self.offer().seller(),
        );
        let verify_buyer_ok = self.verify_signature(self.buyer());

        keys_ok && verify_seller_ok && verify_buyer_ok
    }

    fn execute(&self, view: &mut Fork) {
        let tx_status = self.process(view);
        TxStatusSchema::map(view, |mut schema| {
            schema.set_status(&self.hash(), tx_status)
        });
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
        })
    }
}

impl TradeAskOffer {
    pub fn total_price(&self) -> u64 {
        self.assets()
            .iter()
            .fold(0, |total, item| total + item.total_price())
    }
}

impl TradeAskFee {
    pub fn new(tx_fee: u64, fees: BTreeMap<Wallet, u64>) -> Self {
        TradeAskFee {
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
