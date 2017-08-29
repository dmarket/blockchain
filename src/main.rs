extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;

use exonum::blockchain::{self, Blockchain, Service, GenesisConfig, ValidatorKeys, Transaction, ApiContext};
use exonum::node::{Node, NodeConfig, NodeApiConfig, TransactionSend, ApiSender, NodeChannel};
use exonum::messages::{RawTransaction, FromRaw, Message};
use exonum::storage::{Fork, LevelDB, LevelDBOptions, MapIndex};
use exonum::crypto::{PublicKey, Hash};
use exonum::encoding::{self, Field};
use exonum::api::{Api, ApiError};
use iron::prelude::*;
use iron::Handler;
use iron::status;
use router::Router;

// Service identifier
const SERVICE_ID: u16 = 1;
// Identifier for wallet creation transaction type
const TX_CREATE_WALLET_ID: u16 = 1;
// Identifier for coins transfer transaction type
const TX_TRANSFER_ID: u16 = 2;
// Starting balance of a newly created wallet
const INIT_BALANCE: u64 = 100;

encoding_struct! {
    struct Wallet {
        const SIZE = 48;

        field pub_key:            &PublicKey  [00 => 32]
        field name:               &str        [32 => 40]
        field balance:            u64         [40 => 48]
    }
}

impl Wallet {
    pub fn increase(&mut self, amount: u64) {
        let balance = self.balance() + amount;
        Field::write(&balance, &mut self.raw, 40, 48);
    }

    pub fn decrease(&mut self, amount: u64) {
        let balance = self.balance() - amount;
        Field::write(&balance, &mut self.raw, 40, 48);
    }
}

pub struct CurrencySchema<'a> {
    view: &'a mut Fork,
}


impl<'a> CurrencySchema<'a> {
    pub fn wallets(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        let prefix = blockchain::gen_prefix(SERVICE_ID, 0, &());
        MapIndex::new(prefix, self.view)
    }

    // Utility method to quickly get a separate wallet from the storage
    pub fn wallet(&mut self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }
}

message! {
    struct TxCreateWallet {
        const TYPE = SERVICE_ID;
        const ID = TX_CREATE_WALLET_ID;
        const SIZE = 40;

        field pub_key:     &PublicKey  [00 => 32]
        field name:        &str        [32 => 40]
    }
}

message! {
    struct TxTransfer {
        const TYPE = SERVICE_ID;
        const ID = TX_TRANSFER_ID;
        const SIZE = 80;

        field from:        &PublicKey  [00 => 32]
        field to:          &PublicKey  [32 => 64]
        field amount:      u64         [64 => 72]
        field seed:        u64         [72 => 80]
    }
}


impl Transaction for TxCreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = CurrencySchema { view };
        if schema.wallet(self.pub_key()).is_none() {
            let wallet = Wallet::new(self.pub_key(), self.name(), INIT_BALANCE);
            println!("Create the wallet: {:?}", wallet);
            schema.wallets().put(self.pub_key(), wallet)
        }
    }
}

impl Transaction for TxTransfer {
    fn verify(&self) -> bool {
        (*self.from() != *self.to()) && self.verify_signature(self.from())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = CurrencySchema { view };
        let sender = schema.wallet(self.from());
        let receiver = schema.wallet(self.to());
        if let (Some(mut sender), Some(mut receiver)) = (sender, receiver) {
            let amount = self.amount();
            if sender.balance() >= amount {
                sender.decrease(amount);
                receiver.increase(amount);
                println!("Transfer between wallets: {:?} => {:?}", sender, receiver);
                let mut wallets = schema.wallets();
                wallets.put(self.from(), sender);
                wallets.put(self.to(), receiver);
            }
        }
    }
}


#[derive(Clone)]
struct CryptocurrencyApi {
    channel: ApiSender<NodeChannel>,
}

#[serde(untagged)]
#[derive(Clone, Serialize, Deserialize)]
enum TransactionRequest {
    CreateWallet(TxCreateWallet),
    Transfer(TxTransfer),
}

impl Into<Box<Transaction>> for TransactionRequest {
    fn into(self) -> Box<Transaction> {
        match self {
            TransactionRequest::CreateWallet(trans) => Box::new(trans),
            TransactionRequest::Transfer(trans) => Box::new(trans),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TransactionResponse {
    tx_hash: Hash,
}

#[derive(Serialize, Deserialize)]
struct AssetResponse {
    asset_id: String,
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
        let get_assets_handler = move |req: &mut Request| -> IronResult<Response> {
            let ref pub_key = req.extensions.get::<Router>().unwrap().find("pub_key").unwrap_or("/");

            let fake_assets = vec![AssetResponse { asset_id: "550e8400-e29b-41d4-a716-446655440000".to_owned() }, AssetResponse { asset_id: "110e8400-e29b-41d4-a716-446655440000".to_owned() }];

            self_.ok_response(&serde_json::to_value(&fake_assets).unwrap())
        };

        router.get("/asset/list/:pub_key", get_assets_handler, "get_assets");
    }
}

struct CurrencyService;

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
        let api = CryptocurrencyApi { channel: ctx.node_channel().clone() };
        api.wire(&mut router);
        Some(Box::new(router))
    }
}

fn main() {
    exonum::helpers::init_logger().unwrap();

    let mut options = LevelDBOptions::new();
    options.create_if_missing = true;

    let path = "/var/leveldb/dmc";
    let db = Box::new(LevelDB::open(path, options).unwrap());

    let services: Vec<Box<Service>> = vec![Box::new(CurrencyService)];
    let blockchain = Blockchain::new(db, services);

    /** Create Keys */
    let (consensus_public_key, consensus_secret_key) = exonum::crypto::gen_keypair();
    let (service_public_key, service_secret_key) = exonum::crypto::gen_keypair();

    /** Configure Node */
    let validator_keys = ValidatorKeys {
        consensus_key: consensus_public_key,
        service_key: service_public_key,
    };
    let genesis = GenesisConfig::new(vec![validator_keys].into_iter());
    let api_address = "0.0.0.0:8000".parse().unwrap();
    let api_cfg = NodeApiConfig {
        public_api_address: Some(api_address),
        ..Default::default()
    };
    let peer_address = "0.0.0.0:2000".parse().unwrap(); // for peer-to-peer
    // Complete node configuration
    let node_cfg = NodeConfig {
        listen_address: peer_address,
        peers: vec![],
        service_public_key,
        service_secret_key,
        consensus_public_key,
        consensus_secret_key,
        genesis,
        external_address: None,
        network: Default::default(),
        whitelist: Default::default(),
        api: api_cfg,
        mempool: Default::default(),
        services_configs: Default::default(),
    };

    let mut node = Node::new(blockchain, node_cfg);
    node.run().unwrap();
}
