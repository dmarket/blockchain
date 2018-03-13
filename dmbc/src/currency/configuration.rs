//! Currency service configuration.

use serde_json;

use exonum::storage::Snapshot;
use exonum::blockchain::Schema;

use currency;

encoding_struct! {
    /// Fised fees to be paid to the genesis wallet when transaction is executed.
    #[derive(Eq, PartialOrd, Ord)]
    struct TransactionFees {
        const SIZE = 48;

        field add_asset:     u64 [0 => 8]
        field per_add_asset: u64 [8 => 16]
        field del_asset:     u64 [16 => 24]
        field exchange:      u64 [24 => 32]
        field trade:         u64 [32 => 40]
        field transfer:      u64 [40 => 48]
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

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration::new(TransactionFees::new(0, 0, 0, 0, 0, 0))
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
