extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, Signature};
use exonum::messages::Message;
use exonum::storage::Fork;
use serde_json::Value;

use service::CurrencyService;
use service::asset::{Asset, TradeAsset};
use service::transaction::fee::TxFees;

use service::transaction::intermediary::Intermediary;
use service::schema::wallet::WalletSchema;

use super::SERVICE_ID;
use super::schema::transaction_status::{TxStatus, TxStatusSchema};

pub const TX_TRADE_ASSETS_WITH_INTERMEDIARY_ID: u16 = 504;

encoding_struct! {
    struct TradeOfferWithIntermediary {
        const SIZE = 80;

        field intermediary: Intermediary [00 => 08]
        field buyer: &PublicKey          [08 => 40]
        field seller: &PublicKey         [40 => 72]
        field assets: Vec<TradeAsset>    [72 => 80]
    }
}

message! {
    struct TxTradeWithIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TX_TRADE_ASSETS_WITH_INTERMEDIARY_ID;
        const SIZE = 152;

        field offer:              TradeOfferWithIntermediary [00 => 08]
        field seed:               u64                        [08 => 16]
        field seller_signature:   &Signature                 [16 => 80]
        field intermediary_signature: &Signature             [80 => 144]
        field data_info:          &str                       [144 => 152]
    }
}

impl TradeOfferWithIntermediary {
    pub fn total_price(&self) -> u64 {
        self.assets()
            .iter()
            .fold(0, |total, item| total + item.total_price())
    }
}

impl TxTradeWithIntermediary {
    pub fn get_offer_raw(&self) -> Vec<u8> {
        self.offer().raw
    }

    pub fn get_fee(&self, view: &mut Fork) -> TxFees {
        TxFees::for_trade(view, self.offer().assets())
    }

    fn process(&self, view: &mut Fork) -> TxStatus {
        let mut platform =
            WalletSchema::get_wallet(view, &CurrencyService::genesis_wallet_pub_key());
        let mut buyer = WalletSchema::get_wallet(view, self.offer().buyer());
        let mut seller = WalletSchema::get_wallet(view, self.offer().seller());
        let mut intermediary = WalletSchema::get_wallet(view, self.offer().intermediary().wallet());

        let fee = self.get_fee(view);

        // Pay fee for tx execution
        if WalletSchema::transfer_coins(view, &mut seller, &mut platform, fee.transaction_fee())
            .is_err()
        {
            return TxStatus::Fail;
        }

        // initial point for db rollback, in case if transaction has failed
        view.commit();
        view.checkpoint();

        // pay commison for the transaction to intermediary
        if WalletSchema::transfer_coins(
            view,
            &mut seller,
            &mut intermediary,
            self.offer().intermediary().commission(),
        ).is_err()
        {
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

        println!("--   Trade transaction   --");
        println!("Seller's balance before transaction : {:?}", seller);
        println!("Buyer's balance before transaction : {:?}", buyer);

        let offer_price = self.offer().total_price();
        if WalletSchema::transfer_coins(view, &mut buyer, &mut seller, offer_price).is_err() {
            view.rollback();
            return TxStatus::Fail;
        }

        if WalletSchema::transfer_assets(view, &mut seller, &mut buyer, &assets).is_err() {
            view.rollback();
            return TxStatus::Fail;
        }

        for (mut creator, fee) in fee.assets_fees() {
            println!("\tCreator {:?} will receive {}", creator.pub_key(), fee);
            if WalletSchema::transfer_coins(view, &mut seller, &mut creator, fee).is_err() {
                view.rollback();
                return TxStatus::Fail;
            }
        }

        TxStatus::Success
    }
}

impl Transaction for TxTradeWithIntermediary {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return false;
        }

        let mut keys_ok = *self.offer().seller() != *self.offer().buyer();
        keys_ok &= *self.offer().seller() != *self.offer().intermediary().wallet();
        keys_ok &= *self.offer().buyer() != *self.offer().intermediary().wallet();

        let verify_seller_ok = crypto::verify(
            self.seller_signature(),
            &self.offer().raw,
            self.offer().seller(),
        );

        // not sure if this is ok
        let verify_buyer_ok = self.verify_signature(self.offer().buyer());

        let verify_intermediary_ok = crypto::verify(
            self.intermediary_signature(),
            &self.offer().raw,
            self.offer().intermediary().wallet(),
        );

        keys_ok && verify_buyer_ok && verify_seller_ok && verify_intermediary_ok
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
