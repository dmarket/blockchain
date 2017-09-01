extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;

mod service;

use exonum::blockchain::{Blockchain, Service, GenesisConfig, ValidatorKeys};
use exonum::node::{Node, NodeConfig, NodeApiConfig};
use exonum::storage::{LevelDB, LevelDBOptions};
use service::CurrencyService;

fn main() {
    exonum::helpers::init_logger().unwrap();

    let mut options = LevelDBOptions::new();
    options.create_if_missing = true;

    let path = "/var/db/leveldb/dmc";
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
