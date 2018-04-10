extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::helpers::{Height, ValidatorId};
use exonum_testkit::TestKitBuilder;

use dmbc::currency;
use dmbc::currency::configuration::{Configuration, TransactionFees};

/*
#[test]
#[should_panic]
fn default_service_configuration() {
    let mut testkit = TestKitBuilder::validator()
        .with_validators(1)
        .with_service(currency::Service::new())
        .create();

    testkit.create_block();
    let fork = testkit.blockchain_mut().fork();
    let configuration = Configuration::extract(&fork);
    assert_eq!(configuration, Configuration::default()); //todo: it's fail
}
*/

#[test]
fn proposed_service_configuration() {
    let mut testkit = TestKitBuilder::auditor().with_validators(3).create();

    let configuration = Configuration::new(TransactionFees::with_default_key(100, 2, 100, 100, 100, 100));
    let cfg_change_height = Height(5);
    let proposal = {
        let mut cfg = testkit.configuration_change_proposal();
        // Add us to validators.
        let mut validators = cfg.validators().to_vec();
        validators.push(testkit.network().us().clone());
        cfg.set_validators(validators);
        // Change configuration of our service.
        cfg.set_service_config(&currency::SERVICE_NAME, configuration.clone());
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
