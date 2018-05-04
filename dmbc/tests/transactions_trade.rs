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
use dmbc::currency::assets::{TradeAsset, AssetBundle};
use dmbc::currency::error::Error;
use dmbc::currency::api::error::ApiError;
use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;
use dmbc::currency::transactions::components::FeeStrategy;

#[test]
fn trade_fee_from_recipient() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &seller_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let genesis_balance = api.get_wallet(&dmbc_testkit::default_genesis_key()).balance;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .build();

    let tx_hash = tx_trade.hash();

    let (status, response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_trade);
    assert_eq!(tx_status, Ok(Ok(())));

    let seller_wallet = api.get_wallet(&seller_public_key);
    let buyer_wallet = api.get_wallet(&buyer_public_key);
    let genesis_wallet = api.get_wallet(&dmbc_testkit::default_genesis_key());

    let assets_price = units * price;
    let trade_fee = units * fixed;
    let expected_sellers_balance = balance + assets_price + trade_fee;
    let expected_buyer_balace = balance - assets_price - transaction_fee - trade_fee;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(seller_wallet.balance, expected_sellers_balance);
    assert_eq!(buyer_wallet.balance, expected_buyer_balace);
    assert_eq!(genesis_wallet.balance, expected_genesis_balance);

    let buyer_assets = api.get_wallet_assets(&buyer_public_key).iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    assert!(seller_wallet.assets_count == 0);
    assert_eq!(buyer_assets,vec![asset]);
}

#[test]
fn trade_fee_from_sender() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &seller_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let genesis_balance = api.get_wallet(&dmbc_testkit::default_genesis_key()).balance;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Sender)
        .seed(1)
        .build();

    let tx_hash = tx_trade.hash();

    let (status, response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_trade);
    assert_eq!(tx_status, Ok(Ok(())));

    let seller_wallet = api.get_wallet(&seller_public_key);
    let buyer_wallet = api.get_wallet(&buyer_public_key);
    let genesis_wallet = api.get_wallet(&dmbc_testkit::default_genesis_key());

    let assets_price = units * price;
    let expected_sellers_balance = balance + assets_price - transaction_fee;
    let expected_buyer_balace = balance - assets_price;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(seller_wallet.balance, expected_sellers_balance);
    assert_eq!(buyer_wallet.balance, expected_buyer_balace);
    assert_eq!(genesis_wallet.balance, expected_genesis_balance);

    let buyer_assets = api.get_wallet_assets(&buyer_public_key).iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    assert!(seller_wallet.assets_count == 0);
    assert_eq!(buyer_assets,vec![asset]);
}

#[test]
fn trade_fee_from_recipient_and_sender() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &seller_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let genesis_balance = api.get_wallet(&dmbc_testkit::default_genesis_key()).balance;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(1)
        .build();

    let tx_hash = tx_trade.hash();

    let (status, response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_trade);
    assert_eq!(tx_status, Ok(Ok(())));

    let seller_wallet = api.get_wallet(&seller_public_key);
    let buyer_wallet = api.get_wallet(&buyer_public_key);
    let genesis_wallet = api.get_wallet(&dmbc_testkit::default_genesis_key());

    let expected_sellers_balance = balance + units * price - transaction_fee/2 + fixed * units / 2;
    let expected_buyer_balace = balance - units * price - transaction_fee/2 - fixed * units / 2;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(seller_wallet.balance, expected_sellers_balance);
    assert_eq!(buyer_wallet.balance, expected_buyer_balace);
    assert_eq!(genesis_wallet.balance, expected_genesis_balance);

    let buyer_assets = api.get_wallet_assets(&buyer_public_key).iter().map(|a| a.into()).collect::<Vec<AssetBundle>>();
    assert!(seller_wallet.assets_count == 0);
    assert_eq!(buyer_assets,vec![asset]);
}

