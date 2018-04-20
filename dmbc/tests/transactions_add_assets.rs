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
use dmbc_testkit::{DmbcTestKit, DmbcTestApiBuilder, DmbcTestKitApi, asset_fees, create_asset, create_asset2};

use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::{MetaAsset, AssetBundle, AssetInfo};
use dmbc::currency::error::Error;
use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;

#[test]
fn add_assets_mine_new_asset_to_receiver_empty_wallet() {
    let tax = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&receiver_key, meta_data, units, asset_fees(tax, 0));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(85)
        .build();

    let tx_hash = tx_add_assets.hash();

    let (status, response) = api.post_tx(&tx_add_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_add_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    // check creator wallet
    let creator = testkit.fetch_wallet(&creator_public_key);
    let expected_balance = balance - transaction_fee - per_asset_fee * units;
    assert_eq!(creator.balance(), expected_balance);
    assert!(creator.assets().is_empty());

    // create asset's equivalents created by tx execution
    let (asset, info) = create_asset2(
        meta_data, 
        units, 
        asset_fees(tax, 0), 
        &creator_public_key,
        &tx_hash
    );

    // check receiver wallet
    let receiver = testkit.fetch_wallet(&receiver_key);
    assert_eq!(receiver.assets(), vec![asset.clone()]);

    // compare asset info from blockchain
    let info_from_blockchain = testkit.fetch_asset_info(&asset.id());
    assert_eq!(info_from_blockchain, Some(info));
}

