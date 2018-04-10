extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use exonum::api::Api;
use exonum::blockchain::Transaction;
use iron::headers::AccessControlAllowOrigin;
use hyper::header::ContentType;
use iron::prelude::*;
use iron::status as istatus;
use router::Router;

use currency::transactions::{AddAssets, DeleteAssets, Exchange,
                             ExchangeIntermediary, Mine, Trade,
                             TradeIntermediary, Transfer, EXCHANGE_ID, EXCHANGE_INTERMEDIARY_ID,
                             TRADE_ID, TRADE_INTERMEDIARY_ID};
use currency::api::error::ApiError;

#[derive(Clone)]
pub struct HexApi {}

#[serde(untagged)]
#[derive(Clone, Serialize, Deserialize)]
enum TransactionRequest {
    Transfer(Transfer),
    AddAssets(AddAssets),
    DeleteAssets(DeleteAssets),
    Trade(Trade),
    TradeIntermediary(TradeIntermediary),
    Exchange(Exchange),
    ExchangeIntermediary(ExchangeIntermediary),
    Mine(Mine),
}

impl Into<Box<Transaction>> for TransactionRequest {
    fn into(self) -> Box<Transaction> {
        match self {
            TransactionRequest::Transfer(trans) => Box::new(trans),
            TransactionRequest::AddAssets(trans) => Box::new(trans),
            TransactionRequest::DeleteAssets(trans) => Box::new(trans),
            TransactionRequest::Trade(trans) => Box::new(trans),
            TransactionRequest::TradeIntermediary(trans) => Box::new(trans),
            TransactionRequest::Exchange(trans) => Box::new(trans),
            TransactionRequest::ExchangeIntermediary(trans) => Box::new(trans),
            TransactionRequest::Mine(trans) => Box::new(trans),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct HexResponse {
    pub hex: String,
}

pub type HexApiResponse = Result<Option<HexResponse>, ApiError>;

impl HexApi {
    pub fn hex_string(bytes: Vec<u8>) -> String {
        let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        strs.join("")
    }
}

impl Api for HexApi {
    fn wire(&self, router: &mut Router) {
        let hex_transaction = move |request: &mut Request| -> IronResult<Response> {
            let body: HexApiResponse = match request.get::<bodyparser::Struct<TransactionRequest>>() {
                Ok(Some(transaction)) => {
                    let tx: Box<Transaction> = transaction.into();
                    let hex = Self::hex_string(tx.raw().body().to_vec());
                    Ok(Some(HexResponse{hex}))
                },
                Ok(None) => Err(ApiError::EmptyRequestBody),
                Err(_) => Err(ApiError::IncorrectRequest),
            };

            let mut res = Response::with((
                body.clone().err().map(|e| e.to_status()).unwrap_or(istatus::Ok),
                serde_json::to_string_pretty(&body).unwrap(),
            ));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            Ok(res)
        };

        let hex_tx_offer = move |request: &mut Request| -> IronResult<Response> {
            let body: HexApiResponse = match request.get::<bodyparser::Struct<TransactionRequest>>() {
                Ok(Some(transaction)) => {
                    let transaction: Box<Transaction> = transaction.into();
                    let raw_ = transaction.raw().clone();

                    let vec_hash:Option<Vec<u8>> = match transaction.raw().message_type() {
                        EXCHANGE_ID => match Exchange::from_raw(raw_) {
                            Ok(exchange) => Some(exchange.offer_raw()),
                            Err(_) => None,
                        },
                        EXCHANGE_INTERMEDIARY_ID => match ExchangeIntermediary::from_raw(raw_) {
                            Ok(exchange) => Some(exchange.offer_raw()),
                            Err(_) => None,
                        },
                        TRADE_ID => match Trade::from_raw(raw_) {
                            Ok(trade) => Some(trade.offer_raw()),
                            Err(_) => None,
                        },
                        TRADE_INTERMEDIARY_ID => match TradeIntermediary::from_raw(raw_) {
                            Ok(trade) => Some(trade.offer_raw()),
                            Err(_) => None,
                        },
                        _ => None,
                    };
                    match vec_hash {
                        Some(vec) => {
                            let hex = Self::hex_string(vec);
                            Ok(Some(HexResponse{hex}))
                        },
                        None => Ok(None)
                    }
                },
                Ok(None) => Err(ApiError::EmptyRequestBody),
                Err(_) => Err(ApiError::IncorrectRequest),
            };

            let mut res = Response::with((
                body.clone().err().map(|e| e.to_status()).unwrap_or(istatus::Ok),
                serde_json::to_string_pretty(&body).unwrap(),
            ));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            Ok(res)
        };
        router.post("/v1/hex/transactions", hex_transaction, "hash_transaction");
        router.post("/v1/hex/transactions/offer", hex_tx_offer, "hash_offer");
    }
}