#[test]
fn trade_fee_bad_request() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &seller_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let genesis_balance = api.get_wallet(&dmbc_testkit::default_genesis_key()).balance;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Intermediary)
        .seed(1)
        .build();

    let (status, response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Ok(Err(Error::UnableToVerifyTransaction)));

    let (_, tx_status) = api.get_tx_status(&tx_trade);
    assert_eq!(tx_status, Err(ApiError::TransactionNotFound));

    let seller_wallet = api.get_wallet(&seller_public_key);
    let buyer_wallet = api.get_wallet(&buyer_public_key);
    let genesis_wallet = api.get_wallet(&dmbc_testkit::default_genesis_key());

    let expected_sellers_balance = balance;
    let expected_buyer_balace = balance;
    let expected_genesis_balance = genesis_balance;

    assert_eq!(seller_wallet.balance, expected_sellers_balance);
    assert_eq!(buyer_wallet.balance, expected_buyer_balace);
    assert_eq!(genesis_wallet.balance, expected_genesis_balance);
    assert!(seller_wallet.assets_count > 0);
    assert!(buyer_wallet.assets_count == 0);
}

#[test]
fn trade_asset_not_found() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, _) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &seller_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    let genesis_balance = api.get_wallet(&dmbc_testkit::default_genesis_key()).balance;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(1)
        .build();

    let tx_hash = tx_trade.hash();

    let (status, response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_trade);
    assert_eq!(tx_status, Ok(Err(Error::AssetNotFound)));

    let seller_wallet = api.get_wallet(&seller_public_key);
    let buyer_wallet = api.get_wallet(&buyer_public_key);
    let genesis_wallet = api.get_wallet(&dmbc_testkit::default_genesis_key());

    let expected_sellers_balance = balance - transaction_fee / 2;
    let expected_buyer_balace = balance - transaction_fee / 2;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(seller_wallet.balance, expected_sellers_balance);
    assert_eq!(buyer_wallet.balance, expected_buyer_balace);
    assert_eq!(genesis_wallet.balance, expected_genesis_balance);
}

#[test]
fn trade_insufficient_assets() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &seller_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let genesis_balance = api.get_wallet(&dmbc_testkit::default_genesis_key()).balance;
    let insufficient_asset = AssetBundle::new(asset.id(), units * 2);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(insufficient_asset, price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Sender)
        .seed(1)
        .build();

    let tx_hash = tx_trade.hash();

    let (status, response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_trade);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientAssets)));

    let seller_wallet = api.get_wallet(&seller_public_key);
    let buyer_wallet = api.get_wallet(&buyer_public_key);
    let genesis_wallet = api.get_wallet(&dmbc_testkit::default_genesis_key());

    let expected_sellers_balance = balance - transaction_fee;
    let expected_buyer_balace = balance;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(seller_wallet.balance, expected_sellers_balance);
    assert_eq!(buyer_wallet.balance, expected_buyer_balace);
    assert_eq!(genesis_wallet.balance, expected_genesis_balance);
}

#[test]
fn trade_insufficient_funds() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &seller_public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let genesis_balance = api.get_wallet(&dmbc_testkit::default_genesis_key()).balance;
    let insufficient_asset = AssetBundle::new(asset.id(), units * 2);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(insufficient_asset, price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Sender)
        .seed(1)
        .build();

    let tx_hash = tx_trade.hash();

    let (status, response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_trade);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientFunds)));

    let seller_wallet = api.get_wallet(&seller_public_key);
    let buyer_wallet = api.get_wallet(&buyer_public_key);
    let genesis_wallet = api.get_wallet(&dmbc_testkit::default_genesis_key());

    let expected_sellers_balance = balance;
    let expected_buyer_balace = balance;
    let expected_genesis_balance = genesis_balance;

    assert_eq!(seller_wallet.balance, expected_sellers_balance);
    assert_eq!(buyer_wallet.balance, expected_buyer_balace);
    assert_eq!(genesis_wallet.balance, expected_genesis_balance);
}