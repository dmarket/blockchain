//! Startup configuration.

extern crate toml;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::result::Result;

/// Representation of configuration file contents.
#[derive(Deserialize, Clone)]
#[warn(unused_must_use)]
pub struct Config {
    api: Api,
    db: Db,
    nats: Nats,
    service_discovery: ServiceDiscovery,
}

/// Node communications configuration.
#[derive(Deserialize, Clone)]
pub struct Api {
    current_node: Option<String>,
    address: Option<String>,
    private_address: Option<String>,
    keys_path: Option<String>,
    peer_address: Option<String>,
    peers: Option<Vec<String>>,
    is_validator: Option<bool>,
}

/// Database configuration.
#[derive(Deserialize, Clone)]
pub struct Db {
    path: Option<String>,
}

/// NATS reporting configuration.
#[derive(Deserialize, Clone)]
pub struct Nats {
    enabled: Option<bool>,
    addresses: Option<Vec<String>>,
    queuename: Option<String>,
}

/// Configuration for communicating with a global service discovery.
#[derive(Deserialize, Clone)]
pub struct ServiceDiscovery {
    address: Option<String>,
}

impl Config {
    /// Get `Api` configuration from the config file.
    pub fn api(self) -> Api {
        self.api
    }

    /// Get `Db` configuration from the config file.
    pub fn db(self) -> Db {
        self.db
    }

    /// Get `NATS` configuration from the config file.
    pub fn nats(self) -> Nats {
        self.nats
    }

    /// Get `ServiceDiscovery` configuration from the config file.
    pub fn service_discovery(self) -> ServiceDiscovery {
        self.service_discovery
    }
}

impl Api {
    /// Name of the current node.
    pub fn current_node(self) -> String {
        match env::var("CURRENT_NODE") {
            Ok(value) => value,
            Err(_) => self.current_node.unwrap(),
        }
    }

    /// Public API address of the current node.
    pub fn address(self) -> String {
        match env::var("API_ADDRESS") {
            Ok(value) => value,
            Err(_) => self.address.unwrap(),
        }
    }

    /// Private address of the current node.
    pub fn private_address(self) -> String {
        match env::var("API_PRIVATE_ADDRESS") {
            Ok(value) => value,
            Err(_) => self.private_address.unwrap(),
        }
    }

    /// Path to the directory with key files.
    pub fn keys_path(self) -> String {
        match env::var("API_KEYS_PATH") {
            Ok(value) => value,
            Err(_) => self.keys_path.unwrap(),
        }
    }

    /// Peer address for the current node.
    pub fn peer_address(self) -> String {
        match env::var("API_PEER_ADDRESS") {
            Ok(value) => value,
            Err(_) => self.peer_address.unwrap(),
        }
    }

    /// Existing peers of the current node.
    pub fn peers(self) -> Vec<SocketAddr> {
        match env::var("API_PEERS") {
            Ok(_) => vec![], // todo: add parse environment
            Err(_) => {
                let mut peers: Vec<SocketAddr> = vec![];
                for peer in self.peers.unwrap() {
                    match peer.to_socket_addrs() {
                        Ok(addr) => {
                            let mut a = addr;
                            peers.push(a.next().unwrap());
                        }
                        Err(e) => warn!("Error: {}", e),
                    }
                }
                peers
            }
        }
    }

    /// Checks whether this node will take part in consensus.
    pub fn is_validator(self) -> bool {
        match env::var("VALIDATOR") {
            Ok(value) => value.parse::<bool>().unwrap(),
            Err(_) => self.is_validator.unwrap_or(true),
        }
    }
}

impl Db {
    /// Path to the database.
    pub fn path(self) -> String {
        match env::var("DB_PATH") {
            Ok(value) => value,
            Err(_) => self.path.unwrap(),
        }
    }
}

impl Nats {
    /// Checks whether the current node send messages to NATS.
    pub fn enabled(self) -> bool {
        match env::var("NATS_ENABLED") {
            Ok(value) => if value == "false" {
                false
            } else {
                true
            },
            Err(_) => self.enabled.unwrap(),
        }
    }

    /// Addresses of NATS servers.
    pub fn addresses(self) -> Vec<String> {
        match env::var("NATS_ADDRESSES") {
            Ok(addresses) => addresses
                .split(',')
                .into_iter()
                .map(|a| a.to_string())
                .collect(),
            Err(_) => self.addresses.unwrap(),
        }
    }

    /// Name of the queue to which the messages shall be pushed.
    pub fn queuename(self) -> String {
        match env::var("NATS_QUEUENAME") {
            Ok(queuename) => queuename,
            Err(_) if self.queuename.is_none() => "dmbc.transaction.commit".to_string(),
            Err(_) => self.queuename.unwrap(),
        }
    }
}

impl ServiceDiscovery {
    /// Address of the service discovery.
    pub fn address(self) -> String {
        match env::var("SD_ADDRESS") {
            Ok(address) => address,
            Err(_) => self.address.unwrap(),
        }
    }
}

/// Load configuration
///
/// # Examples
///
/// ```no_run
/// # use dmbc::config;
/// let api_address = config::read_config().unwrap().api().address();
/// ```
pub fn read_config() -> Result<Config, Error> {
    let mut content = String::new();
    let path = env::var("CONFIG_PATH").unwrap_or("./etc/config.toml".to_string());
    let mut f = File::open(Path::new(&path))?;
    let _res = f.read_to_string(&mut content);
    Ok(toml::from_str(content.as_str()).unwrap())
}

lazy_static! {
    static ref CONFIG: Config = {
        read_config().unwrap()
    };
}

/// Read config from the config file.
pub fn config() -> Config {
    CONFIG.clone()
}

#[test]
fn positive() {
    assert_eq!(Some("node0".to_string()), config().api.current_node)
}

#[test]
fn env_positive() {
    let address = "1.1.1.1:1231";
    env::set_var("API_ADDRESS", address);
    assert_eq!(address, config().api().address().as_str())
}