#[test]
fn add_assets_mine_existing_asset_to_receivers_non_empty_wallet() {
    let tax = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let meta_asset = MetaAsset::new(&receiver_key, meta_data, units, asset_fees(tax, 0));
    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&receiver_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();
    
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(85)
        .build();

    let tx_hash = tx_add_assets.hash();

    let (status, response) = api.post_tx(&tx_add_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_add_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    // check creator wallet
    let creator = testkit.fetch_wallet(&creator_public_key);
    let expected_balance = balance - transaction_fee - per_asset_fee * units;
    assert_eq!(creator.balance(), expected_balance);
    assert!(creator.assets().is_empty());

    let updated_asset = AssetBundle::new(asset.id(), asset.amount() * 2);
    let updated_info = AssetInfo::new(info.creator(), info.origin(), info.amount() * 2, info.fees());

    // check receiver wallet
    let receiver = testkit.fetch_wallet(&receiver_key);
    assert_eq!(receiver.assets(), vec![updated_asset.clone()]);

    // compare asset info from blockchain
    let info_from_blockchain = testkit.fetch_asset_info(&updated_asset.id());
    assert_eq!(info_from_blockchain, Some(updated_info));
}

#[test]
fn add_assets_mine_existing_asset_to_creators_empty_wallet() {
    let tax = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&receiver_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();
    
    let meta_asset = MetaAsset::new(&creator_public_key, meta_data, units, asset_fees(tax, 0));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(85)
        .build();

    let tx_hash = tx_add_assets.hash();

    let (status, response) = api.post_tx(&tx_add_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_add_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    // check creator wallet
    let asset = AssetBundle::new(asset.id(), asset.amount());
    let updated_info = AssetInfo::new(info.creator(), info.origin(), info.amount() * 2, info.fees());

    let creator = testkit.fetch_wallet(&creator_public_key);
    let expected_balance = balance - transaction_fee - per_asset_fee * units;
    assert_eq!(creator.balance(), expected_balance);
    assert_eq!(creator.assets(), vec![asset.clone()]);

    // check receiver wallet
    let receiver = testkit.fetch_wallet(&receiver_key);
    assert_eq!(receiver.assets(), vec![asset.clone()]);

    // compare asset info from blockchain
    let info_from_blockchain = testkit.fetch_asset_info(&asset.id());
    assert_eq!(info_from_blockchain, Some(updated_info));
}

#[test]
fn add_assets_mine_existing_asset_to_creator_and_receiver() {
    let tax = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let (asset, info) = create_asset(
        meta_data, 
        units, 
        asset_fees(tax, 0), 
        &creator_public_key, 
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&receiver_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();
    
    let meta_asset_for_creator = MetaAsset::new(&creator_public_key, meta_data, units, asset_fees(tax, 0));
    let meta_asset_for_receiver = MetaAsset::new(&receiver_key, meta_data, units, asset_fees(tax, 0));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset_for_creator)
        .add_asset_value(meta_asset_for_receiver)
        .seed(85)
        .build();

    let tx_hash = tx_add_assets.hash();

    let (status, response) = api.post_tx(&tx_add_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_add_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    // check creator wallet
    let creators_asset = AssetBundle::new(asset.id(), asset.amount());
    let receiver_asset = AssetBundle::new(asset.id(), asset.amount() * 2);
    let updated_info = AssetInfo::new(info.creator(), info.origin(), info.amount() * 3, info.fees());

    let creator = testkit.fetch_wallet(&creator_public_key);
    let expected_balance = balance - transaction_fee - per_asset_fee * units * 2;
    assert_eq!(creator.balance(), expected_balance);
    assert_eq!(creator.assets(), vec![creators_asset]);

    // check receiver wallet
    let receiver = testkit.fetch_wallet(&receiver_key);
    assert_eq!(receiver.assets(), vec![receiver_asset]);

    // compare asset info from blockchain
    let info_from_blockchain = testkit.fetch_asset_info(&asset.id());
    assert_eq!(info_from_blockchain, Some(updated_info));
}

#[test]
fn add_assets_mine_existing_asset_to_receivers_wallet_with_different_asset() {
    let tax = 10;
    let meta_data = "asset";
    let new_meta_data = "new_asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&receiver_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();
    
    let meta_asset_for_receiver = MetaAsset::new(&receiver_key, new_meta_data, units, asset_fees(tax, 0));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset_for_receiver)
        .seed(85)
        .build();

    let tx_hash = tx_add_assets.hash();

    let (status, response) = api.post_tx(&tx_add_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_add_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    // check creator wallet
    let (new_asset, new_info) = create_asset2(
        new_meta_data, 
        units, 
        asset_fees(tax, 0), 
        &creator_public_key, 
        &tx_hash
    );

    let creator = testkit.fetch_wallet(&creator_public_key);
    let expected_balance = balance - transaction_fee - per_asset_fee * units;
    assert_eq!(creator.balance(), expected_balance);
    assert!(creator.assets().is_empty());

    // check receiver wallet
    let receiver = testkit.fetch_wallet(&receiver_key);
    assert_eq!(receiver.assets(), vec![asset, new_asset.clone()]);

    // compare asset info from blockchain
    let info_from_blockchain = testkit.fetch_asset_info(&new_asset.id());
    assert_eq!(info_from_blockchain, Some(new_info));
}

#[test]
fn add_assets_mine_existing_asset_with_different_fees() {
    let tax1 = 10;
    let tax2 = 20;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax1, 0), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset(meta_data, units, asset_fees(tax2, 0))
        .seed(85)
        .build();
    
    let tx_hash = tx_add_assets.hash();

    let (status, response) = api.post_tx(&tx_add_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_add_assets);
    assert_eq!(tx_status, Ok(Err(Error::InvalidAssetInfo)));

    let wallet = testkit.fetch_wallet(&public_key);
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance(), expected_balance);
    assert_eq!(wallet.assets(), vec![asset.clone()]);
    
    let bc_info = testkit.fetch_asset_info(&asset.id());
    assert_eq!(bc_info, Some(info));
}

#[test]
fn add_assets_insufficient_funds() {
    let tax = 10;
    let meta_data = "asset";
    let units = 3;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .create();
    let api = testkit.api();

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset(meta_data, units, asset_fees(tax, 0))
        .seed(85)
        .build();
    
    let tx_hash = tx_add_assets.hash();

    let (status, response) = api.post_tx(&tx_add_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_add_assets);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientFunds)));
}