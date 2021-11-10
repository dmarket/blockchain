//! Types and operations on wallets in the blockchain network.

mod schema;
mod wallet;

pub use currency::wallet::schema::{Schema, move_assets};
pub use currency::wallet::wallet::{move_coins, Wallet};
