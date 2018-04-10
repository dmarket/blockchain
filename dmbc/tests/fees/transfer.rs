use std::collections::HashMap;

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::assets::AssetBundle;
use dmbc::currency::error::Error;

use fees::test_api::*;

#[test]
fn fees_for_transfer() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let amount = 2;
    let tax = 10;
    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee));

    let meta_data = "asset";
    let (creator_key, _) = WalletMiner::new()
        .add_asset(meta_data, amount, asset_fee(tax, 0))
        .mine(&mut testkit);
    let (recipient_key, _) = WalletMiner::new().mine_empty(&mut testkit);
    let (sender_pub_key, sender_sec_key) = WalletMiner::new().mine_empty(&mut testkit);

    let asset = AssetBundle::from_data(meta_data, amount, &creator_key);

    let tx_transfer = transaction::Builder::new()
        .keypair(sender_pub_key, sender_sec_key)
        .tx_transfer()
        .add_asset_value(asset)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let response = post_fee(&api, &tx_transfer);
    let mut expected = HashMap::new();
    let expected_fee = transaction_fee + amount * tax;
    expected.insert(sender_pub_key, expected_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_transfer_sender_is_creator() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let amount = 2;
    let tax = 10;
    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee));

    let meta_data = "asset";
    let (sender_key, secret_key) = WalletMiner::new()
        .add_asset(meta_data, amount, asset_fee(tax, 0))
        .mine(&mut testkit);
    let (recipient_key, _) = WalletMiner::new().mine_empty(&mut testkit);
        
    let tx_transfer = transaction::Builder::new()
        .keypair(sender_key, secret_key)
        .tx_transfer()
        .add_asset(meta_data, amount)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let response = post_fee(&api, &tx_transfer);
    let mut expected = HashMap::new();
    expected.insert(sender_key, transaction_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_transfer_asset_not_found() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let amount = 2;
    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee));

    let meta_data = "asset";
    let (sender_key, secret_key) = WalletMiner::new().mine_empty(&mut testkit);
    let (recipient_key, _) = WalletMiner::new().mine_empty(&mut testkit);
        
    let tx_transfer = transaction::Builder::new()
        .keypair(sender_key, secret_key)
        .tx_transfer()
        .add_asset(meta_data, amount)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let response = post_fee(&api, &tx_transfer);
    assert_eq!(Ok(Err(Error::AssetNotFound)), response);
}