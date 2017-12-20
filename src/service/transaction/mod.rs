use super::schema;
use super::wallet;

use super::SERVICE_ID;
//const SERVICE_ID: u16 = 1;

pub const TRANSACTION_FEE: u64 = 1000;
pub const TX_ADD_ASSET_FEE: u64 = 1000;
pub const PER_ADD_ASSET_FEE: u64 = 1;
pub const TX_DEL_ASSET_FEE: u64 = 100;
pub const TX_TRADE_FEE: u64 = 1000;
pub const MARKETPLACE_FEE: u64 = 0;
pub const PER_TRADE_ASSET_FEE: u64 = 40;
pub const TX_EXCHANGE_FEE: u64 = 1000;
pub const PER_EXCHANGE_ASSET_FEE: u64 = 1;
pub const TX_TRANSFER_FEE: u64 = 1000;

// Create Wallet
pub const TX_CREATE_WALLET_ID: u16 = 1;
const INIT_BALANCE: u64 = 100_000_000; // 1 DMC = 100_000_000 dimosh
pub mod create_wallet;

// Transfer
pub const TX_TRANSFER_ID: u16 = 2;
pub mod transfer;

// Add Assets
pub const TX_ADD_ASSETS_ID: u16 = 3;
pub mod add_assets;

// Add Assets
pub const TX_DEL_ASSETS_ID: u16 = 4;
pub mod del_assets;

// Buy Transaction
pub const TX_TRADE_ASSETS_ID: u16 = 5;
pub mod trade_assets;

// Buy Transaction
pub const TX_EXCHANGE_ID: u16 = 6;
pub mod exchange;

// Mining coin
pub const TX_MINING_ID: u16 = 7;
const AMOUNT_MINING_COIN: u64 = 100_000_000_000_000;
pub mod mining;

pub mod fee;
