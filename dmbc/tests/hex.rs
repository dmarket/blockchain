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
use iron::headers::Headers;
use exonum::crypto;
use exonum::messages::Message;
use exonum_testkit::TestKit;
use dmbc_testkit::{DmbcTestKit, DmbcTestKitApi};

use dmbc::currency::assets::MetaAsset;
use dmbc::currency::api::error::ApiError;
use dmbc::currency::api::hex::{HexApi, HexApiResponse, HexResponse};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::transactions::components::FeeStrategy;

#[test]
fn hex_tx() {
    let api = TestKit::default().api();

    let (b_pk, b_sk) = crypto::gen_keypair();
    let (s_pk, s_sk) = crypto::gen_keypair();

    let trade = transaction::Builder::new()
        .keypair(b_pk, b_sk)
        .tx_trade_assets()
        .seller(s_pk, s_sk)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    let (status, response): (StatusCode, HexApiResponse) = api.post_with_status(
        "/v1/hex/transactions",
        &trade
    );
    let hex = HexApi::hex_string(trade.raw().body().to_vec());

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Some(HexResponse { hex })));
}

#[test]
fn hex_offer_tx() {
    let api = TestKit::default().api();

    let (b_pk, b_sk) = crypto::gen_keypair();
    let (s_pk, s_sk) = crypto::gen_keypair();

    let trade = transaction::Builder::new()
        .keypair(b_pk, b_sk)
        .tx_trade_assets()
        .seller(s_pk, s_sk)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    let (status, response): (StatusCode, HexApiResponse) = api.post_with_status(
        "/v1/hex/transactions/offer",
        &trade
    );
    let hex = HexApi::hex_string(trade.offer_raw().to_vec());

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Some(HexResponse { hex })));
}

#[test]
fn hex_offer_tx_without_offer() {
    let (public_key, secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let fees = dmbc_testkit::asset_fees(10, "0.1".parse().unwrap());
    let meta_asset = MetaAsset::new(&receiver_key, "asset", 5, fees);

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(85)
        .build();

    let api = TestKit::default().api();
    let (status, response): (StatusCode, HexApiResponse) = api.post_with_status(
        "/v1/hex/transactions/offer",
        &tx_add_assets
    );

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(None));
}

#[test]
fn hex_transactions_empty_request_body() {
    let api = TestKit::default().api();

    let (status, response): (StatusCode, HexApiResponse) = api.post_raw_with_status(
        "/v1/hex/transactions",
        ""
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::IncorrectRequest));
}

#[test]
fn hex_transactions_offer_empty_request_body() {
    let api = TestKit::default().api();

    let (status, response): (StatusCode, HexApiResponse) = api.post_raw_with_status(
        "/v1/hex/transactions/offer",
        ""
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::IncorrectRequest));
}

#[test]
fn hex_transactions_empty_request_body_empty_headers() {
    let api = TestKit::default().api();

    let (status, response): (StatusCode, HexApiResponse) = api.post_raw_with_status2(
        "/v1/hex/transactions",
        Headers::new(),
        ""
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::EmptyRequestBody));
}

#[test]
fn hex_transactions_offer_empty_request_body_empty_headers() {
    let api = TestKit::default().api();

    let (status, response): (StatusCode, HexApiResponse) = api.post_raw_with_status2(
        "/v1/hex/transactions/offer",
        Headers::new(),
        ""
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::EmptyRequestBody));
}
