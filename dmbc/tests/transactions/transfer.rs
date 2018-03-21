extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::crypto;
use exonum::messages::Message;

use dmbc::currency::assets::{MetaAsset, AssetId};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::error::Error;

use transactions::*;

#[test]
fn transfer() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, 0, 100));

    let (public_key, secret_key) = mine_wallet(&mut testkit);
    let (recipient_key, _) = crypto::gen_keypair();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 2, fees.clone());
    // Mine 2 items of asset
    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let bc_asset_info = get_asset_info(&api, &asset_id).unwrap().unwrap();
    assert_eq!(meta_asset.to_info(&public_key), bc_asset_info);

    let mining_wallet = get_wallet(&api, &public_key);
    assert_eq!(vec![meta_asset.to_bundle(asset_id.clone())], mining_wallet.assets());

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_transfer()
        .add_asset(meta_data, 2)
        .recipient(recipient_key)
        .seed(42)
        .build();
        
    post_tx(&api, &tx_transfer);
    testkit.create_block();

    let s = get_status(&api, &tx_transfer.hash());
    assert_eq!(Ok(Ok(())), s);

    let recipient_wallet = get_wallet(&api, &recipient_key);
    assert!(!recipient_wallet.assets().is_empty());
    
    let sender_wallet = get_wallet(&api, &public_key);
    assert!(sender_wallet.assets().is_empty());
}

#[test]
fn tranfer_nonexisting_asset() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, 0, 100));

    let (public_key, secret_key) = mine_wallet(&mut testkit);
    let (recipient_key, _) = crypto::gen_keypair();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_transfer()
        .add_asset(meta_data, 2)
        .recipient(recipient_key)
        .seed(42)
        .build();
        
    post_tx(&api, &tx_transfer);
    testkit.create_block();

    let s = get_status(&api, &tx_transfer.hash());
    assert_eq!(Ok(Err(Error::AssetNotFound)), s);
}

#[test]
fn tranfer_insufficient_funds() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, 0, 100));

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_transfer()
        .add_asset(meta_data, 2)
        .recipient(recipient_key)
        .seed(42)
        .build();
        
    post_tx(&api, &tx_transfer);
    testkit.create_block();

    let s = get_status(&api, &tx_transfer.hash());
    assert_eq!(Ok(Err(Error::InsufficientFunds)), s);
}