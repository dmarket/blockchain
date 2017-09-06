
const SERVICE_ID: u16 = 1;


// Create Wallet
const TX_CREATE_WALLET_ID: u16 = 1;
const INIT_BALANCE: u64 = 100;

pub mod create_wallet;


// Transfer
const TX_TRANSFER_ID: u16 = 2;

pub mod transfer;


// Add Assets
const TX_ADD_ASSETS_ID: u16 = 3;

pub mod add_assets;
