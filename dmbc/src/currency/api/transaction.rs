extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use exonum::api::Api;
use exonum::blockchain::{Blockchain, Transaction};
use exonum::crypto::Hash;
use exonum::encoding::serialize::FromHex;
use exonum::node::{ApiSender, TransactionSend};
use hyper::header::ContentType;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use iron::status as istatus;
use prometheus::IntCounter;
use router::Router;

use currency::api::error::ApiError;
use currency::status;
use currency::transactions::{AddAssets, DeleteAssets, Exchange, ExchangeIntermediary, Mine, Trade,
                             TradeIntermediary, Transfer};

use currency::error::Error;

#[derive(Clone)]
pub struct TransactionApi {
    pub channel: ApiSender,
    pub blockchain: Blockchain,
}

#[serde(untagged)]
#[derive(Clone, Serialize, Deserialize)]
pub enum TransactionRequest {
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

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct TransactionResponse {
    pub tx_hash: Hash,
}

pub type TxPostResponse = Result<Result<TransactionResponse, Error>, ApiError>;

//#[derive(Serialize, Deserialize, Debug)]
pub type StatusResponse = Result<Result<(), Error>, ApiError>;

impl TransactionApi {
    fn get_status(&self, tx_hash: &Hash) -> Option<Result<(), Error>> {
        let view = &mut self.blockchain.fork();
        status::Schema(view).fetch(tx_hash)
    }
}

lazy_static! {
    static ref POST_REQUESTS: IntCounter = register_int_counter!(
        "dmbc_transaction_api_post_requests_total",
        "Transaction post requests."
    ).unwrap();
    static ref POST_RESPONSES: IntCounter = register_int_counter!(
        "dmbc_transaction_api_post_responses_total",
        "Transaction post responses."
    ).unwrap();
    static ref GET_STATUS_REQUESTS: IntCounter = register_int_counter!(
        "dmbc_transaction_api_get_status_requests_total",
        "Transaction status requests."
    ).unwrap();
    static ref GET_STATUS_RESPONSES: IntCounter = register_int_counter!(
        "dmbc_transaction_api_get_status_responses_total",
        "Transaction status responses."
    ).unwrap();
}

impl Api for TransactionApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let transaction = move |req: &mut Request| -> IronResult<Response> {
            POST_REQUESTS.inc();

            let s: TxPostResponse = match req.get::<bodyparser::Struct<TransactionRequest>>() {
                Ok(Some(transaction)) => {
                    let tx: Box<Transaction> = transaction.into();
                    let tx_hash = tx.hash();
                    match self_.channel.send(tx) {
                        Ok(_) => Ok(Ok(TransactionResponse { tx_hash })),
                        Err(_) => Ok(Err(Error::UnableToVerifyTransaction)),
                    }
                }
                Ok(None) => Err(ApiError::EmptyRequestBody),
                Err(_) => Err(ApiError::IncorrectRequest),
            };
            let ss = s.clone()
                .ok()
                .map(|r| {
                    r.err()
                        .map(|_| istatus::BadRequest)
                        .unwrap_or(istatus::Created)
                })
                .unwrap_or(istatus::BadRequest);

            let mut res = Response::with((ss, serde_json::to_string_pretty(&s).unwrap()));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            POST_RESPONSES.inc();

            Ok(res)
        };
        // Bind the transaction handler to a specific route.

        let self_ = self.clone();
        let get_status = move |request: &mut Request| -> IronResult<Response> {
            GET_STATUS_REQUESTS.inc();

            let path = request.url.path();
            let tx_hash_str = path.last().unwrap();
            let s: StatusResponse = Hash::from_hex(tx_hash_str)
                .map_err(|_| ApiError::TransactionHashInvalid)
                .and_then(|tx_hash| {
                    self_
                        .get_status(&tx_hash)
                        .ok_or(ApiError::TransactionNotFound)
                });

            let mut res = Response::with((
                s.clone()
                    .err()
                    .map(|e| e.to_status())
                    .unwrap_or(istatus::Ok),
                serde_json::to_string_pretty(&s).unwrap(),
            ));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            GET_STATUS_RESPONSES.inc();

            Ok(res)
        };

        router.post("/v1/transactions", transaction, "transaction");
        router.get(
            "/v1/transactions/:hash",
            get_status,
            "get_transaction_status",
        );
    }
}
