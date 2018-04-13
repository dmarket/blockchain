extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate mount;

pub mod evo_testkit;

use std::collections::HashMap;

use hyper::status::StatusCode;
use exonum::crypto;
use exonum_testkit::TestKit;
use evo_testkit::{EvoTestKit, EvoTestKitApi};

use dmbc::currency::api::assets_intern::{AssetIdBatchResponse, AssetIdRequest,
                                         AssetIdBatchResponseBody, AssetIdBatchRequest,
                                         AssetIdResponse, AssetIdResponseBody};
use dmbc::currency::api::error::ApiError;
use dmbc::currency::assets::AssetId;

#[test]
fn intern_assets_id_from_meta() {
    let mut testkit = TestKit::default();
    let api = testkit.api();
    let balance = 1000;
    let meta_data = "asset";

    let (pub_key, _) = crypto::gen_keypair();
    testkit.create_wallet(&pub_key, balance);

    let (status, response): (StatusCode, AssetIdResponse) = api.get_with_status(
        &format!("/v1/intern/assets/{}/{}", pub_key.to_string(), meta_data),
    );

    let id = AssetId::from_data(meta_data, &pub_key);
    let mut assets = HashMap::new();
    assets.insert(meta_data.to_string(), id.to_string());

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(AssetIdResponseBody { assets }));
}

#[test]
fn intern_assets_id_from_meta_invalid_public_key() {
    let testkit = TestKit::default();
    let api = testkit.api();

    let (status, response): (StatusCode, AssetIdResponse) = api.get_with_status(
        "/v1/intern/assets/invalidpublickey/meta_dummy",
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}

#[test]
fn intern_assets_ids_from_meta() {
    let testkit = TestKit::default();
    let api = testkit.api();

    let (pub_key, _) = crypto::gen_keypair();

    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let id0 = AssetId::from_data(meta_data0, &pub_key);
    let id1 = AssetId::from_data(meta_data1, &pub_key);
    let assets = vec![meta_data0.to_string(), meta_data1.to_string()];

    let (status, response): (StatusCode, AssetIdResponse) = api.post_with_status(
        &format!("/v1/intern/assets/{}", pub_key.to_string()),
        &AssetIdRequest { assets },
    );

    let mut assets = HashMap::new();
    assets.insert(meta_data0.to_string(), id0.to_string());
    assets.insert(meta_data1.to_string(), id1.to_string());

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(AssetIdResponseBody { assets }));
}

#[test]
fn intern_assets_ids_from_meta_invalid_public_key() {
    let testkit = TestKit::default();
    let api = testkit.api();

    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let assets = vec![meta_data0.to_string(), meta_data1.to_string()];

    let (status, response): (StatusCode, AssetIdResponse) = api.post_with_status(
        "/v1/intern/assets/invalidpublickey",
        &AssetIdRequest { assets },
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}

#[test]
fn intern_assets_batch_ids() {
    let testkit = TestKit::default();
    let api = testkit.api();
    let (pub_key0, _) = crypto::gen_keypair();
    let (pub_key1, _) = crypto::gen_keypair();

    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let assets = vec![meta_data0.to_string(), meta_data1.to_string()];

    let mut assets_map = HashMap::new();
    assets_map.insert(pub_key0.to_string(), assets.clone());
    assets_map.insert(pub_key1.to_string(), assets);

    let (status, response): (StatusCode, AssetIdBatchResponse) = api.post_with_status(
        "/v1/intern/assets",
        &AssetIdBatchRequest { assets: assets_map },
    );

    let id0 = AssetId::from_data(meta_data0, &pub_key0);
    let id1 = AssetId::from_data(meta_data1, &pub_key0);
    let id2 = AssetId::from_data(meta_data0, &pub_key1);
    let id3 = AssetId::from_data(meta_data1, &pub_key1);

    let mut response_map = HashMap::new();
    let mut assets = HashMap::new();
    assets.insert(meta_data0.to_string(), id0.to_string());
    assets.insert(meta_data1.to_string(), id1.to_string());
    response_map.insert(pub_key0.to_string(), assets);

    let mut assets = HashMap::new();
    assets.insert(meta_data0.to_string(), id2.to_string());
    assets.insert(meta_data1.to_string(), id3.to_string());
    response_map.insert(pub_key1.to_string(), assets);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(
        response,
        Ok(AssetIdBatchResponseBody {
            assets: response_map
        })
    );
}

#[test]
fn intern_assets_batch_ids_invalid_public_key() {
    let testkit = TestKit::default();
    let api = testkit.api();

    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let assets = vec![meta_data0.to_string(), meta_data1.to_string()];

    let mut assets_map = HashMap::new();
    assets_map.insert("InvalidPublicKey".to_string(), assets);

    let (status, response): (StatusCode, AssetIdBatchResponse) = api.post_with_status(
        "/v1/intern/assets",
        &AssetIdBatchRequest { assets: assets_map },
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}