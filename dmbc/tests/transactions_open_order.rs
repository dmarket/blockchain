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
use dmbc::currency::api::error::ApiError;
use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;

#[test]
fn add_bid() {
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
