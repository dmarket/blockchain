extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate mount;

pub mod dmbc_testkit;

use hyper::status::StatusCode;
use exonum::messages::Message;
use exonum::crypto;
use dmbc_testkit::{DmbcTestApiBuilder, DmbcTestKitApi};

use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::AssetBundle;
use dmbc::currency::error::Error;
use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;

#[test]
fn transfer() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, Default::default()))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
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

    let recipient_wallet = api.get_wallet(&recipient_key);
    let recipient_assets = api.get_wallet_assets(&recipient_key).iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    assert_eq!(recipient_wallet.balance, 0);
    assert_eq!(recipient_assets, vec![asset]);

    let sender_wallet = api.get_wallet(&public_key);
    let expected_balance = balance - transaction_fee;
    let sender_assets = api.get_wallet_assets(&public_key).iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    assert_eq!(sender_wallet.balance, expected_balance);
    assert!(sender_assets.is_empty());
}

#[test]
fn transfer_asset_not_found() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, _) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, Default::default()))
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

    let recipient_wallet = api.get_wallet(&recipient_key);
    assert_eq!(recipient_wallet.balance, 0);
    assert!(recipient_wallet.assets_count == 0);

    let sender_wallet = api.get_wallet(&public_key);
    let expected_balance = balance - transaction_fee;
    let sender_assets = api.get_wallet_assets(&public_key).iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    assert_eq!(sender_wallet.balance, expected_balance);
    assert!(sender_assets.is_empty());
}

#[test]
fn transfer_insufficient_funds() {
    let fixed = 10;
    let transaction_fee = 1000_000_000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, _) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, Default::default()))
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

    let recipient_wallet = api.get_wallet(&recipient_key);
    assert_eq!(recipient_wallet.balance, 0);
    assert!(recipient_wallet.assets_count == 0);

    let sender_wallet = api.get_wallet(&public_key);
    let expected_balance = balance;
    assert_eq!(sender_wallet.balance, expected_balance);
    assert!(sender_wallet.assets_count == 0);
}

#[test]
fn transfer_insufficient_assets() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, Default::default()))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
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

    let recipient_wallet = api.get_wallet(&recipient_key);
    assert_eq!(recipient_wallet.balance, 0);
    assert!(recipient_wallet.assets_count == 0);

    let sender_wallet = api.get_wallet(&public_key);
    let expected_balance = balance - transaction_fee;
    assert_eq!(sender_wallet.balance, expected_balance);
}
