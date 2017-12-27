extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use std::collections::BTreeMap;

use service::asset::{Asset, TradeAsset};
use service::transaction::TX_TRADE_FEE;
use service::wallet::Wallet;

use super::{SERVICE_ID, TX_TRADE_ASSETS_ID};
use super::schema::asset::AssetSchema;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

encoding_struct! {
    struct TradeOffer {
        const SIZE = 40;

        field seller: &PublicKey        [00 => 32]
        field assets: Vec<TradeAsset>   [32 => 40]
    }
}

impl TradeOffer {
    pub fn total_price(&self) -> u64 {
        self.assets().iter().fold(0, |total, item| {
            total + item.total_price()
        })
    }
}

pub struct TradeFee {
    transaction_fee: u64,
    assets_fees: BTreeMap<Wallet, u64>,
}

impl TradeFee {
    pub fn new(tx_fee: u64, fees: BTreeMap<Wallet, u64>) -> Self {
        TradeFee {
            transaction_fee: tx_fee,
            assets_fees: fees,
        }
    }

    pub fn amount(&self) -> u64 {
        let mut amount = self.transaction_fee;
        amount += self.assets_fees.iter().fold(0, |acc, asset| acc + asset.1);
        amount
    }

    pub fn assets_fees(&self) -> BTreeMap<Wallet, u64> {
        self.assets_fees.clone()
    }
}

message! {
    struct TxTrade {
        const TYPE = SERVICE_ID;
        const ID = TX_TRADE_ASSETS_ID;
        const SIZE = 112;

        field buyer:              &PublicKey    [00 => 32]
        field offer:              TradeOffer    [32 => 40]
        field seed:               u64           [40 => 48]
        field seller_signature:   &Signature    [48 => 112]

    }
}

impl TxTrade {
    fn offer_verify(&self) -> bool {
        *self.buyer() != *self.offer().seller() &&
            crypto::verify(
                self.seller_signature(),
                &self.offer().raw,
                self.offer().seller(),
            )
    }

    pub fn get_offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    pub fn get_fee(&self, view: &mut Fork) -> TradeFee {
        let mut assets_fees = BTreeMap::new();
        let fee_ratio = |price: u64, ratio: u64| (price as f64 / ratio as f64).round() as u64;

        for asset in self.offer().assets() {
            if let Some(info) = AssetSchema::map(view, |mut schema| schema.info(&asset.id())) {

                let trade_fee = info.fees().trade();
                let fee = trade_fee.tax() + fee_ratio(asset.total_price(), trade_fee.ratio());

                if let Some(creator) = WalletSchema::map(
                    view,
                    |mut schema| schema.wallet(info.creator()),
                )
                {
                    *assets_fees.entry(creator).or_insert(0) += fee;
                }
            }
        }

        TradeFee::new(TX_TRADE_FEE, assets_fees)
    }
}

impl Transaction for TxTrade {
    fn verify(&self) -> bool {
        self.offer_verify() && self.verify_signature(self.buyer())
    }

    fn execute(&self, view: &mut Fork) {
        let participants = WalletSchema::map(view, |mut schema| {
            (
                schema.wallet(self.buyer()),
                schema.wallet(self.offer().seller()),
            )
        });
        if let (Some(mut buyer), Some(mut seller)) = participants {
            let price = self.offer().total_price();
            let trade_assets = self.offer().assets();
            let assets = trade_assets
                .iter()
                .map(|x| x.clone().into())
                .collect::<Vec<Asset>>();
            println!("Buyer {:?} => Seller {:?}", buyer, seller);

            let fee = self.get_fee(view);
            let seller_have_assets = seller.is_assets_in_wallet(&assets);
            let is_sufficient_funds = seller.balance() + price >= fee.amount();
            let tx_status =
                if (buyer.balance() >= price) && seller_have_assets && is_sufficient_funds {
                    println!("--   Trade transaction   --");
                    println!("Seller's balance before transaction : {:?}", seller);
                    println!("Buyer's balance before transaction : {:?}", buyer);
                    seller.del_assets(&assets);
                    seller.increase(price);
                    seller.decrease(fee.amount());
                    buyer.add_assets(&assets);
                    buyer.decrease(price);
                    println!("Seller's balance after transaction : {:?}", seller);
                    println!("Buyer's balance after transaction : {:?}", buyer);
                    WalletSchema::map(view, |mut schema| {
                        schema.wallets().put(self.buyer(), buyer);
                        schema.wallets().put(self.offer().seller(), seller);
                    });

                    // send fee to creators of assets
                    for (mut creator, fee) in fee.assets_fees() {
                        println!("Creator {:?} will receive {}", creator.pub_key(), fee);
                        creator.increase(fee);
                        WalletSchema::map(view, |mut schema| {
                            schema.wallets().put(creator.pub_key(), creator.clone());
                        });
                    }

                    TxStatus::Success
                } else {
                    TxStatus::Fail
                };
            TxStatusSchema::map(
                view,
                |mut schema| schema.set_status(&self.hash(), tx_status),
            );
        }
    }

    fn info(&self) -> Value {
        json!({
            "transaction_data": self,
        })
    }
}
