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

use service::schema::transaction_status::{TxStatus, TxStatusSchema};
use service::transaction::add_assets::TxAddAsset;
use service::transaction::create_wallet::TxCreateWallet;
use service::transaction::del_assets::TxDelAsset;
use service::transaction::exchange::TxExchange;
use service::transaction::exchange_with_intermediary::TxExchangeWithIntermediary;
use service::transaction::mining::TxMining;
use service::transaction::trade_assets::TxTrade;
use service::transaction::trade_assets_with_intermediary::TxTradeWithIntermediary;
use service::transaction::trade_ask_assets::TxTradeAsk;
use service::transaction::trade_ask_assets_with_intermediary::TxTradeAskWithIntermediary;
use service::transaction::transfer::TxTransfer;

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
    TradeAssetWithIntermediary(TxTradeWithIntermediary),
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
            TransactionRequest::TradeAssetWithIntermediary(trans) => Box::new(trans),
            TransactionRequest::TradeAskAssets(trans) => Box::new(trans),
            TransactionRequest::TradeAskAssetsWithIntermediary(trans) => Box::new(trans),
            TransactionRequest::Exchange(trans) => Box::new(trans),
            TransactionRequest::ExchangeWithIntermediary(trans) => Box::new(trans),
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
        TxStatusSchema::map(&mut view, |mut schema| schema.get_status(tx_hash))
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

        router.post("/transactions", transaction, "transaction");
        router.get("/transactions/:hash", get_status, "get_transaction_status");
    }
}
