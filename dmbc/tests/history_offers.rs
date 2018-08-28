extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate mount;
extern crate serde_json;

pub mod dmbc_testkit;

use std::collections::HashMap;

use dmbc_testkit::{DmbcTestApiBuilder, DmbcTestKitApi};

use exonum::crypto;
use exonum::messages::Message;
use hyper::status::StatusCode;

use dmbc::currency::api::history_offers::{HistoryOffersResponse, HistoryOffersInfo, HistoryOfferInfo, HistoryOfferResult};
use dmbc::currency::api::error::ApiError;

use dmbc::currency::offers::history::{HistoryOffers, HistoryOffer};

use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::TradeAsset;
use dmbc::currency::wallet::Wallet;
//use dmbc::currency::offers::{Offer};

#[test]
fn history_offers() {
    let fixed = 10;
    let units = 2;
    let balance = 1000;
    let meta_data = "asset data";

    let (user1_pk, user1_sk) = crypto::gen_keypair();
    let (user2_pk, user2_sk) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &user1_pk,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .add_wallet_value(&user1_pk, Wallet::new(balance, vec![]))
        .add_wallet_value(&user2_pk, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&user1_pk, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_bid_offer = transaction::Builder::new()
        .keypair(user1_pk, user1_sk.clone())
        .tx_offer()
        .asset(TradeAsset::from_bundle(asset.clone(), 100))
        .data_info("bid")
        .bid_build();

    let (status, _) = api.post_tx(&tx_bid_offer);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);

    let tx_ask_offer = transaction::Builder::new()
        .keypair(user2_pk, user2_sk.clone())
        .tx_offer()
        .asset(TradeAsset::new(asset.id(), asset.amount() * 2, 100))
        .data_info("ask")
        .ask_build();


    let (status, _) = api.post_tx(&tx_ask_offer);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);

    let mut offers_info = HashMap::new();
    offers_info.insert(tx_bid_offer.hash(), HistoryOfferInfo{tx_amount: 1});
    offers_info.insert(tx_ask_offer.hash(), HistoryOfferInfo{tx_amount: 1});

    let (status, response): (StatusCode, HistoryOffersResponse) = api.get_with_status("/v1/history/offers");

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(
        HistoryOffersInfo{
            total:2,
            count:2,
            offer_info: offers_info,
        }
    ));
}

#[test]
fn history_offers_invalid() {
    let testkit = DmbcTestApiBuilder::new().create();
    let api = testkit.api();

    let (status, response): (StatusCode, HistoryOfferResult) = api.get_with_status("/v1/history/offers/123");
    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::TransactionHashInvalid))
}


#[test]
fn history_offers_by_tx_hash() {
    let fixed = 10;
    let units = 2;
    let balance = 1000;
    let meta_data = "asset data";

    let (user1_pk, user1_sk) = crypto::gen_keypair();
    let (user2_pk, user2_sk) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &user1_pk,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .add_wallet_value(&user1_pk, Wallet::new(balance, vec![]))
        .add_wallet_value(&user2_pk, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&user1_pk, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_bid_offer = transaction::Builder::new()
        .keypair(user1_pk, user1_sk.clone())
        .tx_offer()
        .asset(TradeAsset::from_bundle(asset.clone(), 100))
        .data_info("bid")
        .bid_build();

    let (status, _) = api.post_tx(&tx_bid_offer);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);

    let tx_ask_offer = transaction::Builder::new()
        .keypair(user2_pk, user2_sk.clone())
        .tx_offer()
        .asset(TradeAsset::new(asset.id(), asset.amount() * 2, 100))
        .data_info("ask")
        .ask_build();


    let (status, _) = api.post_tx(&tx_ask_offer);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);

    let endpoint = "/v1/history/offers/".to_string() + &tx_bid_offer.hash().to_string();
    let history_offers = HistoryOffers::new(vec![HistoryOffer::new(&tx_ask_offer.hash(), 2)]);

    let (status, response): (StatusCode, HistoryOfferResult) = api.get_with_status(&endpoint);
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Some(history_offers)));
}
