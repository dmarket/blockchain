use api::*;
use std::collections::HashMap;
use iron::headers::{ContentType, Headers};
use iron_test::{request, response};
use hyper::status::StatusCode;

use exonum_testkit::ApiKind;

use dmbc::currency::SERVICE_NAME;
use dmbc::currency::api::assets_intern::{AssetIdResponse, AssetIdResponseBody, AssetIdRequest, 
                                        AssetIdBatchRequest, AssetIdBatchResponse, AssetIdBatchResponseBody};
use dmbc::currency::assets::AssetId;
use dmbc::currency::api::error::ApiError;

#[test]
fn assets_intern_id_from_meta() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let meta_data = "asset";

    let (pub_key, _, _) = WalletMiner::new().mine(&mut testkit);

    let response: AssetIdResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("/v1/intern/assets/{}/{}", pub_key.to_string(), meta_data),
    );

    let id = AssetId::from_data(meta_data, &pub_key);
    let mut assets = HashMap::new();
    assets.insert(meta_data.to_string(), id.to_string());

    assert_eq!(response, Ok(AssetIdResponseBody{ assets }));
}

#[test]
fn assets_intern_id_from_meta_invalid_public_key() {
    let testkit = init_testkit();
    let api = testkit.api();
    let url = format!("{}{}", TEST_KIT_SERVICE_URL, "/v1/intern/assets/invalidpublickey/meta_dummy");
    let mut headers = Headers::new();
    headers.set(ContentType::json());

    let response = request::get(&url, headers, api.public_mount()).unwrap();

    let status = response.status.unwrap();
    let body = response::extract_body_to_string(response);
    let response: AssetIdResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}

#[test]
fn assets_intern_ids_from_meta() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let (pub_key, _, _) = WalletMiner::new().mine(&mut testkit);

    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let id0 = AssetId::from_data(meta_data0, &pub_key);
    let id1 = AssetId::from_data(meta_data1, &pub_key);
    let assets = vec![meta_data0.to_string(), meta_data1.to_string()];
    
    let request_body = serde_json::to_string(&AssetIdRequest { assets }).unwrap();

    let url = format!("{}{}{}", TEST_KIT_SERVICE_URL, "/v1/intern/assets/", pub_key.to_string());
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    let response = request::post(&url, headers, &request_body, api.public_mount()).unwrap();

    let status = response.status.unwrap();
    let body = response::extract_body_to_string(response);
    let response: AssetIdResponse = serde_json::from_str(&body).unwrap();

    let mut assets = HashMap::new();
    assets.insert(meta_data0.to_string(), id0.to_string());
    assets.insert(meta_data1.to_string(), id1.to_string());

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(AssetIdResponseBody{ assets }));
}

#[test]
fn assets_intern_ids_from_meta_invalid_public_key() {
    let testkit = init_testkit();
    let api = testkit.api();

    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let assets = vec![meta_data0.to_string(), meta_data1.to_string()];
    
    let request_body = serde_json::to_string(&AssetIdRequest { assets }).unwrap();

    let url = format!("{}{}", TEST_KIT_SERVICE_URL, "/v1/intern/assets/invalidpublickey");
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    let response = request::post(&url, headers, &request_body, api.public_mount()).unwrap();

    let status = response.status.unwrap();
    let body = response::extract_body_to_string(response);
    let response: AssetIdResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}

#[test]
fn assets_intern_batch_ids() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let (pub_key0, _, _) = WalletMiner::new().mine(&mut testkit);
    let (pub_key1, _, _) = WalletMiner::new().mine(&mut testkit);

    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let assets = vec![meta_data0.to_string(), meta_data1.to_string()];

    let mut assets_map = HashMap::new();
    assets_map.insert(pub_key0.to_string(), assets.clone());
    assets_map.insert(pub_key1.to_string(), assets);

    let request_body = serde_json::to_string(&AssetIdBatchRequest { assets: assets_map }).unwrap();

    let url = format!("{}{}", TEST_KIT_SERVICE_URL, "/v1/intern/assets");
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    let response = request::post(&url, headers, &request_body, api.public_mount()).unwrap();

    let status = response.status.unwrap();
    let body = response::extract_body_to_string(response);
    let response: AssetIdBatchResponse = serde_json::from_str(&body).unwrap();

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
    assert_eq!(response, Ok(AssetIdBatchResponseBody { assets: response_map }));
}

#[test]
fn assets_intern_batch_ids_invalid_public_key() {
    let testkit = init_testkit();
    let api = testkit.api();

    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let assets = vec![meta_data0.to_string(), meta_data1.to_string()];

    let mut assets_map = HashMap::new();
    assets_map.insert("InvalidPublicKey".to_string(), assets);

    let request_body = serde_json::to_string(&AssetIdBatchRequest { assets: assets_map }).unwrap();

    let url = format!("{}{}", TEST_KIT_SERVICE_URL, "/v1/intern/assets");
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    let response = request::post(&url, headers, &request_body, api.public_mount()).unwrap();

    let status = response.status.unwrap();
    let body = response::extract_body_to_string(response);
    let response: AssetIdBatchResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}