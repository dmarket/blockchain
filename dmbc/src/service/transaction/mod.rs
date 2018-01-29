use super::schema;
use super::wallet;

use super::SERVICE_ID;
//const SERVICE_ID: u16 = 1;

pub const TRANSACTION_FEE: u64 = 1000;

pub mod fee;
pub mod intermediary;

// Create Wallet
pub mod create_wallet;

// Transfer
pub mod transfer;

// Add Assets
pub mod add_assets;

// Delete Assets
pub mod del_assets;

// Trade Ask/Bid Assets
pub mod trade_ask_assets;

// Trade Assets
pub mod trade_assets;

// Trade Ask/Bid Assets with intermediary
pub mod trade_ask_assets_with_intermediary;

// Trade Assets with intermediary party
pub mod trade_assets_with_intermediary;

// Exchange Assets
pub mod exchange;

// Exchange Assets with intermediary party
pub mod exchange_with_intermediary;

// Mining coin
pub mod mining;
