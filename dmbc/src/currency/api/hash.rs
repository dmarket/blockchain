extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use exonum::api::{Api, ApiError};
use exonum::blockchain::Transaction;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use router::Router;

use currency::transaction::add_assets::TxAddAsset;
use currency::transaction::create_wallet::TxCreateWallet;
use currency::transaction::del_assets::TxDelAsset;
use currency::transaction::exchange::{TxExchange, TX_EXCHANGE_ID};
use currency::transaction::exchange_with_intermediary::{TxExchangeWithIntermediary,
                                                       TX_EXCHANGE_WITH_INTERMEDIARY_ID};
use currency::transaction::mining::TxMining;
use currency::transaction::trade_assets::{TxTrade, TX_TRADE_ASSETS_ID};
use currency::transaction::trade_assets_with_intermediary::{TxTradeWithIntermediary,
                                                           TX_TRADE_ASSETS_WITH_INTERMEDIARY_ID};
use currency::transaction::trade_ask_assets::{TxTradeAsk, TX_TRADE_ASK_ASSETS_ID};
use currency::transaction::trade_ask_assets_with_intermediary::{TxTradeAskWithIntermediary,
                                                               TX_TRADE_ASK_ASSETS_WITH_INTERMEDIARY_ID};
use currency::transaction::transfer::TxTransfer;

#[derive(Clone)]
pub struct HashApi {}

#[serde(untagged)]
#[derive(Clone, Serialize, Deserialize)]
enum TransactionRequest {
    CreateWallet(TxCreateWallet),
    Transfer(TxTransfer),
    AddAsset(TxAddAsset),
    DelAsset(TxDelAsset),
    TradeAsset(TxTrade),
    TradeAssetsWithIntermediary(TxTradeWithIntermediary),
    TradeAskAssets(TxTradeAsk),
    TradeAskAssetsWithIntermediary(TxTradeAskWithIntermediary),
    Exchange(TxExchange),
    ExchangeWithIntermediary(TxExchangeWithIntermediary),
    Mining(TxMining),
}

impl Into<Box<Transaction>> for TransactionRequest {
    fn into(self) -> Box<Transaction> {
        match self {
            TransactionRequest::CreateWallet(trans) => Box::new(trans),
            TransactionRequest::Transfer(trans) => Box::new(trans),
            TransactionRequest::AddAsset(trans) => Box::new(trans),
            TransactionRequest::DelAsset(trans) => Box::new(trans),
            TransactionRequest::TradeAsset(trans) => Box::new(trans),
            TransactionRequest::TradeAssetsWithIntermediary(trans) => Box::new(trans),
            TransactionRequest::TradeAskAssets(trans) => Box::new(trans),
            TransactionRequest::TradeAskAssetsWithIntermediary(trans) => Box::new(trans),
            TransactionRequest::Exchange(trans) => Box::new(trans),
            TransactionRequest::ExchangeWithIntermediary(trans) => Box::new(trans),
            TransactionRequest::Mining(trans) => Box::new(trans),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TransactionHashResponse {
    hash: String,
}

impl HashApi {
    pub fn hex_string(bytes: Vec<u8>) -> String {
        let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        strs.join("")
    }
}

impl Api for HashApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let hash_transaction = move |request: &mut Request| -> IronResult<Response> {
            match request.get::<bodyparser::Struct<TransactionRequest>>() {
                Ok(Some(transaction)) => {
                    let transaction: Box<Transaction> = transaction.into();
                    let hash = HashApi::hex_string(transaction.raw().body().to_vec());
                    let response_data = json!(TransactionHashResponse { hash });
                    let ok_res = self_.ok_response(&response_data);
                    let mut res = ok_res.unwrap();
                    res.headers.set(AccessControlAllowOrigin::Any);
                    Ok(res)
                }
                Ok(None) => Err(ApiError::IncorrectRequest("Empty request body".into()))?,
                Err(e) => Err(ApiError::IncorrectRequest(Box::new(e)))?,
            }
        };

        let self_ = self.clone();
        let hash_offer = move |request: &mut Request| -> IronResult<Response> {
            match request.get::<bodyparser::Struct<TransactionRequest>>() {
                Ok(Some(transaction)) => {
                    let transaction: Box<Transaction> = transaction.into();
                    let raw_ = transaction.raw().clone();

                    let vec_hash = match transaction.raw().message_type() {
                        TX_EXCHANGE_ID => match TxExchange::from_raw(raw_) {
                            Ok(exchange) => exchange.get_offer_raw(),
                            Err(_) => vec![],
                        },
                        TX_EXCHANGE_WITH_INTERMEDIARY_ID => {
                            match TxExchangeWithIntermediary::from_raw(raw_) {
                                Ok(exchange) => exchange.get_offer_raw(),
                                Err(_) => vec![],
                            }
                        }
                        TX_TRADE_ASSETS_ID => match TxTrade::from_raw(raw_) {
                            Ok(trade) => trade.get_offer_raw(),
                            Err(_) => vec![],
                        },
                        TX_TRADE_ASSETS_WITH_INTERMEDIARY_ID => {
                            match TxTradeWithIntermediary::from_raw(raw_) {
                                Ok(trade) => trade.get_offer_raw(),
                                Err(_) => vec![],
                            }
                        }
                        TX_TRADE_ASK_ASSETS_ID => match TxTradeAsk::from_raw(raw_) {
                            Ok(trade) => trade.get_offer_raw(),
                            Err(_) => vec![],
                        },
                        TX_TRADE_ASK_ASSETS_WITH_INTERMEDIARY_ID => {
                            match TxTradeAskWithIntermediary::from_raw(raw_) {
                                Ok(trade) => trade.get_offer_raw(),
                                Err(_) => vec![],
                            }
                        }
                        _ => vec![],
                    };
                    let hash = HashApi::hex_string(vec_hash);
                    let response_data = json!(TransactionHashResponse { hash });
                    let ok_res = self_.ok_response(&response_data);
                    let mut res = ok_res.unwrap();
                    res.headers.set(AccessControlAllowOrigin::Any);
                    Ok(res)
                }
                Ok(None) => Err(ApiError::IncorrectRequest("Empty request body".into()))?,
                Err(e) => Err(ApiError::IncorrectRequest(Box::new(e)))?,
            }
        };
        router.post("/hash", hash_transaction, "hash_transaction");
        router.post("/hash/offer", hash_offer, "hash_offer");
    }
}
