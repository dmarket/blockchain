extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;
use std::collections::BTreeMap;

use service::asset::{Asset, TradeAsset};
use service::transaction::{PER_TRADE_ASSET_FEE, TX_TRADE_FEE};
use service::transaction::fee;
use service::wallet::Wallet;

use super::{SERVICE_ID, TX_TRADE_ASSETS_ID};
use super::schema::asset::AssetSchema;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

const FEE_FOR_TRADE: f64 = 0.025; // 1/40 = 0.025

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

    pub fn get_fee(&self) -> fee::Fee {
        let fee = fee::TxCalculator::new()
            .tx_fee(TX_TRADE_FEE)
            .trade_calculator()
            .per_asset_fee(PER_TRADE_ASSET_FEE)
            .assets(&self.offer().assets())
            .calculate();

        fee
    }

    fn get_creators_and_fees(&self, view: &mut Fork, fee: fee::Fee) -> BTreeMap<Wallet, u64> {
        let mut creators_and_fees = BTreeMap::new();

        for (assetid, fee) in fee.fees_from_trade() {
            if let Some(info) = AssetSchema::map(view, |mut schema| schema.info(&assetid)) {
                if let Some(creator) = WalletSchema::map(
                    view,
                    |mut schema| schema.wallet(info.creator()),
                )
                {
                    *creators_and_fees.entry(creator).or_insert(0) += fee;
                }
            }
        }

        creators_and_fees
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

            let fee = self.get_fee();
            let seller_have_assets = seller.is_assets_in_wallet(&assets);
            let is_sufficient_funds = seller.balance() + price >= fee.amount();
            let tx_status = if (buyer.balance() >= price) && seller_have_assets &&
                is_sufficient_funds
            //todo: необходимо определится с генергацией fee
            {
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
                for (mut creator, fee) in self.get_creators_and_fees(view, fee) {
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
            "tx_fee": self.get_fee().amount(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TxTrade;
    use exonum::blockchain::Transaction;

    fn get_json() -> String {
        r#"{
            "body": {
                "buyer": "f2ab7abcae9363496ccc458a30ec0a58200d9890a12fdfeca35010da6b276e19",
                "offer": {
                    "seller": "dedb2438fca19f04d2236d3005db0f28caa014f34caf98e23634cb49aef1c307",
                    "assets": [
                        {
                            "id": "67e5504410b1426f9247bb680e5fe0c8",
                            "amount": 5,
                            "price": "44"
                        },
                        {
                            "id": "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8",
                            "amount": 7,
                            "price": "44"
                        }
                    ]
                },
                "seed": "4",
                "seller_signature": "4e7d8d57fdc5c102b241d4e7a8d1228658c1de62a9334fa4e70776759268d67f9cdd8c4d20f7db8b226422c644bf442b0e28d9cbecece7753656c92915b02c06"
            },
            "network_id": 0,
            "protocol_version": 0,
            "service_id": 2,
            "message_id": 5,
            "signature": "aac7ce5fee4fca99fb66c978b94f42ba899834fa6b840491ab5c9245967b5a07bda688c0da52876258ee25d63dc1278cf97a6a90e84c8cb3880b5d6d3e606b06"
        }"#.to_string()
    }

    #[test]
    fn exchange_info_test() {
        let tx: TxTrade = ::serde_json::from_str(&get_json()).unwrap();
        assert_eq!(tx.get_fee().amount(), tx.info()["tx_fee"]);
    }
}
