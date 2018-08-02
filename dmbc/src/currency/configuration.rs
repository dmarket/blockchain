//! Currency service configuration.

use serde_json;

use exonum::blockchain::Schema;
use exonum::crypto::{PublicKey, Hash};
use exonum::encoding::serialize::FromHex;
use exonum::storage::Snapshot;

use currency;

/// Transaction permission mask
/// 0 - Tx not Allowd, 1 - Tx allowed
pub const PM_ADD_ASSETSD:           u64 =           0b_1;
pub const PM_DELETE_ASSETS:         u64 =          0b_10;
pub const PM_EXCHANGE:              u64 =         0b_100;
pub const PM_EXCHANGE_INTERMEDIARY: u64 =        0b_1000;
pub const PM_TRADE:                 u64 =       0b_10000;
pub const PM_TRADE_INTERMEDIARY:    u64 =      0b_100000;
pub const PM_TRANSFER:              u64 =     0b_1000000;
pub const PM_TRANSFER_FEES:         u64 =    0b_10000000;
pub const PM_ALL_ALLOED:            u64 = <u64>::max_value();

encoding_struct! {
    /// Wallet tx permittion configuration
    #[derive(Eq, PartialOrd, Ord)]
    struct WalletPermissions {
        key: &PublicKey,
        mask: u64
    }
}

encoding_struct! {
    /// List of wallets that have permissions.
    #[derive(Default)]
    struct TxPermittionList {
        wallets: Vec<WalletPermissions>,
        digest: &Hash
    }
}

encoding_struct! {
    /// Fixed fees to be paid to the genesis wallet when transaction is executed.
    #[derive(Eq, PartialOrd, Ord)]
    struct TransactionFees {
        recipient:     &PublicKey,
        add_assets:           u64,
        add_assets_per_entry: u64,
        delete_assets:        u64,
        exchange:             u64,
        trade:                u64,
        transfer:             u64,
    }
}

impl TransactionFees {
    pub fn with_default_key(
        add_assets: u64,
        add_assets_per_entry: u64,
        delete_assets: u64,
        exchange: u64,
        trade: u64,
        transfer: u64,
    ) -> Self {
        TransactionFees::new(
            &PublicKey::from_hex(GENESIS_WALLET_PUB_KEY).unwrap(),
            add_assets,
            add_assets_per_entry,
            delete_assets,
            exchange,
            trade,
            transfer,
        )
    }
}

impl Default for TransactionFees {
    fn default() -> Self {
        TransactionFees::new(
            &PublicKey::from_hex(GENESIS_WALLET_PUB_KEY).unwrap(),
            0,
            0,
            0,
            0,
            0,
            0,
        )
    }
}

encoding_struct! {
    /// Currency service configuration.
    #[derive(Eq, PartialOrd, Ord)]
    struct Configuration {
        fees: TransactionFees,
        tx_permission_list: TxPermittionList
    }
}

/// Hexadecimal representation of the public key for genesis wallet.
pub const GENESIS_WALLET_PUB_KEY: &str =
    "36a05e418393fb4b23819753f6e6dd51550ce030d53842c43dd1349857a96a61";

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration::new(TransactionFees::default(), TxPermittionList::default())
    }
}

impl Configuration {
    /// Extract the `Configuration`.
    ///
    /// # Panics
    ///
    /// Panics if service configuration is invalid or absent.
    pub fn extract(snapshot: &Snapshot) -> Configuration {
        let schema = Schema::new(snapshot);
        let stored_configuration = schema.actual_configuration();

        match stored_configuration.services.get(currency::SERVICE_NAME) {
            Some(json) => serde_json::from_value(json.clone())
                .expect(&format!("Configuration is invalid: {:?}", json)),
            None => panic!(
                "No configuration for {} on the blockchain",
                currency::SERVICE_NAME
            ),
        }
    }
}
