pub mod transaction;
pub mod schema;
pub mod wallet;
pub mod api;
pub mod asset;
pub mod builders;

mod nats;

use exonum::api::Api;
use exonum::blockchain::{ApiContext, Schema, Service, ServiceContext, Transaction};
use exonum::crypto::PublicKey;
use exonum::encoding;
use exonum::encoding::serialize::FromHex;
use exonum::messages::RawTransaction;
use exonum::storage::Fork;
use iron::Handler;
use router::Router;
use serde_json;

use self::api::ServiceApi;
use self::asset::Asset;
use self::schema::transaction_status::TxSchema;
use self::schema::wallet::WalletSchema;
use self::transaction::{TX_ADD_ASSETS_ID, TX_CREATE_WALLET_ID, TX_DEL_ASSETS_ID, TX_EXCHANGE_ID,
                        TX_MINING_ID, TX_TRADE_ASSETS_ID, TX_TRANSFER_ID};
use self::transaction::add_assets::TxAddAsset;
use self::transaction::create_wallet::TxCreateWallet;
use self::transaction::del_assets::TxDelAsset;
use self::transaction::exchange::TxExchange;
use self::transaction::mining::TxMining;
use self::transaction::trade_assets::TxTrade;
use self::transaction::transfer::TxTransfer;
use self::wallet::Wallet;
use config;

// Service identifier
pub const SERVICE_ID: u16 = 2;
pub const SERVICE_NAME: &str = "cryptocurrency/v1";
// Identifier for wallet creation transaction type

pub struct CurrencyService;

impl Service for CurrencyService {
    fn service_name(&self) -> &'static str {
        SERVICE_NAME
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
            TX_MINING_ID => Box::new(TxMining::from_raw(raw)?),
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
        let api = ServiceApi {
            channel: ctx.node_channel().clone(),
            blockchain: ctx.blockchain().clone(),
        };
        api.wire(&mut router);
        Some(Box::new(router))
    }

    fn handle_commit(&self, ctx: &ServiceContext) {
        let schema = Schema::new(ctx.snapshot());
        let service_tx_schema = TxSchema::new(ctx.snapshot());
        let las_block = schema.last_block();
        let list = schema.block_txs(las_block.height());
        for hash in list.iter() {
            let tx_hash = hash.to_hex();
            let status = service_tx_schema.get_status(&hash);
            let msg = json!({ tx_hash: status }).to_string();
            let queuename = config::config().nats().queuename();
            nats::publish(queuename, msg);
            println!("Made transaction {:?}", hash.to_hex());
        }
    }

    fn initialize(&self, fork: &mut Fork) -> serde_json::Value {
        let basic_wallet = PublicKey::from_hex(
            "36a05e418393fb4b23819753f6e6dd51550ce030d53842c43dd1349857a96a61",
        ).unwrap();
        let assets: Vec<Asset> = vec![];
        let wallet = Wallet::new(&basic_wallet, 13_700_000_000_000_000, assets);
        println!("Create the wallet: {:?}", wallet);
        WalletSchema::map(fork, |mut db| db.wallets().put(&basic_wallet, wallet));

        serde_json::Value::Null
    }
}
