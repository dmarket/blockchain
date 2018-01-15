use serde_json;

use exonum::storage::Snapshot;
use exonum::blockchain::Schema;

use service;

const TX_ADD_ASSET_FEE: u64 = 1000;
const PER_ADD_ASSET_FEE: u64 = 1;
const TX_DEL_ASSET_FEE: u64 = 100;
const TX_TRADE_FEE: u64 = 1000;
const MARKETPLACE_FEE: u64 = 0;
const PER_TRADE_ASSET_FEE: u64 = 40; // 1/40 = 0.025
const TX_EXCHANGE_FEE: u64 = 1000;
const PER_EXCHANGE_ASSET_FEE: u64 = 40;
const TX_TRANSFER_FEE: u64 = 1000;

encoding_struct! {
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
    struct Configuration {
        const SIZE = 8;

        field fees: TransactionFees   [0 => 8]
    }
}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration::new(TransactionFees::new(
            TX_ADD_ASSET_FEE,
            PER_ADD_ASSET_FEE,
            TX_DEL_ASSET_FEE,
            TX_EXCHANGE_FEE,
            TX_TRADE_FEE,
            TX_TRANSFER_FEE,
        ))
    }
}

impl Configuration {
    /// Returns `Configuration` for the service.
    ///
    /// If configuration is invalid or not stored on the blockchain,
    /// default configuration will be returned.
    pub fn extract(snapshot: &Snapshot) -> Configuration {
        let schema = Schema::new(snapshot);
        let stored_configuration = schema.actual_configuration();

        match stored_configuration.services.get(service::SERVICE_NAME) {
            Some(json) => serde_json::from_value(json.clone()).unwrap_or(Configuration::default()),
            None => Configuration::default(),
        }
    }

    /// Returns `Ok(())` if `snapshot contains valid configuration for service.
    /// Othervise, returns `Err(serde_json::Error)`.
    ///
    /// Valid configuration has two states:
    /// 1. When json value is `null`
    /// 2. When json value is convertible to `Configuration`.
    pub fn is_valid(snapshot: &Snapshot) -> Result<(), serde_json::Error> {
        let schema = Schema::new(snapshot);
        let stored_configuration = schema.actual_configuration();

        match stored_configuration.services.get(service::SERVICE_NAME) {
            Some(json) => {
                let result: Result<Configuration, serde_json::Error> =
                    serde_json::from_value(json.clone());
                if !json.is_null() && result.is_err() {
                    return Err(result.err().unwrap());
                }
                Ok(())
            }
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use exonum::helpers::{Height, ValidatorId};
    use exonum_testkit::TestKitBuilder;

    use service;
    use service::configuration::*;

    #[test]
    fn default_service_configuration() {
        let mut testkit = TestKitBuilder::validator().with_validators(1).create();

        testkit.create_block();
        let fork = testkit.blockchain_mut().fork();
        let configuration = Configuration::extract(&fork);
        assert_eq!(configuration, Configuration::default());
    }

    #[test]
    fn proposed_service_configuration() {
        let mut testkit = TestKitBuilder::auditor().with_validators(3).create();

        let configuration = Configuration::new(TransactionFees::new(100, 2, 100, 100, 100, 100));
        let cfg_change_height = Height(5);
        let proposal = {
            let mut cfg = testkit.configuration_change_proposal();
            // Add us to validators.
            let mut validators = cfg.validators().to_vec();
            validators.push(testkit.network().us().clone());
            cfg.set_validators(validators);
            // Change configuration of our service.
            cfg.set_service_config(service::SERVICE_NAME, configuration.clone());
            // Set the height with which the configuration takes effect.
            cfg.set_actual_from(cfg_change_height);
            cfg
        };

        // Save proposed configuration.
        let stored = proposal.stored_configuration().clone();
        // Commit configuration change proposal to the testkit.
        testkit.commit_configuration_change(proposal);
        // Create blocks up to the height preceding the `actual_from` height.
        testkit.create_blocks_until(cfg_change_height.previous());
        // Check that the proposal has become actual.
        assert_eq!(testkit.network().us().validator_id(), Some(ValidatorId(3)));
        assert_eq!(testkit.validator(ValidatorId(3)), testkit.network().us());
        assert_eq!(testkit.actual_configuration(), stored);

        let fork = testkit.blockchain_mut().fork();
        assert_eq!(Configuration::extract(&fork), configuration);
    }
}
