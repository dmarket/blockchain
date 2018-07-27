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
use dmbc::currency::assets::{MetaAsset, AssetBundle, AssetInfo, TradeAsset};
use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;

#[test]
fn open_bid() {
    let fixed = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (user1_pk, user1_sk) = crypto::gen_keypair();
    let (user2_pk, user2_sk) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&user1_pk, Wallet::new(balance, vec![]))
        .add_wallet_value(&user2_pk, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&user1_pk, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let asset_bundle = AssetBundle::from_data(meta_data, units, &creator_public_key);
    let trade_asset = TradeAsset::from_bundle(asset_bundle, 100);

    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(85)
        .build();

    let (status, _) = api.post_tx(&tx_add_assets);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);

    let tx_open_offer = transaction::Builder::new()
        .keypair(user1_pk, user1_sk)
        .tx_open_offer()
        .bid(true)
        .asset(trade_asset)
        .seed(100)
        .data_info("bid")
        .build();

    let (status, _) = api.post_tx(&tx_open_offer);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);

}
