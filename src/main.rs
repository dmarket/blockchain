#![allow(dead_code)]
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate exonum;
extern crate exonum_configuration;
extern crate router;
extern crate bodyparser;
extern crate iron;
extern crate nats;

mod service;
mod config;
mod keys;

use exonum::blockchain::{Blockchain, Service, GenesisConfig, ValidatorKeys, ConsensusConfig,
                         TimeoutAdjusterConfig};
use exonum::node::{Node, NodeConfig, NodeApiConfig};
use exonum::storage::{LevelDB, LevelDBOptions};
use exonum_configuration::ConfigurationService;
use service::CurrencyService;
use keys::{KeyPair, Disc};

fn main() {
    exonum::helpers::init_logger().unwrap();

    let mut options = LevelDBOptions::new();
    options.create_if_missing = true;

    let path = config::config().db().path();
    let db = Box::new(LevelDB::open(path, options).unwrap());

    let services: Vec<Box<Service>> = vec![
        Box::new(ConfigurationService::new()),
        Box::new(CurrencyService),
    ];
    let blockchain = Blockchain::new(db, services);

    /** Create Keys */
    println!("Current node: {}", config::config().api().current_node());
    let consensus_name: String = config::config().api().current_node() + "_consensus.json";
    let service_name: String = config::config().api().current_node() + "_service.json";

    let consensus_keys = KeyPair::read(&consensus_name);
    let service_keys = KeyPair::read(&service_name);

    let consensus_public_key = consensus_keys.public;
    let consensus_secret_key = consensus_keys.secret;
    let service_public_key = service_keys.public;
    let service_secret_key = service_keys.secret;

    let nodenames = vec!["node0" /*, "node1", "node2", "node3"*/];
    let mut validators: Vec<ValidatorKeys> = vec![];
    for name_prefix in nodenames {
        let consensus_name = name_prefix.to_string() + "_consensus.json";
        let service_name = name_prefix.to_string() + "_service.json";
        let consensus_keys = KeyPair::read(&consensus_name);
        let service_keys = KeyPair::read(&service_name);
        validators.push(ValidatorKeys {
            consensus_key: consensus_keys.public,
            service_key: service_keys.public,
        });
    }

    let consensus_config = ConsensusConfig {
        round_timeout: 3000,
        status_timeout: 5000,
        peers_timeout: 10_000,
        txs_block_limit: 1000,
        timeout_adjuster: TimeoutAdjusterConfig::Dynamic {
            min: 200,
            max: 1000,
            threshold: 1,
        },
    };

    /* Configure Node */
    let genesis = GenesisConfig::new_with_consensus(consensus_config, validators.into_iter());
    let api_cfg = NodeApiConfig {
        public_api_address: Some(config::config().api().address().parse().unwrap()),
        private_api_address: Some(config::config().api().private_address().parse().unwrap()),
        ..Default::default()
    };

    // Complete node configuration
    let node_cfg = NodeConfig {
        listen_address: config::config().api().peer_address().parse().unwrap(),
        peers: config::config().api().peers(),
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
