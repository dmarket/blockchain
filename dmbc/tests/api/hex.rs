extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;

use api::*;
use hyper::status::StatusCode;
use iron::headers::{ContentType, Headers};
use iron::Response;
use iron_test::{request, response};

use exonum::crypto;
use exonum_testkit::TestKitApi;

use dmbc::currency::api::error::ApiError;
use dmbc::currency::api::hex::{HexApi, HexApiResponse, HexResponse};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::transactions::components::FeeStrategy;

fn parse_response(iron_response: Response) -> (Option<StatusCode>, HexApiResponse) {
    let status = iron_response.status;
    let body = response::extract_body_to_string(iron_response);
    let iron_body_response: HexApiResponse = serde_json::from_str(&body).unwrap();

    (status, iron_body_response)
}

fn get_tx_hex(api: &TestKitApi, body: &str) -> (Option<StatusCode>, HexApiResponse) {
    let url = format!("{}/{}", TEST_KIT_SERVICE_URL, "v1/hex/transactions");
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    let res = request::post(&url, headers, &body, api.public_mount()).unwrap();
    parse_response(res)
}

fn get_tx_offer_hex(api: &TestKitApi, body: &str) -> (Option<StatusCode>, HexApiResponse) {
    let url = format!("{}/{}", TEST_KIT_SERVICE_URL, "v1/hex/transactions/offer");
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    let res = request::post(&url, headers, &body, api.public_mount()).unwrap();
    parse_response(res)
}

#[test]
fn hex_tx() {
    let api = init_testkit().api();

    let (public_key, secret_key) = crypto::gen_keypair();
    let mine = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_mine()
        .seed(123)
        .build();
    let tx_hex = HexApi::hex_string(mine.raw().body().to_vec());
    let tx = serde_json::to_string(&mine).unwrap();

    let (status, response_body) = get_tx_hex(&api, &tx);

    assert_eq!(Some(StatusCode::Ok), status);
    assert_eq!(Ok(Some(HexResponse { hex: tx_hex })), response_body);
}

#[test]
fn hex_offer_tx() {
    let api = init_testkit().api();

    let (b_pk, b_sk) = crypto::gen_keypair();
    let (s_pk, s_sk) = crypto::gen_keypair();

    let trade = transaction::Builder::new()
        .keypair(b_pk, b_sk)
        .tx_trade_assets()
        .seller(s_pk, s_sk)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    let tx_hex = HexApi::hex_string(trade.offer_raw().to_vec());
    let tx = serde_json::to_string(&trade).unwrap();

    let (status, response_body) = get_tx_offer_hex(&api, &tx);

    assert_eq!(Some(StatusCode::Ok), status);
    assert_eq!(Ok(Some(HexResponse { hex: tx_hex })), response_body);
}

#[test]
fn hex_offer_tx_without_offer() {
    let api = init_testkit().api();

    let (public_key, secret_key) = crypto::gen_keypair();
    let mine = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_mine()
        .seed(123)
        .build();
    let tx = serde_json::to_string(&mine).unwrap();

    let (status, response_body) = get_tx_offer_hex(&api, &tx);

    assert_eq!(Some(StatusCode::Ok), status);
    assert_eq!(Ok(None), response_body);
}

#[test]
fn hex_transactions_empty_request_body() {
    let api = init_testkit().api();

    let (status, response_body) = get_tx_hex(&api, "");

    assert_eq!(Some(ApiError::IncorrectRequest.to_status()), status);
    assert_eq!(Err(ApiError::IncorrectRequest), response_body);
}

#[test]
fn hex_transactions_offer_empty_request_body() {
    let api = init_testkit().api();

    let (status, response_body) = get_tx_offer_hex(&api, "");

    assert_eq!(Some(ApiError::IncorrectRequest.to_status()), status);
    assert_eq!(Err(ApiError::IncorrectRequest), response_body);
}

#[test]
fn hex_transactions_empty_request_body_empty_headers() {
    let api = init_testkit().api();
    let url = format!("{}/{}", TEST_KIT_SERVICE_URL, "v1/hex/transactions");
    let res = request::post(&url, Headers::new(), "", api.public_mount()).unwrap();
    let (status, body_response) = parse_response(res);

    assert_eq!(Some(ApiError::EmptyRequestBody.to_status()), status);
    assert_eq!(Err(ApiError::EmptyRequestBody), body_response);
}

#[test]
fn hex_transactions_offer_empty_request_body_empty_headers() {
    let api = init_testkit().api();
    let url = format!("{}/{}", TEST_KIT_SERVICE_URL, "v1/hex/transactions/offer");
    let res = request::post(&url, Headers::new(), "", api.public_mount()).unwrap();
    let (status, body_response) = parse_response(res);

    assert_eq!(Some(ApiError::EmptyRequestBody.to_status()), status);
    assert_eq!(Err(ApiError::EmptyRequestBody), body_response);
}
