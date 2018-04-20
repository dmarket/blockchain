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
use dmbc_testkit::{DmbcTestApiBuilder, DmbcTestKitApi, asset_fees, create_asset};

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::error::Error;
use dmbc::currency::transactions::components::FeeStrategy;

#[test]
fn fees_for_exchange_intermediary_recipient() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_to_wallet(&sender_public_key, (asset0.clone(), info0))
        .add_asset_to_wallet(&sender_public_key, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_public_key, (asset2.clone(), info2))
        .add_asset_to_wallet(&recipient_public_key, (asset3.clone(), info3))
        .create();
    let api = testkit.api();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange_with_intermediary()
        .sender_key_pair(sender_public_key, sender_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .sender_add_asset_value(asset0)
        .sender_add_asset_value(asset1)
        .sender_add_asset_value(asset2)
        .recipient_add_asset_value(asset3)
        .build();

    let (status, response) = api.post_fee(&tx_exchange_assets);

    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax * units * 4;
    expected.insert(recipient_public_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_exchange_intermediary_sender() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_to_wallet(&sender_public_key, (asset0.clone(), info0))
        .add_asset_to_wallet(&sender_public_key, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_public_key, (asset2.clone(), info2))
        .add_asset_to_wallet(&recipient_public_key, (asset3.clone(), info3))
        .create();
    let api = testkit.api();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange_with_intermediary()
        .sender_key_pair(sender_public_key, sender_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::Sender)
        .sender_add_asset_value(asset0)
        .sender_add_asset_value(asset1)
        .sender_add_asset_value(asset2)
        .recipient_add_asset_value(asset3)
        .build();

    let (status, response) = api.post_fee(&tx_exchange_assets);

    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax * units * 4;
    expected.insert(sender_public_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_exchange_intermediary_recipient_and_sender() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_to_wallet(&sender_public_key, (asset0.clone(), info0))
        .add_asset_to_wallet(&sender_public_key, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_public_key, (asset2.clone(), info2))
        .add_asset_to_wallet(&recipient_public_key, (asset3.clone(), info3))
        .create();
    let api = testkit.api();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange_with_intermediary()
        .sender_key_pair(sender_public_key, sender_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(asset0)
        .sender_add_asset_value(asset1)
        .sender_add_asset_value(asset2)
        .recipient_add_asset_value(asset3)
        .build();

    let (status, response) = api.post_fee(&tx_exchange_assets);

    let mut expected = HashMap::new();
    let expected_fee = transaction_fee / 2 + tax * units * 2;
    expected.insert(sender_public_key, expected_fee);
    expected.insert(recipient_public_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_exchange_intermediary_intermediary() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_to_wallet(&sender_public_key, (asset0.clone(), info0))
        .add_asset_to_wallet(&sender_public_key, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_public_key, (asset2.clone(), info2))
        .add_asset_to_wallet(&recipient_public_key, (asset3.clone(), info3))
        .create();
    let api = testkit.api();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange_with_intermediary()
        .sender_key_pair(sender_public_key, sender_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::Intermediary)
        .sender_add_asset_value(asset0)
        .sender_add_asset_value(asset1)
        .sender_add_asset_value(asset2)
        .recipient_add_asset_value(asset3)
        .build();

    let (status, response) = api.post_fee(&tx_exchange_assets);

    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax * units * 4;
    expected.insert(intermediary_public_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_exchange_intermediary_recipient_and_sender_creator() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &sender_public_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &sender_public_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &sender_public_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &sender_public_key);

    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_to_wallet(&sender_public_key, (asset0.clone(), info0))
        .add_asset_to_wallet(&sender_public_key, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_public_key, (asset2.clone(), info2))
        .add_asset_to_wallet(&recipient_public_key, (asset3.clone(), info3))
        .create();
    let api = testkit.api();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange_with_intermediary()
        .sender_key_pair(sender_public_key, sender_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(asset0)
        .sender_add_asset_value(asset1)
        .sender_add_asset_value(asset2)
        .recipient_add_asset_value(asset3)
        .build();

    let (status, response) = api.post_fee(&tx_exchange_assets);

    let mut expected = HashMap::new();
    let expected_sender_fee = transaction_fee / 2;
    let expected_recipient_fee = transaction_fee / 2 + tax * units * 2;
    expected.insert(sender_public_key, expected_sender_fee);
    expected.insert(recipient_public_key, expected_recipient_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_exchange_intermediary_asset_not_found() {
let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset0, _) = create_asset(meta_data0, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset1, _) = create_asset(meta_data1, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset2, _) = create_asset(meta_data2, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset3, _) = create_asset(meta_data3, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .create();
    let api = testkit.api();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange_with_intermediary()
        .sender_key_pair(sender_public_key, sender_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .sender_add_asset_value(asset0)
        .sender_add_asset_value(asset1)
        .sender_add_asset_value(asset2)
        .recipient_add_asset_value(asset3)
        .build();

    let (status, response) = api.post_fee(&tx_exchange_assets);

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Ok(Err(Error::AssetNotFound)));
}