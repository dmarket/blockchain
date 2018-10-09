extern crate curl;
extern crate exonum;
extern crate exonum_configuration;
extern crate exonum_rocksdb;
extern crate serde;
extern crate serde_derive;
extern crate clap;
extern crate serde_json;

extern crate dmbc;

mod flag;
mod keyfile;

use dmbc::config;
use dmbc::currency::Service;
use exonum::blockchain;
use exonum::blockchain::{ConsensusConfig, GenesisConfig, TimeoutAdjusterConfig, ValidatorKeys};
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;
use exonum::node::{Node, NodeApiConfig, NodeConfig};
use exonum::storage::{RocksDB, RocksDBOptions};
use exonum_configuration::ConfigurationService;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const GENESIS_VALIDATOR_PUBLIC: &str =
    "4e298e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411b9f";
const GENESIS_SERVICE_PUBLIC: &str =
    "68e774a4339cccfae644dcf3e44360839c84a6475c7d2943ed59b81d7eb6e9f0";

fn main() {
    let _f = match flag::parse() {
        Some(f) => f,
        None => ::std::process::exit(0),
    };

    exonum::helpers::init_logger().unwrap();

    /** Create Keys */
    println!("Initializing node version: v{}", VERSION);

    let (consensus_public_key, consensus_secret_key) = keyfile::pair("consensus").unwrap();
    let (service_public_key, service_secret_key) = keyfile::pair("service").unwrap();

    let public_api = config::config().api().address();
    let private_api = config::config().api().private_address();

    let is_validator = config::config().api().is_validator();
    eprintln!(
        "Connecting {}",
        if is_validator {
            "as validator"
        } else {
            "as auditor"
        }
    );

    let consensus_config = ConsensusConfig {
        round_timeout: 3500,
        status_timeout: 5000,
        peers_timeout: 10_000,
        txs_block_limit: 3000,
        max_message_len: ConsensusConfig::DEFAULT_MESSAGE_MAX_LEN,
        timeout_adjuster: TimeoutAdjusterConfig::Constant { timeout: 2500 },
    };

    // Configure Node
    let validators = Some(ValidatorKeys {
        consensus_key: PublicKey::from_hex(GENESIS_VALIDATOR_PUBLIC).unwrap(),
        service_key: PublicKey::from_hex(GENESIS_SERVICE_PUBLIC).unwrap(),
    });

    let genesis = GenesisConfig::new_with_consensus(consensus_config, validators.into_iter());
    let api_cfg = NodeApiConfig {
        public_api_address: Some(public_api.parse().unwrap()),
        private_api_address: Some(private_api.parse().unwrap()),
        ..Default::default()
    };

    let peers = config::config().api().peers();

    // Complete node configuration
    let node_cfg = NodeConfig {
        listen_address: config::config().api().peer_address().parse().unwrap(),
        peers,
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

    // Initialize database
    let mut options = RocksDBOptions::default();
    options.create_if_missing(true);
    options.enable_statistics();
    if cfg!(target_os = "linux") {
        use exonum_rocksdb::DBCompressionType;
        options.set_compression_type(DBCompressionType::Zlib);
    }
    let path = config::config().db().path();
    let db = Box::new(RocksDB::open(path, &options).unwrap());

    // Initialize services
    let services: Vec<Box<blockchain::Service>> = vec![
        Box::new(ConfigurationService::new()),
        Box::new(Service::new()),
    ];

    eprintln!("Launching node. What can possibly go wrong?");

    let node = Node::new(db, services, node_cfg);
    node.run().unwrap();
}
