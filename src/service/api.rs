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
use service::schema::currency::CurrencySchema;


// Service identifier
const SERVICE_ID: u16 = 1;
// Identifier for wallet creation transaction type
const TX_CREATE_WALLET_ID: u16 = 1;
// Identifier for coins transfer transaction type
const TX_TRANSFER_ID: u16 = 2;
// Add Asset
const TX_ADD_ASSETS_ID: u16 = 3;

#[derive(Clone)]
struct CryptocurrencyApi {
    channel: ApiSender<NodeChannel>,
    bc: Blockchain
}

#[serde(untagged)]
#[derive(Clone, Serialize, Deserialize)]
enum TransactionRequest {
    CreateWallet(TxCreateWallet),
    Transfer(TxTransfer),
    AddAsset(TxAddAsset),
}

impl Into<Box<Transaction>> for TransactionRequest {
    fn into(self) -> Box<Transaction> {
        match self {
            TransactionRequest::CreateWallet(trans) => Box::new(trans),
            TransactionRequest::Transfer(trans) => Box::new(trans),
            TransactionRequest::AddAsset(trans) => Box::new(trans),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TransactionResponse {
    tx_hash: Hash,
}

impl Api for CryptocurrencyApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();

        let tx_handler = move |req: &mut Request| -> IronResult<Response> {
            match req.get::<bodyparser::Struct<TransactionRequest>>() {
                Ok(Some(tx)) => {
                    let tx: Box<Transaction> = tx.into();
                    let tx_hash = tx.hash();
                    self_.channel.send(tx).map_err(|e| ApiError::Events(e))?;
                    let json = TransactionResponse { tx_hash };
                    self_.ok_response(&serde_json::to_value(&json).unwrap())
                }
                Ok(None) => Err(ApiError::IncorrectRequest("Empty request body".into()))?,
                Err(e) => Err(ApiError::IncorrectRequest(Box::new(e)))?,
            }
        };

        // Bind the transaction handler to a specific route.
        let route_post = "/v1/wallets/transaction";
        router.post(&route_post, tx_handler, "transaction");


        let self_ = self.clone();
        let get_balance_handler = move |req: &mut Request| -> IronResult<Response> {
            let ref pub_key_str = req.extensions.get::<Router>().unwrap().find("pub_key").unwrap_or("/");
            let pub_key = PublicKey::from_hex(pub_key_str).unwrap();

            let view = &mut self_.bc.fork();
            let mut schema = CurrencySchema { view };

            let wrapped_wallet = schema.wallet(&pub_key);

            if wrapped_wallet.is_none() {
                println!("Wallet not found: {:?}", pub_key);
                return Err(ApiError::NotFound)?;
            }

            let wallet = wrapped_wallet.unwrap();

            self_.ok_response(&serde_json::to_value(&wallet).unwrap())
        };
    
        router.get("/wallet/:pub_key", get_balance_handler, "get_balance");
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
            bc: ctx.blockchain().clone()
        };
        api.wire(&mut router);
        Some(Box::new(router))
    }
}