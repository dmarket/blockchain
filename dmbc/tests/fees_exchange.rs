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
use evo_testkit::{EvoTestKit, EvoTestKitApi, asset_fees, create_asset};

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::error::Error;
use dmbc::currency::transactions::components::FeeStrategy;

#[test]
fn fees_for_exchange_recipient() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let balance = 100_000_000;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();

    testkit.create_wallet(&sender_public_key, balance);
    testkit.create_wallet(&recipient_public_key, balance);    

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &creator_pub_key);

    testkit.add_assets(
        &sender_public_key, 
        vec![asset0.clone(), asset1.clone(), asset2.clone()], 
        vec![info0.clone(), info1.clone(), info2.clone()]
    );
    testkit.add_assets(
        &recipient_public_key, 
        vec![asset3.clone()], 
        vec![info3.clone()]
    );

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange()
        .sender(sender_public_key)
        .sender_secret(sender_secret_key)
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
fn fees_for_exchange_sender() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let balance = 100_000_000;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();

    testkit.create_wallet(&sender_public_key, balance);
    testkit.create_wallet(&recipient_public_key, balance);    

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &creator_pub_key);

    testkit.add_assets(
        &sender_public_key, 
        vec![asset0.clone(), asset1.clone(), asset2.clone()], 
        vec![info0.clone(), info1.clone(), info2.clone()]
    );
    testkit.add_assets(
        &recipient_public_key, 
        vec![asset3.clone()], 
        vec![info3.clone()]
    );

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange()
        .sender(sender_public_key)
        .sender_secret(sender_secret_key)
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
fn fees_for_exchange_recipient_and_sender() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let balance = 100_000_000;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();

    testkit.create_wallet(&sender_public_key, balance);
    testkit.create_wallet(&recipient_public_key, balance);    

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &creator_pub_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &creator_pub_key);

    testkit.add_assets(
        &sender_public_key, 
        vec![asset0.clone(), asset1.clone(), asset2.clone()], 
        vec![info0.clone(), info1.clone(), info2.clone()]
    );
    testkit.add_assets(
        &recipient_public_key, 
        vec![asset3.clone()], 
        vec![info3.clone()]
    );

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange()
        .sender(sender_public_key)
        .sender_secret(sender_secret_key)
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
fn fees_for_exchange_recipient_and_sender_creator() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let balance = 100_000_000;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();

    testkit.create_wallet(&sender_public_key, balance);
    testkit.create_wallet(&recipient_public_key, balance);    

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &sender_public_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &sender_public_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &sender_public_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &sender_public_key);

    testkit.add_assets(
        &sender_public_key, 
        vec![asset0.clone(), asset1.clone(), asset2.clone()], 
        vec![info0.clone(), info1.clone(), info2.clone()]
    );
    testkit.add_assets(
        &recipient_public_key, 
        vec![asset3.clone()], 
        vec![info3.clone()]
    );

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange()
        .sender(sender_public_key)
        .sender_secret(sender_secret_key)
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
fn fees_for_exchange_invalid_transaction() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let balance = 100_000_000;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();

    testkit.create_wallet(&sender_public_key, balance);
    testkit.create_wallet(&recipient_public_key, balance);    

    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &sender_public_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &sender_public_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &sender_public_key);
    let (asset3, info3) = create_asset(meta_data3, units, asset_fees(tax, 0), &sender_public_key);

    testkit.add_assets(
        &sender_public_key, 
        vec![asset0.clone(), asset1.clone(), asset2.clone()], 
        vec![info0.clone(), info1.clone(), info2.clone()]
    );
    testkit.add_assets(
        &recipient_public_key, 
        vec![asset3.clone()], 
        vec![info3.clone()]
    );

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange()
        .sender(sender_public_key)
        .sender_secret(sender_secret_key)
        .fee_strategy(FeeStrategy::Intermediary)
        .sender_add_asset_value(asset0)
        .sender_add_asset_value(asset1)
        .sender_add_asset_value(asset2)
        .recipient_add_asset_value(asset3)
        .build();

    let (status, response) = api.post_fee(&tx_exchange_assets);

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Ok(Err(Error::InvalidTransaction)));
}

#[test]
fn fees_for_exchange_asset_not_found() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let balance = 100_000_000;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (sender_public_key, sender_secret_key) = crypto::gen_keypair();
    let (recipient_public_key, recipient_secret_key) = crypto::gen_keypair();

    testkit.create_wallet(&sender_public_key, balance);
    testkit.create_wallet(&recipient_public_key, balance);    

    let (asset0, _) = create_asset(meta_data0, units, asset_fees(tax, 0), &sender_public_key);
    let (asset1, _) = create_asset(meta_data1, units, asset_fees(tax, 0), &sender_public_key);
    let (asset2, _) = create_asset(meta_data2, units, asset_fees(tax, 0), &sender_public_key);
    let (asset3, _) = create_asset(meta_data3, units, asset_fees(tax, 0), &sender_public_key);

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_public_key, recipient_secret_key)
        .tx_exchange()
        .sender(sender_public_key)
        .sender_secret(sender_secret_key)
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