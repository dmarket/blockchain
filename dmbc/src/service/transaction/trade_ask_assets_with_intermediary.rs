extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::CurrencyService;
use service::asset::{Asset, TradeAsset};
use service::transaction::utils;
use service::transaction::utils::Intermediary;
use service::transaction::fee::{calculate_fees_for_trade, TxFees};

use super::{SERVICE_ID, TX_TRADE_ASK_ASSETS_WITH_INTERMEDIARY_ID};
use super::schema::transaction_status::{TxStatus, TxStatusSchema};
use super::schema::wallet::WalletSchema;

encoding_struct! {
    struct TradeAskOfferWithIntermediary {
        const SIZE = 48;

        field intermediary: Intermediary [00 => 08]
        field seller: &PublicKey         [08 => 40]
        field assets: Vec<TradeAsset>    [40 => 48]
    }
}

message! {
    struct TxTradeAskWithIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TX_TRADE_ASK_ASSETS_WITH_INTERMEDIARY_ID;
        const SIZE = 184;

        field buyer:                  &PublicKey                     [00 => 32]
        field offer:                  TradeAskOfferWithIntermediary  [32 => 40]
        field seed:                   u64                            [40 => 48]
        field seller_signature:       &Signature                     [48 => 112]
        field intermediary_signature: &Signature                     [112 => 176]
        field data_info:              &str                           [176 => 184]
    }
}

impl TxTradeAskWithIntermediary {
    pub fn get_fee(&self, view: &mut Fork) -> TxFees {
        calculate_fees_for_trade(view, self.offer().assets())
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        let (mut platform, mut buyer, mut seller, mut intermediary) =
            WalletSchema::map(view, |mut schema| {
                let platform_key = CurrencyService::get_platfrom_wallet();
                (
                    schema.wallet(&platform_key),
                    schema.wallet(self.buyer()),
                    schema.wallet(self.offer().seller()),
                    schema.wallet(self.offer().intermediary().wallet()),
                )
            });

        let fee = self.get_fee(view);

        // Pay fee for tx execution
        if !utils::transfer_coins(view, &mut seller, &mut platform, fee.transaction_fee()) {
            return TxStatus::Fail;
        }

        // initial point for db rollback, in case if transaction has failed
        view.checkpoint();

        // pay commison for the transaction to intermediary
        if !utils::transfer_coins(
            view,
            &mut seller,
            &mut intermediary,
            self.offer().intermediary().commision(),
        ) {
            view.rollback();
            return TxStatus::Fail;
        }

        // convert trade assets to assets stored on the blockchain
        let trade_assets = self.offer().assets();
        let assets = trade_assets
            .iter()
            .map(|x| x.clone().into())
            .collect::<Vec<Asset>>();
        println!("Buyer {:?} => Seller {:?}", buyer, seller);

        println!("--   Ask/Bid transaction   --");
        println!("Seller's balance before transaction : {:?}", seller);
        println!("Buyer's balance before transaction : {:?}", buyer);

        let offer_price = self.offer().total_price();
        if !utils::transfer_coins(view, &mut buyer, &mut seller, offer_price) {
            view.rollback();
            return TxStatus::Fail;
        }

        if !utils::transfer_assets(view, &mut seller, &mut buyer, &assets) {
            view.rollback();
            return TxStatus::Fail;
        }

        for (mut creator, fee) in fee.assets_fees() {
            println!("\tCreator {:?} will receive {}", creator.pub_key(), fee);
            if !utils::transfer_coins(view, &mut seller, &mut creator, fee) {
                view.rollback();
                return TxStatus::Fail;
            }
        }

        TxStatus::Success
    }
}

impl Transaction for TxTradeAskWithIntermediary {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }
        let mut keys_ok = *self.offer().seller() != *self.buyer();
        keys_ok &= *self.offer().seller() != *self.offer().intermediary().wallet();
        keys_ok &= *self.buyer() != *self.offer().intermediary().wallet();

        let verify_seller_ok = crypto::verify(
            self.seller_signature(),
            &self.offer().raw,
            self.offer().seller(),
        );
        let verify_buyer_ok = self.verify_signature(self.buyer());
        let verify_intermediary_ok = crypto::verify(
            self.intermediary_signature(),
            &self.offer().raw,
            self.offer().intermediary().wallet(),
        );

        keys_ok && verify_seller_ok && verify_buyer_ok && verify_intermediary_ok
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

impl TradeAskOfferWithIntermediary {
    pub fn total_price(&self) -> u64 {
        self.assets()
            .iter()
            .fold(0, |total, item| total + item.total_price())
    }
}
