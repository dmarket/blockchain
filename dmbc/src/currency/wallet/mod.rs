//! Types and operations on wallets in the blockchain network.

mod schema;
mod wallet;

pub use currency::wallet::schema::Schema;
pub use currency::wallet::wallet::{move_assets, move_coins,
                                   create_ask, create_bid,
                                   Wallet};
