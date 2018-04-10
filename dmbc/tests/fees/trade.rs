use std::collections::HashMap;

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::transactions::components::FeeStrategy;
use dmbc::currency::assets::{AssetBundle, TradeAsset};
use dmbc::currency::error::Error;

use fees::test_api::*;

#[test]
fn fees_for_trade_recipient() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";

    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0));
    
    let (creator_pub_key, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let asset = AssetBundle::from_data(meta_data, units, &creator_pub_key);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    let response = post_fee(&api, &tx_trade);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax;
    expected.insert(buyer_public_key, expected_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_trade_sender() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";

    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0));
    
    let (creator_pub_key, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let asset = AssetBundle::from_data(meta_data, units, &creator_pub_key);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Sender)
        .seed(12)
        .build();

    let response = post_fee(&api, &tx_trade);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax;
    expected.insert(seller_public_key, expected_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_trade_recipient_and_sender() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";

    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0));
    
    let (creator_pub_key, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let asset = AssetBundle::from_data(meta_data, units, &creator_pub_key);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset, price_per_unit))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(12)
        .build();

    let response = post_fee(&api, &tx_trade);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee/2 + tax/2;
    expected.insert(seller_public_key, expected_fee);
    expected.insert(buyer_public_key, expected_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_trade_recipient_and_sender_creator() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";

    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0));
    
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(12)
        .build();

    let response = post_fee(&api, &tx_trade);
    let mut expected = HashMap::new();
    let expected_fee_buyer = transaction_fee/2 + tax/2;
    let expected_fee_seller = transaction_fee/2;
    expected.insert(seller_public_key, expected_fee_seller);
    expected.insert(buyer_public_key, expected_fee_buyer);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_trade_invalid_transaction() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";

    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0));

    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Intermediary)
        .seed(12)
        .build();

    let response = post_fee(&api, &tx_trade);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax;
    expected.insert(buyer_public_key, expected_fee);

    assert_eq!(Ok(Err(Error::InvalidTransaction)), response);
}

#[test]
fn fees_for_trade_asset_not_found() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let price_per_unit = 1000;
    let meta_data = "asset";

    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0));

    let (seller_public_key, seller_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Intermediary)
        .seed(12)
        .build();

    let response = post_fee(&api, &tx_trade);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax;
    expected.insert(buyer_public_key, expected_fee);

    assert_eq!(Ok(Err(Error::AssetNotFound)), response);
}