extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate mount;

pub mod evo_testkit;

use std::collections::HashMap;

use hyper::status::StatusCode;
use exonum::crypto;
use exonum_testkit::TestKit;
use evo_testkit::{EvoTestKit, EvoTestKitApi};

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;

#[test]
fn fees_for_delete_assets() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);
    
    testkit.set_configuration(Configuration::new(config_fees));

    let meta_data = "asset";
    let (public_key, secret_key) = crypto::gen_keypair();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, 5)
        .seed(85)
        .build();

    let (status, response) = api.post_fee(&tx_delete_assets);

    let mut expected = HashMap::new();
    expected.insert(public_key, transaction_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}