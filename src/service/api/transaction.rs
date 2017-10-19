extern crate serde;
extern crate serde_json;
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;

use exonum::blockchain::{Blockchain, Transaction};
use exonum::node::{TransactionSend, ApiSender, NodeChannel};
use exonum::crypto::Hash;
use exonum::api::{Api, ApiError};
use iron::headers::{AccessControlAllowOrigin};
use iron::prelude::*;
use router::Router;

use service::transaction::create_wallet::TxCreateWallet;
use service::transaction::transfer::TxTransfer;
use service::transaction::add_assets::TxAddAsset;
use service::transaction::del_assets::TxDelAsset;
use service::transaction::trade_assets::TxTrade;
use service::transaction::exchange::TxExchange;

#[derive(Clone)]
pub struct TransactionApi {
    pub channel: ApiSender<NodeChannel>,
    pub bc: Blockchain,
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
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TransactionResponse {
    tx_hash: Hash,
    transaction_info: serde_json::Value,
    tx_status: String
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
                    self_.channel.send(transaction).map_err(ApiError::Events)?;
                    let response_data = json!(TransactionResponse{
                        tx_hash,
                        transaction_info: tx_info,
                        tx_status: "pending".to_string()
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

        router.post("/wallets/transaction", transaction, "transaction");
    }
}
