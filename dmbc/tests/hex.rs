extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate mount;

pub mod evo_testkit;

use hyper::status::StatusCode;
use iron::headers::Headers;
use exonum::crypto;
use exonum::messages::Message;
use exonum_testkit::TestKit;
use evo_testkit::{EvoTestKit, EvoTestKitApi};

use dmbc::currency::api::error::ApiError;
use dmbc::currency::api::hex::{HexApi, HexApiResponse, HexResponse};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::transactions::components::FeeStrategy;

#[test]
fn hex_tx() {
    let api = TestKit::default().api();

    let (public_key, secret_key) = crypto::gen_keypair();
    let mine = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_mine()
        .seed(123)
        .build();
    
    let (status, response): (StatusCode, HexApiResponse) = api.post_with_status(
        "/v1/hex/transactions",
        &mine
    );
    let hex = HexApi::hex_string(mine.raw().body().to_vec());

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
    let api = TestKit::default().api();

    let (public_key, secret_key) = crypto::gen_keypair();
    let mine = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_mine()
        .seed(123)
        .build();

    let (status, response): (StatusCode, HexApiResponse) = api.post_with_status(
        "/v1/hex/transactions/offer",
        &mine
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