extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate mount;

pub mod dmbc_testkit;

use std::collections::HashMap;

use hyper::status::StatusCode;
use exonum::crypto;
use dmbc_testkit::{DmbcTestKitApi, DmbcTestApiBuilder};

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::assets::MetaAsset;
use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;

#[test]
fn fees_for_add_assets() {
    let transaction_fee = 100;
    let per_asset_fee = 4;
    let amount = 5;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);
    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .create();
    
    let api = testkit.api();

    let (public_key, secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let fees = dmbc_testkit::asset_fees(10, 10);

    let meta_data = "asset";
    let meta_asset = MetaAsset::new(&receiver_key, meta_data, amount, fees);

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(85)
        .build();

    let (status, response) = api.post_fee(&tx_add_assets);

    let mut expected = HashMap::new();
    expected.insert(public_key, transaction_fee + amount * per_asset_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}