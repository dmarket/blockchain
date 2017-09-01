#[cfg(test)]

extern crate toml;

use std::result::Result;
use std::io::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub api: Api
}

#[derive(Deserialize)]
pub struct Api {
    pub address: Option<String>
}

///
/// Load configuration
///
/// # Examples
///
/// ```
/// assert_eq!("0.0.0.0:8000", config().ok().unwrap().api.address.unwrap().as_str())
/// ```
pub fn config() -> Result<Config, Error> {
    let mut content: String = String::new();
    let mut f = File::open(Path::new("./config.toml"))?;
    f.read_to_string(&mut content);
    Ok(toml::from_str(content.as_str()).unwrap())
}

#[test]
fn positive() {
    assert_eq!("0.0.0.0:8000", config().ok().unwrap().api.address.unwrap().as_str())
}