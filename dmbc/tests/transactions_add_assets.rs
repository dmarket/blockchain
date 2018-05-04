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
use dmbc::currency::assets::{MetaAsset, AssetBundle, AssetInfo};
use dmbc::currency::error::Error;
use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;

#[test]
fn add_assets_mine_new_asset_to_receiver_empty_wallet() {
    let fixed = 10;
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
    let meta_asset = MetaAsset::new(&receiver_key, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
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
    let creator = api.get_wallet(&creator_public_key);
    let expected_balance = balance - transaction_fee - per_asset_fee * units;
    assert_eq!(creator.balance, expected_balance);
    assert!(creator.assets_count == 0);

    // create asset's equivalents created by tx execution
    let (asset, info) = dmbc_testkit::create_asset2(
        meta_data, 
        units, 
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_public_key,
        &tx_hash
    );

    // check receiver wallet
    let receivers_assets = api.get_wallet_assets(&receiver_key);
    let assets: Vec<AssetBundle> = receivers_assets.iter().map(|a| a.into()).collect();
    assert_eq!(assets, vec![asset.clone()]);

    // compare asset info from blockchain
    let asset_info: Vec<AssetInfo> = receivers_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect();
    assert_eq!(asset_info, vec![info]);
}

#[test]
fn add_assets_mine_existing_asset_to_receivers_non_empty_wallet() {
    let fixed = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let meta_asset = MetaAsset::new(&receiver_key, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &creator_public_key);

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
    let creator = api.get_wallet(&creator_public_key);
    let expected_balance = balance - transaction_fee - per_asset_fee * units;
    assert_eq!(creator.balance, expected_balance);
    assert!(creator.assets_count == 0);

    let updated_asset = AssetBundle::new(asset.id(), asset.amount() * 2);
    let updated_info = AssetInfo::new(info.creator(), info.origin(), info.amount() * 2, info.fees());

    // check receiver wallet
    let receiver_assets = api.get_wallet_assets(&receiver_key);
    let assets: Vec<AssetBundle> = receiver_assets.iter().map(|a| a.into()).collect();
    assert_eq!(assets, vec![updated_asset.clone()]);

    // compare asset info from blockchain
    let assets_infos: Vec<AssetInfo> = receiver_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect();
    assert_eq!(assets_infos, vec![updated_info]);
}

#[test]
fn add_assets_mine_existing_asset_to_creators_empty_wallet() {
    let fixed = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &creator_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&receiver_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();
    
    let meta_asset = MetaAsset::new(&creator_public_key, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
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

    let creator = api.get_wallet(&creator_public_key);
    let creators_assets = api.get_wallet_assets(&creator_public_key);
    let assets: Vec<AssetBundle> = creators_assets.iter().map(|a| a.into()).collect();
    let expected_balance = balance - transaction_fee - per_asset_fee * units;
    assert_eq!(creator.balance, expected_balance);
    assert_eq!(assets, vec![asset.clone()]);

    // check receiver wallet
    let receivers_assets = api.get_wallet_assets(&receiver_key);
    let assets: Vec<AssetBundle> = receivers_assets.iter().map(|a| a.into()).collect();
    assert_eq!(assets, vec![asset.clone()]);

    // compare asset info from blockchain
    let assets_infos: Vec<AssetInfo> = receivers_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect();
    assert_eq!(assets_infos, vec![updated_info]);
}

#[test]
fn add_assets_mine_existing_asset_to_creator_and_receiver() {
    let fixed = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data, 
        units, 
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_public_key, 
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&receiver_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();
    
    let meta_asset_for_creator = MetaAsset::new(&creator_public_key, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let meta_asset_for_receiver = MetaAsset::new(&receiver_key, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
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

    let creator = api.get_wallet(&creator_public_key);
    let creators_assets = api.get_wallet_assets(&creator_public_key);
    let assets: Vec<AssetBundle> = creators_assets.iter().map(|a| a.into()).collect();
    let expected_balance = balance - transaction_fee - per_asset_fee * units * 2;
    assert_eq!(creator.balance, expected_balance);
    assert_eq!(assets, vec![creators_asset]);

    // check receiver wallet
    let receivers_assets = api.get_wallet_assets(&receiver_key);
    let assets: Vec<AssetBundle> = receivers_assets.iter().map(|a| a.into()).collect();
    assert_eq!(assets, vec![receiver_asset]);

    // compare asset info from blockchain
    let assets_infos: Vec<AssetInfo> = receivers_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect();
    assert_eq!(assets_infos, vec![updated_info]);
}

#[test]
fn add_assets_mine_existing_asset_to_receivers_wallet_with_different_asset() {
    let fixed = 10;
    let meta_data = "asset";
    let new_meta_data = "new_asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &creator_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&receiver_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();
    
    let meta_asset_for_receiver = MetaAsset::new(&receiver_key, new_meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
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
    let (new_asset, new_info) = dmbc_testkit::create_asset2(
        new_meta_data, 
        units, 
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_public_key, 
        &tx_hash
    );

    // let creator = testkit.fetch_wallet(&creator_public_key);
    let creator = api.get_wallet(&creator_public_key);
    let expected_balance = balance - transaction_fee - per_asset_fee * units;
    assert_eq!(creator.balance, expected_balance);
    assert!(creator.assets_count == 0);

    // check receiver wallet
    let receivers_assets = api.get_wallet_assets(&receiver_key);
    let assets: Vec<AssetBundle> = receivers_assets.iter().map(|a| a.into()).collect();
    assert_eq!(assets, vec![asset, new_asset.clone()]);

    // compare asset info from blockchain
    let assets_infos: Vec<AssetInfo> = receivers_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect();
    assert_eq!(assets_infos[1], new_info);
}

#[test]
fn add_assets_mine_existing_asset_with_different_fees() {
    let fixed1 = 10;
    let fixed2 = 20;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (public_key, secret_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed1, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset(meta_data, units, dmbc_testkit::asset_fees(fixed2, "0.0".parse().unwrap()))
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

    let wallet = api.get_wallet(&public_key);
    let wallet_assets = api.get_wallet_assets(&public_key);
    let assets: Vec<AssetBundle> = wallet_assets.iter().map(|a| a.into()).collect();
    let expected_balance = balance - transaction_fee;
    assert_eq!(wallet.balance, expected_balance);
    assert_eq!(assets, vec![asset.clone()]);

    let assets_infos: Vec<AssetInfo> = wallet_assets.iter().map(|a| a.clone().meta_data.unwrap()).collect();
    assert_eq!(assets_infos, vec![info]);
}

#[test]
fn add_assets_insufficient_funds() {
    let fixed = 10;
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
        .add_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()))
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
