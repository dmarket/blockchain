use std::collections::HashMap;

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::transactions::components::FeeStrategy;
use dmbc::currency::assets::AssetBundle;
use dmbc::currency::error::Error;

use fees::test_api::*;

#[test]
fn fees_for_exchange_recipient() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";

    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, transaction_fee, 0, 0));

    let (creator_pub_key, _) = WalletMiner::new()
        .add_asset(meta_data0, units, asset_fee(tax, 0))
        .add_asset(meta_data1, units, asset_fee(tax, 0))
        .add_asset(meta_data2, units, asset_fee(tax, 0))
        .add_asset(meta_data3, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (sender_public_key, sender_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (recipient_public_key, recipient_secret_key) = WalletMiner::new().mine(&mut testkit);

    let asset0 = AssetBundle::from_data(meta_data0, units, &creator_pub_key);
    let asset1 = AssetBundle::from_data(meta_data1, units, &creator_pub_key);
    let asset2 = AssetBundle::from_data(meta_data2, units, &creator_pub_key);
    let asset3 = AssetBundle::from_data(meta_data3, units, &creator_pub_key);

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

    let response = post_fee(&api, &tx_exchange_assets);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax * units * 4;
    expected.insert(recipient_public_key, expected_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_exchange_sender() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";

    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, transaction_fee, 0, 0));

    let (creator_pub_key, _) = WalletMiner::new()
        .add_asset(meta_data0, units, asset_fee(tax, 0))
        .add_asset(meta_data1, units, asset_fee(tax, 0))
        .add_asset(meta_data2, units, asset_fee(tax, 0))
        .add_asset(meta_data3, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (sender_public_key, sender_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (recipient_public_key, recipient_secret_key) = WalletMiner::new().mine(&mut testkit);

    let asset0 = AssetBundle::from_data(meta_data0, units, &creator_pub_key);
    let asset1 = AssetBundle::from_data(meta_data1, units, &creator_pub_key);
    let asset2 = AssetBundle::from_data(meta_data2, units, &creator_pub_key);
    let asset3 = AssetBundle::from_data(meta_data3, units, &creator_pub_key);

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

    let response = post_fee(&api, &tx_exchange_assets);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + tax * units * 4;
    expected.insert(sender_public_key, expected_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_exchange_recipient_and_sender() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";

    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, transaction_fee, 0, 0));

    let (creator_pub_key, _) = WalletMiner::new()
        .add_asset(meta_data0, units, asset_fee(tax, 0))
        .add_asset(meta_data1, units, asset_fee(tax, 0))
        .add_asset(meta_data2, units, asset_fee(tax, 0))
        .add_asset(meta_data3, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (sender_public_key, sender_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (recipient_public_key, recipient_secret_key) = WalletMiner::new().mine(&mut testkit);

    let asset0 = AssetBundle::from_data(meta_data0, units, &creator_pub_key);
    let asset1 = AssetBundle::from_data(meta_data1, units, &creator_pub_key);
    let asset2 = AssetBundle::from_data(meta_data2, units, &creator_pub_key);
    let asset3 = AssetBundle::from_data(meta_data3, units, &creator_pub_key);

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

    let response = post_fee(&api, &tx_exchange_assets);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee / 2 + tax * units * 2;
    expected.insert(sender_public_key, expected_fee);
    expected.insert(recipient_public_key, expected_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_exchange_recipient_and_sender_creator() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";

    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, transaction_fee, 0, 0));

    let (sender_public_key, sender_secret_key) = WalletMiner::new()
        .add_asset(meta_data0, units, asset_fee(tax, 0))
        .add_asset(meta_data1, units, asset_fee(tax, 0))
        .add_asset(meta_data2, units, asset_fee(tax, 0))
        .add_asset(meta_data3, units, asset_fee(tax, 0))
        .mine(&mut testkit);
    let (recipient_public_key, recipient_secret_key) = WalletMiner::new().mine(&mut testkit);

    let asset0 = AssetBundle::from_data(meta_data0, units, &sender_public_key);
    let asset1 = AssetBundle::from_data(meta_data1, units, &sender_public_key);
    let asset2 = AssetBundle::from_data(meta_data2, units, &sender_public_key);
    let asset3 = AssetBundle::from_data(meta_data3, units, &sender_public_key);

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

    let response = post_fee(&api, &tx_exchange_assets);
    let mut expected = HashMap::new();
    let expected_sender_fee = transaction_fee / 2;
    let expected_recipient_fee = transaction_fee / 2 + tax * units * 2;
    expected.insert(sender_public_key, expected_sender_fee);
    expected.insert(recipient_public_key, expected_recipient_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_exchange_invalid_transaction() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";

    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, transaction_fee, 0, 0));

    let (creator_pub_key, _) = WalletMiner::new()
        .add_asset(meta_data0, units, asset_fee(tax, 0))
        .add_asset(meta_data1, units, asset_fee(tax, 0))
        .add_asset(meta_data2, units, asset_fee(tax, 0))
        .add_asset(meta_data3, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (sender_public_key, sender_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (recipient_public_key, recipient_secret_key) = WalletMiner::new().mine(&mut testkit);

    let asset0 = AssetBundle::from_data(meta_data0, units, &creator_pub_key);
    let asset1 = AssetBundle::from_data(meta_data1, units, &creator_pub_key);
    let asset2 = AssetBundle::from_data(meta_data2, units, &creator_pub_key);
    let asset3 = AssetBundle::from_data(meta_data3, units, &creator_pub_key);

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

    let response = post_fee(&api, &tx_exchange_assets);

    assert_eq!(Ok(Err(Error::InvalidTransaction)), response);
}

#[test]
fn fees_for_exchange_asset_not_found() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";

    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, transaction_fee, 0, 0));

    let (creator_pub_key, _) = WalletMiner::new().mine(&mut testkit);
    let (sender_public_key, sender_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (recipient_public_key, recipient_secret_key) = WalletMiner::new().mine(&mut testkit);

    let asset0 = AssetBundle::from_data(meta_data0, units, &creator_pub_key);
    let asset1 = AssetBundle::from_data(meta_data1, units, &creator_pub_key);
    let asset2 = AssetBundle::from_data(meta_data2, units, &creator_pub_key);
    let asset3 = AssetBundle::from_data(meta_data3, units, &creator_pub_key);

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

    let response = post_fee(&api, &tx_exchange_assets);

    assert_eq!(Ok(Err(Error::AssetNotFound)), response);
}