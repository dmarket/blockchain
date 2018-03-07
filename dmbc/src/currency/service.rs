use iron::Handler;
use router::Router;
use exonum::api::Api;
use exonum::blockchain;
use exonum::blockchain::{ApiContext, ServiceContext, Transaction};
use exonum::crypto::PublicKey;
use exonum::encoding;
use exonum::messages::RawTransaction;
use exonum::storage::Fork;
use exonum::encoding::serialize::FromHex;

use config;
use super::nats;
use currency::api::ServiceApi;
use currency::configuration::Configuration;
use currency::wallet;
use currency::wallet::Wallet;
use currency::status;
use currency::transactions::{AddAssets, CreateWallet, DeleteAssets, Exchange,
                             ExchangeIntermediary, Mining, Trade, TradeAsk, TradeAskIntermediary,
                             TradeIntermediary, ADD_ASSETS_ID, CREATE_WALLET_ID, DELETE_ASSETS_ID,
                             EXCHANGE_ID, EXCHANGE_INTERMEDIARY_ID, MINING_ID, TRADE_ASK_ID,
                             TRADE_ASK_INTERMEDIARY_ID, TRADE_ID, TRADE_INTERMEDIARY_ID};
use serde_json;

/// Service identifier.
pub const SERVICE_ID: u16 = 2;

/// Name of the cryptocurrency service.
pub const SERVICE_NAME: &str = "cryptocurrency";

/// Hexadecimal representation of the public key for genesis wallet.
pub const GENESIS_WALLET_PUB_KEY: &str =
    "36a05e418393fb4b23819753f6e6dd51550ce030d53842c43dd1349857a96a61";

/// Service data.
pub struct Service();

impl Service {
    /// Create a new cryptocurrency service.
    pub fn new() -> Self {
        Service()
    }

    /// Genesis wallet public key.
    pub fn genesis_wallet() -> PublicKey {
        PublicKey::from_hex(GENESIS_WALLET_PUB_KEY).unwrap()
    }
}

impl blockchain::Service for Service {
    fn service_name(&self) -> &'static str {
        // TODO: need to use versioned name instead of constant.
        "cryptocurrency/v1"
    }

    fn service_id(&self) -> u16 {
        SERVICE_ID
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
        let trans: Box<Transaction> = match raw.message_type() {
            ADD_ASSETS_ID => Box::new(AddAssets::from_raw(raw)?),
            CREATE_WALLET_ID => Box::new(CreateWallet::from_raw(raw)?),
            DELETE_ASSETS_ID => Box::new(DeleteAssets::from_raw(raw)?),
            EXCHANGE_ID => Box::new(Exchange::from_raw(raw)?),
            EXCHANGE_INTERMEDIARY_ID => Box::new(ExchangeIntermediary::from_raw(raw)?),
            MINING_ID => Box::new(Mining::from_raw(raw)?),
            TRADE_ID => Box::new(Trade::from_raw(raw)?),
            TRADE_INTERMEDIARY_ID => Box::new(TradeIntermediary::from_raw(raw)?),
            TRADE_ASK_ID => Box::new(TradeAsk::from_raw(raw)?),
            TRADE_ASK_INTERMEDIARY_ID => Box::new(TradeAskIntermediary::from_raw(raw)?),
            _ => {
                return Err(encoding::Error::IncorrectMessageType {
                    message_type: raw.message_type(),
                })
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
        let schema = blockchain::Schema::new(ctx.snapshot());
        let last_block = schema.last_block();

        info!("Block #{}.", last_block.height());

        let txs = schema.block_txs(last_block.height());
        for hash in txs.iter() {
            let status = status::Schema(ctx.snapshot()).fetch(&hash);
            let msg = json!({ "tx_hash": status }).to_string();
            let queuename = config::config().nats().queuename();
            nats::publish(queuename, msg);
            info!("Made transaction {:?}", hash.to_hex());
        }
    }

    fn initialize(&self, fork: &mut Fork) -> serde_json::Value {
        let genesis_wallet = Service::genesis_wallet();
        let wallet = Wallet::new(137_000_000_00000000, Vec::new());
        wallet::Schema(fork).store(&genesis_wallet, wallet);

        serde_json::to_value(Configuration::default()).unwrap()
    }
}
