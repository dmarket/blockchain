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

use dmbc::currency::configuration::{Configuration, TransactionFees, TransactionPermissions};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::{MetaAsset, AssetBundle, TradeAsset};
//use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;
use dmbc::currency::offers::{OpenOffers, Offer};

#[test]
fn set_3_bid_1_ask_result_1_bid_1_ask() {
    let fixed = 1;
    let meta_data = "asset";
    let units = 100;
    let balance = 100_000;
    let permissions = TransactionPermissions::default();
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (user1_pk, user1_sk) = crypto::gen_keypair();
    let (user2_pk, user2_sk) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&user1_pk, Wallet::new(balance, vec![]))
        .add_wallet_value(&user2_pk, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&user1_pk, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .build();

    let (status, _) = api.post_tx(&tx_add_assets);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);
    let seller_assets = api
        .get_wallet_assets(&user1_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(units, seller_assets[0].amount());

    let creator_wallet = api.get_wallet(&creator_public_key);
    assert_eq!(balance, creator_wallet.balance);

    let mut sample_offers = OpenOffers::new_open_offers();

    let bid_amount = 2;
    for bid_price in vec![10, 30, 50] {
        let asset_bundle = AssetBundle::from_data(meta_data, bid_amount, &creator_public_key);
        let trade_asset = TradeAsset::from_bundle(asset_bundle, bid_price);
        let tx_bid_offer = transaction::Builder::new()
            .keypair(user1_pk, user1_sk.clone())
            .tx_offer()
            .asset(trade_asset.clone())
            .data_info("bid")
            .bid_build();

        let (status, _) = api.post_tx(&tx_bid_offer);
        testkit.create_block();
        assert_eq!(status, StatusCode::Created);
        let offer = Offer::new(&user1_pk, bid_amount, &tx_bid_offer.hash());
        sample_offers.add_bid(bid_price, offer);
    }

    let seller_assets = api
        .get_wallet_assets(&user1_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(units - 3*bid_amount, seller_assets[0].amount());

    let ask_amount = 5;
    let ask_price = 40;
    let asset_bundle = AssetBundle::from_data(meta_data, ask_amount, &creator_public_key);
    let trade_asset = TradeAsset::from_bundle(asset_bundle.clone(), ask_price);
    let tx_ask_offer = transaction::Builder::new()
        .keypair(user2_pk, user2_sk)
        .tx_offer()
        .asset(trade_asset)
        .data_info("ask")
        .ask_build();

    let (status, _) = api.post_tx(&tx_ask_offer);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);
    let _close_bids = sample_offers.close_bid(ask_price, ask_amount);
    let offer = Offer::new(&user2_pk, 1, &tx_ask_offer.hash());
    sample_offers.add_ask(ask_price, offer);

    let creator_wallet = api.get_wallet(&creator_public_key);
    assert_eq!(balance + 4 * fixed, creator_wallet.balance);
    let buyer_wallet = api.get_wallet(&user2_pk);
    let seller_wallet = api.get_wallet(&user1_pk);
    let buyer_balance = balance - ask_price* ask_amount + (ask_price - 30) * 2 + (ask_price - 10) * 2;
    assert_eq!(buyer_balance, buyer_wallet.balance);
    assert_eq!(balance + 10 * 2 + 30 * 2 - 4 * fixed, seller_wallet.balance);

    let buyer_assets = api
        .get_wallet_assets(&user2_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(4, buyer_assets[0].amount());

    let seller_assets = api
        .get_wallet_assets(&user1_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(units - 3*bid_amount, seller_assets[0].amount());
    assert_eq!(buyer_assets[0].id(), seller_assets[0].id());

    let offers = api.get_offers(&asset_bundle.id()).unwrap();
    assert_eq!(sample_offers, offers);
}

#[test]
fn set_3_ask_1_bid_result_2_ask_1_bid() {
    let fixed = 1;
    let meta_data = "asset";
    let units = 100;
    let balance = 100_000;
    let permissions = TransactionPermissions::default();
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (user1_pk, user1_sk) = crypto::gen_keypair();
    let (user2_pk, user2_sk) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&user1_pk, Wallet::new(balance, vec![]))
        .add_wallet_value(&user2_pk, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&user1_pk, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .build();

    let (status, _) = api.post_tx(&tx_add_assets);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);
    let seller_assets = api
        .get_wallet_assets(&user1_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(units, seller_assets[0].amount());

    let creator_wallet = api.get_wallet(&creator_public_key);
    assert_eq!(balance, creator_wallet.balance);

    let ask_amount = 2;
    let asset_bundle = AssetBundle::from_data(meta_data, ask_amount, &creator_public_key);
    let mut sample_offers = OpenOffers::new_open_offers();
    for ask_price in vec![10, 30, 50] {
        let trade_asset = TradeAsset::from_bundle(asset_bundle.clone(), ask_price);
        let tx_ask_offer = transaction::Builder::new()
            .keypair(user2_pk, user2_sk.clone())
            .tx_offer()
            .asset(trade_asset)
            .data_info("ask")
            .ask_build();

        let (status, _) = api.post_tx(&tx_ask_offer);
        testkit.create_block();
        assert_eq!(status, StatusCode::Created);
        let offer = Offer::new(&user2_pk, ask_amount, &tx_ask_offer.hash());
        sample_offers.add_ask(ask_price, offer);
    }

    let offers = api.get_offers(&asset_bundle.id()).unwrap();
    assert_eq!(sample_offers, offers);


    let bid_amount = 5;
    let bid_price = 40;
    let asset_bundle = AssetBundle::from_data(meta_data, bid_amount, &creator_public_key);
    let trade_asset = TradeAsset::from_bundle(asset_bundle.clone(), bid_price);
    let tx_bid_offer = transaction::Builder::new()
        .keypair(user1_pk, user1_sk)
        .tx_offer()
        .asset(trade_asset)
        .data_info("bid")
        .bid_build();

    let (status, _) = api.post_tx(&tx_bid_offer);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);
    let _close_bids = sample_offers.close_ask(bid_price, bid_amount);
    let offer = Offer::new(&user1_pk, bid_amount - 2 , &tx_bid_offer.hash());
    sample_offers.add_bid(bid_price, offer);


    let creator_wallet = api.get_wallet(&creator_public_key);
    assert_eq!(balance + 2 * fixed, creator_wallet.balance);
    let buyer_wallet = api.get_wallet(&user2_pk);
    let seller_wallet = api.get_wallet(&user1_pk);
    let buyer_balance = balance - 10*2 - 30*2 - 50*2 + 10 * 2;
    assert_eq!(buyer_balance, buyer_wallet.balance);
    assert_eq!(balance + 40 * 2 - 2 * fixed, seller_wallet.balance);

    let buyer_assets = api
        .get_wallet_assets(&user2_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(2, buyer_assets[0].amount());

    let seller_assets = api
        .get_wallet_assets(&user1_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(units - bid_amount, seller_assets[0].amount());
    assert_eq!(buyer_assets[0].id(), seller_assets[0].id());

    let offers = api.get_offers(&asset_bundle.id()).unwrap();
    assert_eq!(sample_offers, offers);
}