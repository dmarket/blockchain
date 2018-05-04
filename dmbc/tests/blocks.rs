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
use exonum::helpers::Height;
use exonum::messages::Message;
use exonum::crypto;
use dmbc_testkit::{DmbcTestKitApi, DmbcTestApiBuilder};

use dmbc::currency::api::error::ApiError;
use dmbc::currency::api::blocks::{BlockResponse, BlocksResponse};
use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::MetaAsset;
use dmbc::currency::wallet::Wallet;

#[test]
fn blocks() {
    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();

    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status, response): (StatusCode, BlocksResponse) = api.get_with_status(
        "/v1/blocks"
    );

    assert_eq!(status, StatusCode::Ok);
    assert!(response.is_ok());

    let blocks = response.unwrap();

    for (i, block) in blocks.iter().enumerate() {
        let height = Height((blocks.len() - 1 - i) as u64);
        assert_eq!(block.height(), height);
    }
}

#[test]
fn blocks_count() {
    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();
    let count = 2;

    testkit.create_block();
    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status, response): (StatusCode, BlocksResponse) = api.get_with_status(
        &format!("/v1/blocks?count={}", count)
    );

    assert_eq!(status, StatusCode::Ok);
    assert!(response.is_ok());

    let blocks = response.unwrap();
    assert_eq!(blocks.len(), count);

    for (i, block) in blocks.iter().enumerate() {
        let height = Height((4 - i) as u64);
        assert_eq!(block.height(), height);
    }
}

#[test]
fn blocks_latest() {
    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();
    let latest = 3;

    testkit.create_block();
    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status, response): (StatusCode, BlocksResponse) = api.get_with_status(
        &format!("/v1/blocks?latest={}", latest)
    );

    assert_eq!(status, StatusCode::Ok);
    assert!(response.is_ok());

    let blocks = response.unwrap();
    assert_eq!(blocks.len(), latest);

    for (i, block) in blocks.iter().enumerate() {
        let height = Height((latest - 1 - i) as u64);
        assert_eq!(block.height(), height);
    }
}

#[test]
fn blocks_skip_empty_with_empty_blockchain() {
    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();

    testkit.create_block();
    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status, response): (StatusCode, BlocksResponse) = api.get_with_status(
        "/v1/blocks?skip_empty_blocks=true"
    );

    assert_eq!(status, StatusCode::Ok);
    assert!(response.is_ok());

    let blocks = response.unwrap();
    assert!(blocks.is_empty());
}

#[test]
fn blocks_skip_empty_with_tx_in_blockchain() {
    let fixed = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&receiver_key, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let tx_add_assets1 = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    let tx_add_assets2 = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(86)
        .build();  

    let tx_hash1 = tx_add_assets1.hash();
    let tx_hash2 = tx_add_assets2.hash();

    testkit.create_block();
    let (_, _) = api.post_tx(&tx_add_assets1);
    testkit.create_block();
    let (_, _) = api.post_tx(&tx_add_assets2);
    testkit.create_block();
    testkit.create_block();

    let (status, response): (StatusCode, BlocksResponse) = api.get_with_status(
        "/v1/blocks?skip_empty_blocks=true"
    );

    assert_eq!(status, StatusCode::Ok);
    assert!(response.is_ok());

    let blocks = response.unwrap();
    assert_eq!(blocks.len(), 2);
    let heights = vec![Height(4), Height(3)];
    let hashes = vec![tx_hash2, tx_hash1];

    for (i, block) in blocks.iter().enumerate() {
        assert_eq!(block.height(), heights[i]);
        assert_eq!(block.tx_hash().to_string(), hashes[i].to_string());
    }
}

#[test]
fn blocks_latest_out_of_bounds() {
    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();
    let latest = 10;
    let actual = 5;

    testkit.create_block();
    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status, response): (StatusCode, BlocksResponse) = api.get_with_status(
        &format!("/v1/blocks?latest={}", latest)
    );

    assert_eq!(status, StatusCode::Ok);
    assert!(response.is_ok());

    let blocks = response.unwrap();
    assert_eq!(blocks.len(), actual);

    for (i, block) in blocks.iter().enumerate() {
        let height = Height((actual - 1 - i) as u64);
        assert_eq!(block.height(), height);
    }
}

#[test]
fn blocks_latest_negative() {
    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();
    let latest = -1;

    testkit.create_block();
    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status, response): (StatusCode, BlocksResponse) = api.get_with_status(
        &format!("/v1/blocks?latest={}", latest)
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::IncorrectRequest));
}

#[test]
fn blocks_height() {
    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();
    let height = 4;

    testkit.create_block();
    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status, response): (StatusCode, BlockResponse) = api.get_with_status(
        &format!("/v1/blocks/{}", height)
    );

    assert_eq!(status, StatusCode::Ok);

    let info = response.unwrap();
    assert_eq!(info.unwrap().block.height(), Height(height));
}

#[test]
fn blocks_negative_height() {
    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();
    let height = -4;

    testkit.create_block();
    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status, _): (StatusCode, BlockResponse) = api.get_with_status(
        &format!("/v1/blocks/{}", height)
    );

    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn blocks_height_with_tx_in_blockchain() {
    let fixed = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&receiver_key, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    let tx_hash = tx_add_assets.hash();
    let height = 3;

    testkit.create_block();
    let (_, _) = api.post_tx(&tx_add_assets);
    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status, response): (StatusCode, BlockResponse) = api.get_with_status(
        &format!("/v1/blocks/{}", height)
    );

    assert_eq!(status, StatusCode::Ok);

    let info = response.unwrap().unwrap();

    assert_eq!(info.block.height(), Height(height));
    assert_eq!(info.block.tx_hash().to_string(), tx_hash.to_string());
}

#[test]
fn blocks_height_out_of_range() {
    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();
    let height = 10;

    testkit.create_block();
    testkit.create_block();
    testkit.create_block();
    testkit.create_block();

    let (status,_): (StatusCode, BlockResponse) = api.get_with_status(
        &format!("/v1/blocks/{}", height)
    );

    assert_eq!(status, StatusCode::NotFound);
    // assert_eq!(response, Ok(None));
}