use std::error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Error {
    AssetNotFound,
    TransactionNotFound,
    InvalidAssetInfo,
    InsufficientFunds,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::AssetNotFound => "asset not found",
            &Error::TransactionNotFound => "transaction not found",
            &Error::InvalidAssetInfo => "invalid asset info",
            &Error::InsufficientFunds => "insufficient funds",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", error::Error::description(self))
    }
}

