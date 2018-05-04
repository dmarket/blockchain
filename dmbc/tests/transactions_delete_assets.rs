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
use dmbc::currency::assets::{AssetBundle, AssetInfo};
use dmbc::currency::error::Error;
use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;

#[test]
fn delete_assets_one_from_bundle() {
    let meta_data = "asset";
    let units = 5;
    let units_to_remove = 1;
    let transaction_fee = 100;
    let balance = 100_000;
    let tax = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &public_key);
    
    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, units_to_remove)
        .seed(5)
        .build();

    let tx_hash = tx_delete_assets.hash();

    let (status, response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_delete_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    let wallet = api.get_wallet(&public_key);
    let wallet_assets = api.get_wallet_assets(&public_key);
    let assets: Vec<AssetBundle> = wallet_assets.iter().map(|a| a.into()).collect();
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance, expected_balance);
    assert_eq!(assets, vec![AssetBundle::new(asset.clone().id(), units-units_to_remove)]);

    let assets_infos: Vec<AssetInfo> = wallet_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect();
    assert_eq!(assets_infos[0], info.decrease(units_to_remove).unwrap());
}

#[test]
fn delete_assets_all_from_bundle() {
    let meta_data = "asset";
    let units = 5;
    let transaction_fee = 100;
    let balance = 100_000;
    let tax = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &public_key);
    
    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, units)
        .seed(5)
        .build();

    let tx_hash = tx_delete_assets.hash();

    let (status, response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_delete_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    let wallet = api.get_wallet(&public_key);
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance, expected_balance);
    assert!(wallet.assets_count == 0);
}

#[test]
fn delete_assets_that_doent_exist() {
    let meta_data = "asset";
    let units = 5;
    let transaction_fee = 100;
    let balance = 100_000;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, units)
        .seed(1)
        .build();

    let tx_hash = tx_delete_assets.hash();

    let (status, response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_delete_assets);
    assert_eq!(tx_status, Ok(Err(Error::AssetNotFound)));

    let wallet = api.get_wallet(&public_key);
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance, expected_balance);
}

#[test]
fn delete_assets_that_doent_exist2() {
    let meta_data = "asset";
    let meta_data2 = "another_asset";
    let units = 5;
    let transaction_fee = 100;
    let balance = 100_000;
    let tax = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();
    let (another_asset, another_info) = dmbc_testkit::create_asset(meta_data2, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (another_asset.clone(), another_info.clone()))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, units)
        .seed(1)
        .build();

    let tx_hash = tx_delete_assets.hash();

    let (status, response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_delete_assets);
    assert_eq!(tx_status, Ok(Err(Error::AssetNotFound)));

    let wallet = api.get_wallet(&public_key);
    let wallet_assets = api.get_wallet_assets(&public_key);
    let assets = wallet_assets.iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance, expected_balance);
    assert_eq!(assets, vec![another_asset.clone()]);

    let assets_infos = wallet_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect::<Vec<AssetInfo>>();
    assert_eq!(assets_infos[0], another_info);
}

#[test]
fn delete_assets_amount_more_than_wallet_have() {
    let meta_data = "asset";
    let units = 5;
    let units_to_delete = units + 5;
    let transaction_fee = 100;
    let balance = 100_000;
    let tax = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, units_to_delete)
        .seed(1)
        .build();

    let tx_hash = tx_delete_assets.hash();

    let (status, response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_delete_assets);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientAssets)));

    let wallet = api.get_wallet(&public_key);
    let wallet_assets = api.get_wallet_assets(&public_key);
    let assets = wallet_assets.iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance, expected_balance);
    assert_eq!(assets, vec![asset.clone()]);

    let assets_infos = wallet_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect::<Vec<AssetInfo>>();
    assert_eq!(assets_infos[0], info);
}

