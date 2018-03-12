//! Service errors.

use std::error;
use std::fmt;

/// Enumeration of errors that can happen when processing a transaction.
#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Error {
    /// Asset ID is not found in the network.
    AssetNotFound = 1,

    /// Transaction hash is not found in the network.
    TransactionNotFound = 2,

    /// AssetInfo is invalid.
    InvalidAssetInfo = 3,

    /// There is not enough funds on wallet for the operation to succeed.
    InsufficientFunds = 4,

    /// Wallet does not have the amount of assets of specified type required
    /// for the operation to succeed.
    InsufficientAssets = 5,

    /// Transaction is invalid
    InvalidTransaction = 6,

    /// Requested operation is not implemented. Must not happen in production
    /// setting.
    NotImplemented = 255,
}

impl Error {
    /// Try creating an error from its u8 representation.
    pub fn try_from(value: u8) -> Option<Self> {
        match value {
            1 => Some(Error::AssetNotFound),
            2 => Some(Error::TransactionNotFound),
            3 => Some(Error::InvalidAssetInfo),
            4 => Some(Error::InsufficientFunds),
            5 => Some(Error::InsufficientAssets),
            6 => Some(Error::InvalidTransaction),
            255 => Some(Error::NotImplemented),
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
            &Error::InsufficientAssets => "insufficient assets",
            &Error::NotImplemented => "not implemented",
            &Error::InvalidTransaction => "invalid transaction",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", error::Error::description(self))
    }
}
