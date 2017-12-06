extern crate serde;
extern crate serde_json;
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;

use exonum::blockchain::{Blockchain, Transaction};
use exonum::node::{TransactionSend, ApiSender};
use exonum::crypto::{Hash, HexValue};
use exonum::api::{Api, ApiError};
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use router::Router;

use service::transaction::create_wallet::TxCreateWallet;
use service::transaction::transfer::TxTransfer;
use service::transaction::add_assets::TxAddAsset;
use service::transaction::del_assets::TxDelAsset;
use service::transaction::trade_assets::TxTrade;
use service::transaction::exchange::TxExchange;
use service::transaction::mining::TxMining;
use service::schema::transaction_status::{TxStatusSchema, TxStatus};


#[derive(Clone)]
pub struct TransactionApi {
    pub channel: ApiSender,
    pub blockchain: Blockchain,
}

#[serde(untagged)]
#[derive(Clone, Serialize, Deserialize)]
enum TransactionRequest {
    CreateWallet(TxCreateWallet),
    Transfer(TxTransfer),
    AddAsset(TxAddAsset),
    DelAsset(TxDelAsset),
    TradeAsset(TxTrade),
    Exchange(TxExchange),
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
            TransactionRequest::Exchange(trans) => Box::new(trans),
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
    fn get_status(&self, tx_hash: &Hash) -> Option<TxStatus> {
        let mut view = self.blockchain.fork();
        TxStatusSchema::map(&mut view, |mut schema| {schema.get_status(tx_hash)})
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
                let res = self_.ok_response(&json!({
                    "tx_status": status
                }));
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res = self_.not_found_response(
                    &serde_json::to_value("Transaction hash not found").unwrap(),
                );
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };

        router.post("/wallets/transaction", transaction, "transaction");
        router.get("/transaction/:hash", get_status, "get_transaction_status");

    }
}
