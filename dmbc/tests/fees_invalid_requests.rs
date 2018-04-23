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
use exonum_testkit::TestKit;
use dmbc_testkit::{DmbcTestKit, DmbcTestKitApi};

use dmbc::currency::api::error::ApiError;
use dmbc::currency::api::fees::FeesResponse;


#[test]
fn fees_empty_request_body() {
    let api = TestKit::default().api();

    let (status, response): (StatusCode, FeesResponse) = api.post_raw_with_status(
        "/v1/fees/transactions",
        ""
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::IncorrectRequest));
}

#[test]
fn fees_empty_request_body_empty_headers() {
    let api = TestKit::default().api();

    let (status, response): (StatusCode, FeesResponse) = api.post_raw_with_status2(
        "/v1/hex/transactions",
        Headers::new(),
        ""
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::EmptyRequestBody));
}