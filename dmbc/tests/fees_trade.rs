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
use dmbc::currency::assets::TradeAsset;

#[test]
fn fees_for_trade_recipient() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);
    testkit.add_assets(&creator_pub_key, vec![asset.clone()], vec![info]);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
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
fn fees_for_trade_sender() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);
    testkit.add_assets(&creator_pub_key, vec![asset.clone()], vec![info]);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
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
fn fees_for_trade_recipient_and_sender() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);
    testkit.add_assets(&creator_pub_key, vec![asset.clone()], vec![info]);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
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
fn fees_for_trade_recipient_and_sender_creator() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &seller_public_key);
    testkit.add_assets(&seller_public_key, vec![asset.clone()], vec![info]);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(12)
        .build();

    let (status, response) = api.post_fee(&tx_trade);
    let mut expected = HashMap::new();
    let expected_fee_buyer = transaction_fee / 2 + tax / 2;
    let expected_fee_seller = transaction_fee / 2;
    expected.insert(seller_public_key, expected_fee_seller);
    expected.insert(buyer_public_key, expected_fee_buyer);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Ok(FeesResponseBody { fees: expected })));
}

#[test]
fn fees_for_trade_invalid_transaction() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();;

    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);
    testkit.add_assets(&creator_pub_key, vec![asset.clone()], vec![info]);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Intermediary)
        .seed(12)
        .build();

    let (status, response) = api.post_fee(&tx_trade);

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Ok(Err(Error::InvalidTransaction)));
}

#[test]
fn fees_for_trade_asset_not_found() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);

    testkit.set_configuration(Configuration::new(config_fees));

    let (creator_pub_key, _) = crypto::gen_keypair();
    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();


    let (asset, _) = create_asset(meta_data, units, asset_fees(tax, 0), &creator_pub_key);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    let (status, response) = api.post_fee(&tx_trade);

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Ok(Err(Error::AssetNotFound)));
}
