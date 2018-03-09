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

use currency::transactions::{AddAssets, CreateWallet, DeleteAssets, Exchange,
                             ExchangeIntermediary, Mining, Trade,
                             TradeIntermediary, Transfer, EXCHANGE_ID, EXCHANGE_INTERMEDIARY_ID,
                             TRADE_ID, TRADE_INTERMEDIARY_ID};

#[derive(Clone)]
pub struct HashApi {}

#[serde(untagged)]
#[derive(Clone, Serialize, Deserialize)]
enum TransactionRequest {
    CreateWallet(CreateWallet),
    Transfer(Transfer),
    AddAssets(AddAssets),
    DeleteAssets(DeleteAssets),
    Trade(Trade),
    TradeIntermediary(TradeIntermediary),
    Exchange(Exchange),
    ExchangeIntermediary(ExchangeIntermediary),
    Mining(Mining),
}

impl Into<Box<Transaction>> for TransactionRequest {
    fn into(self) -> Box<Transaction> {
        match self {
            TransactionRequest::CreateWallet(trans) => Box::new(trans),
            TransactionRequest::Transfer(trans) => Box::new(trans),
            TransactionRequest::AddAssets(trans) => Box::new(trans),
            TransactionRequest::DeleteAssets(trans) => Box::new(trans),
            TransactionRequest::Trade(trans) => Box::new(trans),
            TransactionRequest::TradeIntermediary(trans) => Box::new(trans),
            TransactionRequest::Exchange(trans) => Box::new(trans),
            TransactionRequest::ExchangeIntermediary(trans) => Box::new(trans),
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
                        EXCHANGE_ID => match Exchange::from_raw(raw_) {
                            Ok(exchange) => exchange.offer_raw(),
                            Err(_) => vec![],
                        },
                        EXCHANGE_INTERMEDIARY_ID => match ExchangeIntermediary::from_raw(raw_) {
                            Ok(exchange) => exchange.offer_raw(),
                            Err(_) => vec![],
                        },
                        TRADE_ID => match Trade::from_raw(raw_) {
                            Ok(trade) => trade.offer_raw(),
                            Err(_) => vec![],
                        },
                        TRADE_INTERMEDIARY_ID => match TradeIntermediary::from_raw(raw_) {
                            Ok(trade) => trade.offer_raw(),
                            Err(_) => vec![],
                        },
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
