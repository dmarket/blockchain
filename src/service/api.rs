extern crate serde;
extern crate serde_json;
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;

use exonum::blockchain::{Blockchain, Service, Transaction, ApiContext};
use exonum::node::{TransactionSend, ApiSender, NodeChannel};
use exonum::messages::{RawTransaction, FromRaw};
use exonum::crypto::{PublicKey, Hash, HexValue};
use exonum::encoding;
use exonum::api::{Api, ApiError};
use iron::prelude::*;
use iron::Handler;
use router::Router;

use service::transaction::create_wallet::TxCreateWallet;
use service::transaction::transfer::TxTransfer;
use service::transaction::add_assets::TxAddAsset;
use service::transaction::del_assets::TxDelAsset;
use service::schema::currency::CurrencySchema;
use service::wallet::Wallet;


// Service identifier
const SERVICE_ID: u16 = 1;
// Identifier for wallet creation transaction type
const TX_CREATE_WALLET_ID: u16 = 1;
// Identifier for coins transfer transaction type
const TX_TRANSFER_ID: u16 = 2;
// Add Asset
const TX_ADD_ASSETS_ID: u16 = 3;
// Add Asset
const TX_DEL_ASSETS_ID: u16 = 4;

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
}

impl Into<Box<Transaction>> for TransactionRequest {
    fn into(self) -> Box<Transaction> {
        match self {
            TransactionRequest::CreateWallet(trans) => Box::new(trans),
            TransactionRequest::Transfer(trans) => Box::new(trans),
            TransactionRequest::AddAsset(trans) => Box::new(trans),
            TransactionRequest::DelAsset(trans) => Box::new(trans),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TransactionResponse {
    tx_hash: Hash,
}
/// Shortcut to get data on wallets.
impl CryptocurrencyApi {
    fn get_wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        let mut view = self.bc.fork();
        let mut schema = CurrencySchema { view: &mut view };
        schema.wallet(pub_key)
    }

    fn get_wallets(&self) -> Option<Vec<Wallet>> {
        let mut view = self.bc.fork();
        let mut schema = CurrencySchema { view: &mut view };
        let idx = schema.wallets();
        let wallets: Vec<Wallet> = idx.values().collect();
        if wallets.is_empty() {
            None
        } else {
            Some(wallets)
        }
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
                    self_.channel.send(transaction).map_err(ApiError::Events)?;
                    let json = TransactionResponse { tx_hash };
                    self_.ok_response(&serde_json::to_value(&json).unwrap())
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
                self_.ok_response(&serde_json::to_value(wallet).unwrap())
            } else {
                self_.not_found_response(&serde_json::to_value("Wallet not found").unwrap())
            }
        };


        // Gets status of all wallets.
        let self_ = self.clone();
        let wallets_info = move |_: &mut Request| -> IronResult<Response> {
            if let Some(wallets) = self_.get_wallets() {
                self_.ok_response(&serde_json::to_value(wallets).unwrap())
            } else {
                self_.not_found_response(
                    &serde_json::to_value("Wallets database is empty")
                        .unwrap(),
                )
            }
        };

        router.post("/wallets/transaction", transaction, "transaction");
        router.get("/wallets", wallets_info, "wallets_info");
        router.get("/wallet/:pub_key", wallet_info, "get_balance");
    }
}

pub struct CurrencyService;

impl Service for CurrencyService {
    fn service_name(&self) -> &'static str {
        "cryptocurrency"
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
}