#[test]
fn delete_assets_insufficient_funds() {
    let meta_data = "asset";
    let units = 5;
    let transaction_fee = 100;
    let balance = 5;
    let tax = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, units)
        .seed(1)
        .build();

    let tx_hash = tx_delete_assets.hash();

    let (status, response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_delete_assets);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientFunds)));

    let wallet = api.get_wallet(&public_key);
    let wallet_assets = api.get_wallet_assets(&public_key);
    let assets = wallet_assets.iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    assert_eq!(wallet.balance, balance);
    assert_eq!(assets, vec![asset.clone()]);

    let assets_infos = wallet_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect::<Vec<AssetInfo>>();
    assert_eq!(assets_infos[0], info);
}

#[test]
fn delete_assets_with_different_creator() {
    let meta_data = "asset";
    let units = 5;
    let transaction_fee = 100;
    let balance = 100_000;
    let tax = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);

    let (creator_key, _) = crypto::gen_keypair();
    let (public_key, secret_key) = crypto::gen_keypair();
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &creator_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset_value(asset.clone())
        .seed(1)
        .build();

    let tx_hash = tx_delete_assets.hash();

    let (status, response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_delete_assets);
    assert_eq!(tx_status, Ok(Err(Error::InvalidTransaction)));

    let wallet = api.get_wallet(&public_key);
    let wallet_assets = api.get_wallet_assets(&public_key);
    let assets = wallet_assets.iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance, expected_balance);
    assert_eq!(assets, vec![asset.clone()]);

    let assets_infos = wallet_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect::<Vec<AssetInfo>>();
    assert_eq!(assets_infos[0], info);
}

#[test]
fn delete_assets_two_assets_where_one_asset_doesnt_have_enough_items() {
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let units = 5;
    let transaction_fee = 100;
    let balance = 100_000;
    let tax = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();
    let (asset1, info1) = dmbc_testkit::create_asset(meta_data1, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &public_key);
    let (asset2, info2) = dmbc_testkit::create_asset(meta_data2, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset1.clone(), info1.clone()))
        .add_asset_to_wallet(&public_key, (asset2.clone(), info2.clone()))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset_value(AssetBundle::new(asset1.id(), 3))
        .add_asset_value(AssetBundle::new(asset2.id(), 7))
        .seed(1)
        .build();

    let tx_hash = tx_delete_assets.hash();

    let (status, response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_delete_assets);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientAssets)));

    let wallet = api.get_wallet(&public_key);
    let wallet_assets = api.get_wallet_assets(&public_key);
    let assets = wallet_assets.iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance, expected_balance);
    assert_eq!(assets, vec![asset1.clone(), asset2.clone()]);

    let assets_infos = wallet_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect::<Vec<AssetInfo>>();
    assert_eq!(assets_infos, vec![info1, info2]);
}

#[test]
fn delete_assets_two_assets_where_one_have_another_creator() {
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let units = 5;
    let transaction_fee = 100;
    let balance = 100_000;
    let tax = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);

    let (creator_key, _) = crypto::gen_keypair();
    let (public_key, secret_key) = crypto::gen_keypair();
    let (asset1, info1) = dmbc_testkit::create_asset(meta_data1, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &creator_key);
    let (asset2, info2) = dmbc_testkit::create_asset(meta_data2, units, dmbc_testkit::asset_fees(tax, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset1.clone(), info1.clone()))
        .add_asset_to_wallet(&public_key, (asset2.clone(), info2.clone()))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset_value(asset1.clone())
        .add_asset_value(asset2.clone())
        .seed(1)
        .build();

    let tx_hash = tx_delete_assets.hash();

    let (status, response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_delete_assets);
    assert_eq!(tx_status, Ok(Err(Error::InvalidTransaction)));

    let wallet = api.get_wallet(&public_key);
    let wallet_assets = api.get_wallet_assets(&public_key);
    let assets = wallet_assets.iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance, expected_balance);
    assert_eq!(assets, vec![asset1.clone(), asset2.clone()]);

    let assets_infos = wallet_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect::<Vec<AssetInfo>>();
    assert_eq!(assets_infos, vec![info1, info2]);
}
