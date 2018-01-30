use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::{Once, ONCE_INIT};

use toml;

static mut CONFIG: *const Config = 0usize as *const _;
static CONFIG_INIT: Once = ONCE_INIT;

#[derive(Deserialize, Clone, Default)]
pub struct Config {
    listen_address: Option<String>,
    peers_path: Option<String>,
}

impl Config {
    pub fn listen_address(&self) -> &str {
        self.listen_address.as_ref().unwrap()
    }

    pub fn peers_path(&self) -> &str {
        self.peers_path.as_ref().unwrap()
    }
}

pub fn get() -> &'static Config {
    fn read_env_config() -> Config {
        Config {
            listen_address: env::var("DISCOVERY_LISTEN_ADDRESS").ok(),
            peers_path: env::var("DISCOVERY_PEERS_PATH").ok(),
        }
    }

    fn read_file_config() -> Config {
        let path = env::var("DISCOVERY_CONFIG_PATH").unwrap_or("./etc/discovery.toml".to_string());
        let mut file = File::open(path).unwrap();
        let mut buf = Vec::new();

        let _ = file.read_to_end(&mut buf);
        toml::from_slice(&buf).unwrap_or_default()
    }

    CONFIG_INIT.call_once(|| {
        let config = {
            let cfg = read_env_config();
            match &cfg {
                &Config {
                    listen_address: Some(_),
                    peers_path: Some(_),
                } => cfg,
                _ => {
                    let file_cfg = read_file_config();
                    let listen_address = cfg.listen_address.or(file_cfg.listen_address);
                    let peers_path = cfg.peers_path.or(file_cfg.peers_path);
                    Config {
                        listen_address,
                        peers_path,
                    }
                }
            }
        };

        let ptr = Box::into_raw(Box::new(config));

        // Mutable write is safe in once_init().
        unsafe {
            CONFIG = ptr;
        };
    });

    // Immutably referencing static data is safe.
    unsafe { &*CONFIG }
}
