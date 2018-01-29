use super::schema;
use super::wallet;

use super::SERVICE_ID;
//const SERVICE_ID: u16 = 1;

pub const TRANSACTION_FEE: u64 = 1000;

pub mod utils;
pub mod fee;
pub mod intermediary;

// Create Wallet
pub const TX_CREATE_WALLET_ID: u16 = 1;
pub const INIT_BALANCE: u64 = 100_000_000; // 1 DMC = 100_000_000 dimosh
pub mod create_wallet;

// Transfer
pub const TX_TRANSFER_ID: u16 = 2;
pub mod transfer;

// Add Assets
pub const TX_ADD_ASSETS_ID: u16 = 3;
pub mod add_assets;

// Delete Assets
pub const TX_DEL_ASSETS_ID: u16 = 4;
pub mod del_assets;

// Trade Ask/Bid Assets
pub const TX_TRADE_ASK_ASSETS_ID: u16 = 501;
pub mod trade_ask_assets;

// Trade Assets
pub const TX_TRADE_ASSETS_ID: u16 = 502;
pub mod trade_assets;

// Trade Ask/Bid Assets with intermediary
pub const TX_TRADE_ASK_ASSETS_WITH_INTERMEDIARY_ID: u16 = 503;
pub mod trade_ask_assets_with_intermediary;

// Trade Assets with intermediary party
pub const TX_TRADE_ASSETS_WITH_INTERMEDIARY_ID: u16 = 504;
pub mod trade_assets_with_intermediary;

// Exchange Assets
pub const TX_EXCHANGE_ID: u16 = 601;
pub mod exchange;

// Exchange Assets with intermediary party
pub const TX_EXCHANGE_WITH_INTERMEDIARY_ID: u16 = 602;
pub mod exchange_with_intermediary;

// Mining coin
pub const TX_MINING_ID: u16 = 7;
const AMOUNT_MINING_COIN: u64 = 100_000_000_000_000;
pub mod mining;
