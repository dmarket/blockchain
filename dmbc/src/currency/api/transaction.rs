extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use exonum::api::{Api, ApiError};
use exonum::blockchain::{Blockchain, Transaction};
use exonum::crypto::Hash;
use exonum::encoding::serialize::FromHex;
use exonum::node::{ApiSender, TransactionSend};
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use router::Router;

use currency::status;
use currency::transactions::{AddAssets, CreateWallet, DeleteAssets, Exchange,
                             ExchangeIntermediary, Mining, Trade,
                             TradeIntermediary, Transfer};

use currency::error::Error;

#[derive(Clone)]
pub struct TransactionApi {
    pub channel: ApiSender,
    pub blockchain: Blockchain,
}

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
struct TransactionResponse {
    tx_hash: Hash,
    transaction_info: serde_json::Value,
    tx_status: String,
}

impl TransactionApi {
    fn get_status(&self, tx_hash: &Hash) -> Option<Result<(), Error>> {
        let view = &mut self.blockchain.fork();
        status::Schema(view).fetch(tx_hash)
    }
}

impl Api for TransactionApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let transaction = move |req: &mut Request| -> IronResult<Response> {
            match req.get::<bodyparser::Struct<TransactionRequest>>() {
                Ok(Some(transaction)) => {
                    let transaction: Box<Transaction> = transaction.into();
                    let tx_hash = transaction.hash();
                    let tx_info = transaction.info();
                    self_.channel.send(transaction).map_err(ApiError::from)?;
                    let response_data = json!(TransactionResponse {
                        tx_hash,
                        transaction_info: tx_info,
                        tx_status: "pending".to_string(),
                    });
                    let ok_res = self_.ok_response(&response_data);
                    let mut res = ok_res.unwrap();
                    res.headers.set(AccessControlAllowOrigin::Any);
                    Ok(res)
                }
                Ok(None) => Err(ApiError::IncorrectRequest("Empty request body".into()))?,
                Err(e) => Err(ApiError::IncorrectRequest(Box::new(e)))?,
            }
        };
        // Bind the transaction handler to a specific route.

        let self_ = self.clone();
        let get_status = move |request: &mut Request| -> IronResult<Response> {
            let path = request.url.path();
            let tx_hash_str = path.last().unwrap();
            let tx_hash = Hash::from_hex(tx_hash_str).unwrap();
            if let Some(status) = self_.get_status(&tx_hash) {
                let res = self_.ok_response(&json!({ "tx_status": status }));
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res = self_
                    .not_found_response(&serde_json::to_value("Transaction hash not found").unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };

        router.post("/v1/transactions", transaction, "transaction");
        router.get("/v1/transactions/:hash", get_status, "get_transaction_status");
    }
}
