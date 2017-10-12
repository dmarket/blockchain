extern crate serde;
extern crate serde_json;
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;
extern crate nats;

use exonum::blockchain::{Blockchain, Service, Transaction, ApiContext, ServiceContext, Schema};
use exonum::node::{TransactionSend, ApiSender, NodeChannel};
use exonum::messages::{RawTransaction, FromRaw};
use exonum::crypto::{PublicKey, Hash, HexValue};
use exonum::encoding;
use exonum::api::{Api, ApiError};
use exonum::storage::{Fork};
use iron::headers::{AccessControlAllowOrigin};
use iron::prelude::*;
use iron::Handler;
use router::Router;
use nats::Client;

use service::transaction::{TX_TRADE_ASSETS_ID, TX_DEL_ASSETS_ID, TX_ADD_ASSETS_ID, TX_CREATE_WALLET_ID, TX_TRANSFER_ID, TX_EXCHANGE_ID};
use service::transaction::create_wallet::TxCreateWallet;
use service::transaction::transfer::TxTransfer;
use service::transaction::add_assets::TxAddAsset;
use service::transaction::del_assets::TxDelAsset;
use service::transaction::trade_assets::TxTrade;
use service::transaction::exchange::TxExchange;
use service::schema::wallet::WalletSchema;
use service::schema::asset::AssetSchema;
use service::wallet::{Wallet, Asset};
use config;

// Service identifier
pub const SERVICE_ID: u16 = 2;
// Identifier for wallet creation transaction type

#[derive(Clone)]
struct CryptocurrencyApi {
    channel: ApiSender<NodeChannel>,
    bc: Blockchain,
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

pub struct CurrencyService;

impl Service for CurrencyService {
    fn service_name(&self) -> &'static str {
        "cryptocurrency/v1"
    }

    fn service_id(&self) -> u16 {
        SERVICE_ID
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
        let trans: Box<Transaction> = match raw.message_type() {
            TX_TRANSFER_ID => Box::new(TxTransfer::from_raw(raw)?),
            TX_CREATE_WALLET_ID => Box::new(TxCreateWallet::from_raw(raw)?),
            TX_ADD_ASSETS_ID => Box::new(TxAddAsset::from_raw(raw)?),
            TX_DEL_ASSETS_ID => Box::new(TxDelAsset::from_raw(raw)?),
            TX_TRADE_ASSETS_ID => Box::new(TxTrade::from_raw(raw)?),
            TX_EXCHANGE_ID => Box::new(TxExchange::from_raw(raw)?),
            _ => {
                return Err(encoding::Error::IncorrectMessageType {
                    message_type: raw.message_type(),
                });
            }
        };
        Ok(trans)
    }

    fn public_api_handler(&self, ctx: &ApiContext) -> Option<Box<Handler>> {
        let mut router = Router::new();
        let api = CryptocurrencyApi {
            channel: ctx.node_channel().clone(),
            bc: ctx.blockchain().clone(),
        };
        api.wire(&mut router);
        Some(Box::new(router))
    }

    fn handle_commit(&self, ctx: &mut ServiceContext) {
        match Client::new(config::config().nats().addresses()) {
            Ok(mut client) => {
                let schema = Schema::new(ctx.snapshot());
                if let Some(las_block) = schema.last_block() {
                    let list = schema.block_txs(las_block.height());
                    for hash in list.iter() {
                        match client.publish("transaction.commit", hash.to_hex().as_bytes()) {
                            Ok(_) => println!("success published"),
                            Err(e) => println!("{:?}", e)
                        }
                        println!("Made transaction {:?}", hash.to_hex());
                    }
                }
            },
            Err(e) => println!("NATS server error {:?}", e)
        }
    }

    fn initialize(&self, fork: &mut Fork) -> serde_json::Value {
        let mut schema = WalletSchema { view: fork };
        let basic_wallet = PublicKey::from_hex("36a05e418393fb4b23819753f6e6dd51550ce030d53842c43dd1349857a96a61").unwrap();
        let assets: Vec<Asset> = vec![];
        let wallet = Wallet::new(&basic_wallet, 100_000_000_000, assets);
        println!("Create the wallet: {:?}", wallet);
        schema.wallets().put(&basic_wallet, wallet);

        serde_json::Value::Null
    }
}
