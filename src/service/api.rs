extern crate serde;
extern crate serde_json;
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;

use exonum::blockchain::{Blockchain, Transaction};
use exonum::node::{TransactionSend, ApiSender, NodeChannel};
use exonum::crypto::{PublicKey, Hash, HexValue};
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
use service::schema::wallet::WalletSchema;
use service::schema::asset::AssetSchema;
use service::wallet::Wallet;

#[derive(Clone)]
pub struct CryptocurrencyApi {
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
}
/// Shortcut to get data on wallets.
impl CryptocurrencyApi {
    fn get_wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        let mut view = self.bc.fork();
        let mut schema = WalletSchema { view: &mut view };
        schema.wallet(pub_key)
    }

    fn get_wallets(&self) -> Option<Vec<Wallet>> {
        let mut view = self.bc.fork();
        let mut schema = WalletSchema { view: &mut view };
        let idx = schema.wallets();
        let wallets: Vec<Wallet> = idx.values().collect();
        if wallets.is_empty() {
            None
        } else {
            Some(wallets)
        }
    }

    fn get_owner_for_asset(&self, asset_id: &str) -> Option<PublicKey> {
        let mut view = self.bc.fork();
        let mut schema = AssetSchema { view: &mut view };
        schema.creator(&asset_id.to_string())

    }
}

impl Api for CryptocurrencyApi {
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

        // Gets status of the wallet corresponding to the public key.
        let self_ = self.clone();
        let wallet_info = move |req: &mut Request| -> IronResult<Response> {
            let path = req.url.path();
            let wallet_key = path.last().unwrap();
            let public_key = PublicKey::from_hex(wallet_key).map_err(ApiError::FromHex)?;
            if let Some(wallet) = self_.get_wallet(&public_key) {
                let res = self_.ok_response(&serde_json::to_value(wallet).unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res = self_.not_found_response(&serde_json::to_value("Wallet not found").unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };


        // Gets status of all wallets.
        let self_ = self.clone();
        let wallets_info = move |_: &mut Request| -> IronResult<Response> {
            if let Some(wallets) = self_.get_wallets() {
                let res = self_.ok_response(&serde_json::to_value(wallets).unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res = self_.not_found_response(
                    &serde_json::to_value("Wallets database is empty")
                        .unwrap(),
                );
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };

        let self_ = self.clone();
        let get_owner_for_asset_id = move |req: &mut Request| ->IronResult<Response> {
            let path = req.url.path();
            let asset_id = path.last().unwrap();
            if let Some(owner) = self_.get_owner_for_asset(*asset_id) {
                let res= self_.ok_response(&serde_json::to_value(owner).unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res = self_.not_found_response(&serde_json::to_value("Asset not found").unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };

        router.post("/wallets/transaction", transaction, "transaction");
        router.get("/wallets", wallets_info, "wallets_info");
        router.get("/wallet/:pub_key", wallet_info, "get_balance");
        router.get("/asset/:asset_id", get_owner_for_asset_id, "get_owner_for_asset_id");
    }
}
