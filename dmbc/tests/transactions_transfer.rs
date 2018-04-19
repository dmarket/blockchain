extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate mount;

pub mod evo_testkit;

use hyper::status::StatusCode;
use exonum::messages::Message;
use exonum::crypto;
use evo_testkit::{EvoTestKit, EvoTestApiBuilder, EvoTestKitApi, asset_fees, create_asset};

use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::AssetBundle;
use dmbc::currency::error::Error;
use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;

#[test]
fn transfer() {
    let tax = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &public_key);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_value_to_wallet(&public_key, asset.clone(), info)
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer()
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .seed(42)
        .build();

    let tx_hash = tx_transfer.hash();

    let (status, response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_transfer);
    assert_eq!(tx_status, Ok(Ok(())));

    let recipient_wallet = testkit.fetch_wallet(&recipient_key);
    assert_eq!(recipient_wallet.balance(), 0);
    assert_eq!(recipient_wallet.assets(), vec![asset]);

    let sender_wallet = testkit.fetch_wallet(&public_key);
    let expected_balance = balance - transaction_fee;
    assert_eq!(sender_wallet.balance(), expected_balance);
    assert!(sender_wallet.assets().is_empty());
}

#[test]
fn tranfer_asset_not_found() {
    let tax = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, _) = create_asset(meta_data, units, asset_fees(tax, 0), &public_key);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer()
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .seed(42)
        .build();

    let tx_hash = tx_transfer.hash();

    let (status, response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_transfer);
    assert_eq!(tx_status, Ok(Err(Error::AssetNotFound)));

    let recipient_wallet = testkit.fetch_wallet(&recipient_key);
    assert_eq!(recipient_wallet.balance(), 0);
    assert!(recipient_wallet.assets().is_empty());

    let sender_wallet = testkit.fetch_wallet(&public_key);
    let expected_balance = balance - transaction_fee;
    assert_eq!(sender_wallet.balance(), expected_balance);
    assert!(sender_wallet.assets().is_empty());
}

#[test]
fn tranfer_insufficient_funds() {
    let tax = 10;
    let transaction_fee = 1000_000_000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, _) = create_asset(meta_data, units, asset_fees(tax, 0), &public_key);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer()
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .seed(42)
        .build();

    let tx_hash = tx_transfer.hash();

    let (status, response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_transfer);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientFunds)));

    let recipient_wallet = testkit.fetch_wallet(&recipient_key);
    assert_eq!(recipient_wallet.balance(), 0);
    assert!(recipient_wallet.assets().is_empty());

    let sender_wallet = testkit.fetch_wallet(&public_key);
    let expected_balance = balance;
    assert_eq!(sender_wallet.balance(), expected_balance);
    assert!(sender_wallet.assets().is_empty());
}

#[test]
fn tranfer_insufficient_assets() {
    let tax = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &public_key);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_value_to_wallet(&public_key, asset.clone(), info)
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer()
        .add_asset_value(AssetBundle::new(asset.id(), asset.amount()*2))
        .recipient(recipient_key)
        .seed(42)
        .build();

    let tx_hash = tx_transfer.hash();

    let (status, response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_transfer);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientAssets)));

    let recipient_wallet = testkit.fetch_wallet(&recipient_key);
    assert_eq!(recipient_wallet.balance(), 0);
    assert!(recipient_wallet.assets().is_empty());

    let sender_wallet = testkit.fetch_wallet(&public_key);
    let expected_balance = balance - transaction_fee;
    assert_eq!(sender_wallet.balance(), expected_balance);
}