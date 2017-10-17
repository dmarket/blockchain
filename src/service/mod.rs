extern crate serde;
extern crate serde_json;
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;
extern crate nats;

pub mod transaction;
pub mod schema;
pub mod wallet;
pub mod api;

use exonum::blockchain::{Service, Transaction, ApiContext, ServiceContext, Schema};
use exonum::messages::{RawTransaction, FromRaw};
use exonum::crypto::{PublicKey, HexValue};
use exonum::encoding;
use exonum::api::Api;
use exonum::storage::{Fork};
use iron::Handler;
use router::Router;
use nats::Client;
use config;

use self::transaction::{TX_TRADE_ASSETS_ID, TX_DEL_ASSETS_ID, TX_ADD_ASSETS_ID, TX_CREATE_WALLET_ID, TX_TRANSFER_ID, TX_EXCHANGE_ID};
use self::transaction::create_wallet::TxCreateWallet;
use self::transaction::transfer::TxTransfer;
use self::transaction::add_assets::TxAddAsset;
use self::transaction::del_assets::TxDelAsset;
use self::transaction::trade_assets::TxTrade;
use self::transaction::exchange::TxExchange;
use self::schema::wallet::WalletSchema;
use self::wallet::{Wallet, Asset};
use self::api::CryptocurrencyApi;

// Service identifier
pub const SERVICE_ID: u16 = 2;
// Identifier for wallet creation transaction type

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
