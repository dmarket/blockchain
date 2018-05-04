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
use exonum::crypto;
use dmbc_testkit::{DmbcTestKitApi, DmbcTestApiBuilder};

use dmbc::currency::api::error::ApiError;
use dmbc::currency::api::asset::AssetResponse;

#[test]
fn asset_is_in_blockchain() {
    let meta_data = "asset";
    let units = 2;
    let fixed = 10;
    let (public_key, _) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let testkit = DmbcTestApiBuilder::new()
        .add_asset_info(&asset.id(), info.clone())
        .create();

    let api = testkit.api();

    let (status, response): (StatusCode, AssetResponse) = api.get_with_status(
        &format!("/v1/assets/{}", asset.id().to_string())
    );

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(Some(info)));
}

#[test]
fn asset_isnt_in_blockchain() {
    let meta_data = "asset";
    let units = 2;
    let fixed = 10;
    let (public_key, _) = crypto::gen_keypair();

    let (asset, _) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let testkit = DmbcTestApiBuilder::new()
        .create();

    let api = testkit.api();

    let (status, response): (StatusCode, AssetResponse) = api.get_with_status(
        &format!("/v1/assets/{}", asset.id().to_string())
    );

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(None));
}

#[test]
fn asset_invalid_id() {
    let testkit = DmbcTestApiBuilder::new()
        .create();

    let api = testkit.api();

    let (status, response): (StatusCode, AssetResponse) = api.get_with_status(
        "/v1/assets/badassetid"
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::AssetIdInvalid));
}
