//! Currency service configuration.

use serde_json;

use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;
use exonum::storage::Snapshot;
use exonum::blockchain::Schema;

use currency;

encoding_struct! {
    /// Fixed fees to be paid to the genesis wallet when transaction is executed.
    #[derive(Eq, PartialOrd, Ord)]
    struct TransactionFees {
        const SIZE = 80;

        field recipient:     &PublicKey [0  => 32]
        field add_assets:           u64 [32 => 40]
        field add_assets_per_entry: u64 [40 => 48]
        field delete_assets:        u64 [48 => 56]
        field exchange:             u64 [56 => 64]
        field trade:                u64 [64 => 72]
        field transfer:             u64 [72 => 80]
    }
}

encoding_struct! {
    /// Currency service configuration.
    #[derive(Eq, PartialOrd, Ord)]
    struct Configuration {
        const SIZE = 8;

        field fees: TransactionFees   [0 => 8]
    }
}

/// Hexadecimal representation of the public key for genesis wallet.
pub const GENESIS_WALLET_PUB_KEY: &str =
    "36a05e418393fb4b23819753f6e6dd51550ce030d53842c43dd1349857a96a61";

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration::new(
            TransactionFees::new(
                &PublicKey::from_hex(GENESIS_WALLET_PUB_KEY).unwrap(),
                0, 0, 0, 0, 0, 0,
            )
        )
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
