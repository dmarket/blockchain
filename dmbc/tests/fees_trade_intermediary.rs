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
use evo_testkit::{EvoTestApiBuilder, EvoTestKitApi, asset_fees, create_asset};

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::error::Error;
use dmbc::currency::transactions::components::FeeStrategy;
use dmbc::currency::assets::TradeAsset;

#[test]
fn fees_for_trade_intermediary_recipient() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_value_to_wallet(asset.clone(), info, &seller_public_key)
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    let (status, response) = api.post_fee(&tx_trade);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax;
    expected.insert(buyer_public_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_trade_intermediary_sender() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_value_to_wallet(asset.clone(), info, &seller_public_key)
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::Sender)
        .seed(12)
        .build();

    let (status, response) = api.post_fee(&tx_trade);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax;
    expected.insert(seller_public_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_trade_intermediary_recipient_and_sender() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_value_to_wallet(asset.clone(), info, &seller_public_key)
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(12)
        .build();

    let (status, response) = api.post_fee(&tx_trade);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee / 2 + tax / 2;
    expected.insert(buyer_public_key, expected_fee);
    expected.insert(seller_public_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_trade_intermediary_intermedniary() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_value_to_wallet(asset.clone(), info, &seller_public_key)
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::Intermediary)
        .seed(12)
        .build();

    let (status, response) = api.post_fee(&tx_trade);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax;
    expected.insert(intermediary_public_key, expected_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_trade_intermediary_recipient_and_sender_creator() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &seller_public_key);

    let testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_asset_value_to_wallet(asset.clone(), info, &seller_public_key)
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(12)
        .build();

    let (status, response) = api.post_fee(&tx_trade);
    let mut expected = HashMap::new();
    let expected_seller_fee = transaction_fee / 2;
    let expected_buyer_fee = transaction_fee / 2 + tax / 2;
    expected.insert(seller_public_key, expected_seller_fee);
    expected.insert(buyer_public_key, expected_buyer_fee);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_trade_intermediary_asset_not_found() {
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset, _) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);

    let testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    let (status, response) = api.post_fee(&tx_trade);

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Ok(Err(Error::AssetNotFound)));
}