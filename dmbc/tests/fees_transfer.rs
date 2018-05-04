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
use dmbc_testkit::{DmbcTestApiBuilder, DmbcTestKitApi};

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::error::Error;

#[test]
fn fees_for_transfer() {
    let transaction_fee = 1000;
    let amount = 2;
    let fixed = 10;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);

    let (creator_key, _) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();
    let (sender_pub_key, sender_sec_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, amount, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &creator_key);

    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_to_wallet(&sender_pub_key, (asset.clone(), info))
        .create();
    let api = testkit.api();    

    let tx_transfer = transaction::Builder::new()
        .keypair(sender_pub_key, sender_sec_key)
        .tx_transfer()
        .add_asset_value(asset)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let (status, response) = api.post_fee(&tx_transfer);

    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + amount * fixed;
    expected.insert(sender_pub_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_transfer_sender_is_creator() {
    let transaction_fee = 1000;
    let amount = 2;
    let fixed = 10;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);

    let (recipient_key, _) = crypto::gen_keypair();
    let (sender_pub_key, sender_sec_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, amount, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &sender_pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_to_wallet(&sender_pub_key, (asset.clone(), info))
        .create();
    let api = testkit.api();    

    let tx_transfer = transaction::Builder::new()
        .keypair(sender_pub_key, sender_sec_key)
        .tx_transfer()
        .add_asset_value(asset)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let (status, response) = api.post_fee(&tx_transfer);

    let mut expected = HashMap::new();
    expected.insert(sender_pub_key, transaction_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_transfer_asset_not_found() {
        let transaction_fee = 1000;
    let amount = 2;
    let fixed = 10;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);

    let (creator_key, _) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();
    let (sender_pub_key, sender_sec_key) = crypto::gen_keypair();

    let (asset, _) = dmbc_testkit::create_asset(meta_data, amount, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &creator_key);

    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .create();
    let api = testkit.api(); 

    let tx_transfer = transaction::Builder::new()
        .keypair(sender_pub_key, sender_sec_key)
        .tx_transfer()
        .add_asset_value(asset)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let (status, response) = api.post_fee(&tx_transfer);

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Ok(Err(Error::AssetNotFound)));
}
