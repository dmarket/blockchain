use std::error;
use std::fmt;

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Error {
    AssetNotFound = 1,
    TransactionNotFound = 2,
    InvalidAssetInfo = 3,
    InsufficientFunds = 4,
}

impl Error {
    pub fn try_from(value: u8) -> Option<Self> {
        match value {
            1 => Some(Error::AssetNotFound),
            2 => Some(Error::TransactionNotFound),
            3 => Some(Error::InvalidAssetInfo),
            4 => Some(Error::InsufficientFunds),
            _ => None,
        }
    }
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

