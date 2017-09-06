extern crate toml;

use std::env;
use std::result::Result;
use std::io::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    api: Api
}

#[derive(Deserialize)]
struct Api {
    address: Option<String>
}

impl Config {
    pub fn api(self) -> Api {
        self.api
    }
}

impl Api {
    pub fn address(self) -> String {
        match env::var("API_ADDRESS") {
            Ok(value) => value,
            Err(_) => self.address.unwrap()
        }
    }
}

///
/// Load configuration
///
/// # Examples
///
/// ```
/// assert_eq!("0.0.0.0:8000", read_config().ok().unwrap().api.address.unwrap().as_str())
/// ```
pub fn read_config() -> Result<Config, Error> {
    let mut content: String = String::new();
    let mut f = File::open(Path::new("./config.toml"))?;
    f.read_to_string(&mut content);
    Ok(toml::from_str(content.as_str()).unwrap())
}

pub fn config() -> Config {
    read_config().ok().unwrap()
}

#[test]
fn positive() {
    assert_eq!("0.0.0.0:8000", config().api.address.unwrap().as_str())
}

#[test]
fn env_positive() {
    let address = "1.1.1.1:1231";
    env::set_var("API_ADDRESS", address);
    assert_eq!(address, config().api.address().as_str())
}