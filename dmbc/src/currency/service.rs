use exonum::api::Api;
use exonum::blockchain;
use exonum::blockchain::{ApiContext, ServiceContext, Transaction};
use exonum::crypto::{Hash, PublicKey};
use exonum::encoding;
use exonum::encoding::serialize::FromHex;
use exonum::messages::Message;
use exonum::messages::RawTransaction;
use exonum::storage::Fork;
use exonum::storage::Snapshot;
use iron::Handler;
use prometheus::IntGauge;
use router::Router;
use std::sync::RwLock;
use std::collections::HashMap;

use super::nats;
use config;
use currency::api::ServiceApi;
use currency::configuration;
use currency::configuration::Configuration;
use currency::status;
use currency::transactions::{
    AddAssets, DeleteAssets, Exchange, ExchangeIntermediary, Trade, TradeIntermediary, Transfer, TransferWithFeesPayer,
    BidOffer, AskOffer,
    ADD_ASSETS_ID, DELETE_ASSETS_ID, EXCHANGE_ID, EXCHANGE_INTERMEDIARY_ID, TRADE_ID,
    TRADE_INTERMEDIARY_ID, TRANSFER_ID, TRANSFER_FEES_PAYER_ID, BID_OFFER_ID, ASK_OFFER_ID
};
use currency::wallet;
use currency::wallet::Wallet;
use serde_json;

/// Service identifier.
pub const SERVICE_ID: u16 = 2;

/// Name of the cryptocurrency service.
pub const SERVICE_NAME: &str = "cryptocurrency";

/// Service data.
pub struct Service();

impl Service {
    /// Create a new cryptocurrency service.
    pub fn new() -> Self {
        Service()
    }

    /// Genesis wallet public key.
    pub fn genesis_wallet<S: AsRef<Snapshot>>(view: S) -> PublicKey {
        let config = Configuration::extract(view.as_ref());
        *config.fees().recipient()
    }
}

lazy_static! {
    static ref BLOCKCHAIN_HEIGHT: IntGauge = register_int_gauge!(
        "dmbc_blockchain_height_blocks",
        "Height of the blockchain of the current node in blocks."
    ).unwrap();
    pub static ref CONFIGURATION: RwLock<Configuration> = RwLock::new(Configuration::default());
    pub static ref PERMISSIONS: RwLock<HashMap<PublicKey, u64>> = RwLock::new(HashMap::new());
    static ref CONFIG_HASH: RwLock<Option<Hash>> = RwLock::new(None);
}

impl blockchain::Service for Service {
    fn service_name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn service_id(&self) -> u16 {
        SERVICE_ID
    }

    fn state_hash(&self, _snapshot: &Snapshot) -> Vec<Hash> {
        vec![]
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
        let trans: Box<Transaction> = match raw.message_type() {
            ADD_ASSETS_ID => Box::new(AddAssets::from_raw(raw)?),
            DELETE_ASSETS_ID => Box::new(DeleteAssets::from_raw(raw)?),
            EXCHANGE_ID => Box::new(Exchange::from_raw(raw)?),
            EXCHANGE_INTERMEDIARY_ID => Box::new(ExchangeIntermediary::from_raw(raw)?),
            TRADE_ID => Box::new(Trade::from_raw(raw)?),
            TRADE_INTERMEDIARY_ID => Box::new(TradeIntermediary::from_raw(raw)?),
            TRANSFER_ID => Box::new(Transfer::from_raw(raw)?),
            TRANSFER_FEES_PAYER_ID => Box::new(TransferWithFeesPayer::from_raw(raw)?),
            BID_OFFER_ID => Box::new(BidOffer::from_raw(raw)?),
            ASK_OFFER_ID => Box::new(AskOffer::from_raw(raw)?),
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

        BLOCKCHAIN_HEIGHT.set(last_block.height().0 as i64);
        let hash = schema.actual_configuration().previous_cfg_hash;
        let stored_hash = *CONFIG_HASH.read().unwrap();
        if stored_hash.is_none() || stored_hash.unwrap() != hash {
            let service_configuration = Configuration::extract(ctx.snapshot());
            let mut updated_permission_list = HashMap::<PublicKey, u64>::new();
            for wallet in service_configuration.permissions().wallets() {
                updated_permission_list.insert(*wallet.key(), wallet.mask());
            }
            *CONFIGURATION.write().unwrap() = service_configuration;
            *PERMISSIONS.write().unwrap() = updated_permission_list;

            *CONFIG_HASH.write().unwrap() = Some(hash);
        }

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
        let genesis_wallet = PublicKey::from_hex(configuration::GENESIS_WALLET_PUB_KEY).unwrap();
        let wallet = Wallet::new(56_921_773_17197150, Vec::new());
        wallet::Schema(fork).store(&genesis_wallet, wallet);

        serde_json::to_value(Configuration::default()).unwrap()
    }
}
