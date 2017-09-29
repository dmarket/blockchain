use service::api::{SERVICE_ID};
//const SERVICE_ID: u16 = 1;

// Create Wallet
pub const TX_CREATE_WALLET_ID: u16 = 1;
const INIT_BALANCE: u64 = 100;
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
pub const  TX_TRADE_ASSETS_ID: u16 = 5;
pub mod trade_assets;
