extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate mount;

pub mod dmbc_testkit;

use std::collections::HashMap;

use hyper::status::StatusCode;
use exonum::crypto;
use dmbc_testkit::{DmbcTestKit, DmbcTestApiBuilder, DmbcTestKitApi};

use dmbc::currency::api::wallet::{self, ExtendedAsset, WalletAssetsResponse, WalletAssetsResponseBody,
                                  WalletInfo, WalletResponse, WalletsResponse, WalletsResponseBody, 
                                  WalletAssetResponse};
use dmbc::currency::wallet::Wallet;
use dmbc::currency::assets::AssetId;
use dmbc::currency::api::error::ApiError;

#[test]
fn wallet() {
    let tax = 10;
    let units = 2;
    let balance = 1000;
    let meta_data = "asset";

    let (pub_key, _) = crypto::gen_keypair();
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, 0), &pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .add_wallet_value(&pub_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&pub_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let (status, response): (StatusCode, WalletResponse) = api.get_with_status(
        &format!("/v1/wallets/{}", pub_key.to_string())
    );

    let wallet = Wallet::new(balance, vec![asset]);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(WalletInfo::from(wallet)));
}

#[test]
fn wallets() {
    let tax = 10;
    let units = 2;
    let balance = 1000;
    let meta_data = "asset";

    let (pub_key1, _) = crypto::gen_keypair();
    let (pub_key2, _) = crypto::gen_keypair();

    let asset1 = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, 0), &pub_key1);
    let asset2 = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, 0), &pub_key2);

    let mut testkit = DmbcTestApiBuilder::new()
        .add_wallet_value(&pub_key1, Wallet::new(balance, vec![]))
        .add_wallet_value(&pub_key2, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&pub_key1, asset1)
        .add_asset_to_wallet(&pub_key2, asset2)
        .create();
    let api = testkit.api();

    let genesis_key = dmbc_testkit::default_genesis_key();
    let genesis = testkit.fetch_wallet(&genesis_key);
    let genesis_count_assets = genesis.assets().len() as u64;

    let mut wallets = HashMap::new();
    wallets.insert(
        genesis_key,
        WalletInfo {
            balance: genesis.balance(),
            assets_count: genesis_count_assets,
        },
    );
    wallets.insert(
        pub_key1,
        WalletInfo {
            balance: balance,
            assets_count: 1,
        },
    );
    wallets.insert(
        pub_key2,
        WalletInfo {
            balance: balance,
            assets_count: 1,
        },
    );
    let total = wallets.len() as u64;
    let count = wallets.len() as u64;

    let (status, response): (StatusCode, WalletsResponse) = api.get_with_status(
        "/v1/wallets"
    );

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(
        response,
        Ok(WalletsResponseBody {
            total,
            count,
            wallets
        })
    );
}

#[test]
fn wallets_pagination() {
    let tax = 10;
    let units = 2;
    let meta_data = "asset";

    let (pub_key1, _) = crypto::gen_keypair();
    let (pub_key2, _) = crypto::gen_keypair();

    let asset1 = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, 0), &pub_key1);
    let asset2 = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, 0), &pub_key2);

    let testkit = DmbcTestApiBuilder::new()
        .add_asset_to_wallet(&pub_key1, asset1)
        .add_asset_to_wallet(&pub_key2, asset2)
        .create();

    let api = testkit.api();

    let total = 3;
    let count = 1;

    let (status, response): (StatusCode, WalletsResponse) = api.get_with_status(
        "/v1/wallets?offset=0&limit=1",
    );

    assert_eq!(status, StatusCode::Ok);
    assert!(response.is_ok());

    let body = response.unwrap();
    assert_eq!(body.total, total);
    assert_eq!(body.count, count);
}

#[test]
fn wallet_assets() {
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";

    let (pub_key, _) = crypto::gen_keypair();
    let (asset0, info0) = dmbc_testkit::create_asset(meta_data0, units, dmbc_testkit::asset_fees(tax, 0), &pub_key);
    let (asset1, info1) = dmbc_testkit::create_asset(meta_data1, units, dmbc_testkit::asset_fees(tax, 0), &pub_key);
    let (asset2, info2) = dmbc_testkit::create_asset(meta_data2, units, dmbc_testkit::asset_fees(tax, 0), &pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .add_asset_to_wallet(&pub_key, (asset0.clone(), info0.clone()))
        .add_asset_to_wallet(&pub_key, (asset1.clone(), info1.clone()))
        .add_asset_to_wallet(&pub_key, (asset2.clone(), info2.clone()))
        .create();
    let api = testkit.api();

    let (status, response): (StatusCode, WalletAssetsResponse) = api.get_with_status(
        &format!("/v1/wallets/{}/assets?{}=true", pub_key.to_string(), wallet::PARAMETER_META_DATA_KEY)
    );

    let assets = vec![
        ExtendedAsset::from_asset(&asset0, Some(info0)),
        ExtendedAsset::from_asset(&asset1, Some(info1)),
        ExtendedAsset::from_asset(&asset2, Some(info2)),
    ];
    let total = assets.len() as u64;
    let count = assets.len() as u64;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(
        response,
        Ok(WalletAssetsResponseBody {
            total,
            count,
            assets
        })
    );
}

#[test]
fn wallet_assets_meta_data() {
    let tax = 10;
    let units = 2;
    let meta_data = "asset";

    let (pub_key, _) = crypto::gen_keypair();
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, 0), &pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .add_asset_to_wallet(&pub_key, (asset.clone(), info.clone()))
        .create();

    let api = testkit.api();

    let (status, response): (StatusCode, WalletAssetsResponse) = api.get_with_status(
        &format!(
            "/v1/wallets/{}/assets?{}=true",
            pub_key.to_string(),
            wallet::PARAMETER_META_DATA_KEY
        ),
    );

    let assets = vec![ExtendedAsset::from_asset(&asset, Some(info))];
    let total = assets.len() as u64;
    let count = assets.len() as u64;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(
        response,
        Ok(WalletAssetsResponseBody {
            total,
            count,
            assets
        })
    );
}

#[test]
fn wallet_empty_with_valid_key() {
    let testkit = DmbcTestApiBuilder::new().create();
    let api = testkit.api();

    let (pub_key, _) = crypto::gen_keypair();
    let (status, response): (StatusCode, WalletResponse) = api.get_with_status(
        &format!("/v1/wallets/{}", pub_key.to_string()),
    );

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(WalletInfo::from(Wallet::new_empty())));
}

#[test]
fn wallet_invalid_public_key() {
    let testkit = DmbcTestApiBuilder::new().create();
    let api = testkit.api();

    let (status, response): (StatusCode, WalletResponse) = api.get_with_status(
        "/v1/wallets/invalidpubkey"
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}

#[test]
fn wallet_assets_invalid_public_key() {
    let testkit = DmbcTestApiBuilder::new().create();
    let api = testkit.api();

    let (status, response): (StatusCode, WalletAssetsResponse) = api.get_with_status(
        "/v1/wallets/invalidpubkey/assets"
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}

#[test]
fn wallet_asset_from_empty_wallet() {
    let testkit = DmbcTestApiBuilder::new().create();
    let api = testkit.api();
    let meta_data = "asset";

    let (pub_key, _) = crypto::gen_keypair();
    let id = AssetId::from_data(meta_data, &pub_key);
    let (status, response): (StatusCode, WalletAssetResponse) = api.get_with_status(
        &format!("/v1/wallets/{}/assets/{}", pub_key.to_string(), id.to_string()),
    );

    assert_eq!(status, StatusCode::NotFound);
    assert_eq!(response, Err(ApiError::AssetIdNotFound));
}

#[test]
fn wallet_asset_invalid_asset_id() {
    let testkit = DmbcTestApiBuilder::new().create();
    let api = testkit.api();

    let (pub_key, _) = crypto::gen_keypair();
    let (status, response): (StatusCode, WalletAssetResponse) = api.get_with_status(
        &format!("/v1/wallets/{}/assets/invalid_asset_id", pub_key.to_string()),
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::AssetIdInvalid));    
}

#[test]
fn wallet_asset_invalid_public_key() {
    let testkit = DmbcTestApiBuilder::new().create();
    let api = testkit.api();
    let meta_data = "asset";

    let (pub_key, _) = crypto::gen_keypair();
    let id = AssetId::from_data(meta_data, &pub_key);
    let (status, response): (StatusCode, WalletAssetResponse) = api.get_with_status(
        &format!("/v1/wallets/invalid_public_key/assets/{}", id.to_string()),
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));    
}

#[test]
fn wallet_asset() {
    let meta_data = "asset";
    let units = 6;
    let tax = 11;
    let (pub_key, _) = crypto::gen_keypair();
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, 0), &pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .add_wallet_value(&pub_key, Wallet::new(0, vec![]))
        .add_asset_to_wallet(&pub_key, (asset.clone(), info))
        .create();
    let api = testkit.api();    

    let (status, response): (StatusCode, WalletAssetResponse) = api.get_with_status(
        &format!("/v1/wallets/{}/assets/{}", pub_key.to_string(), asset.id().to_string()),
    );

    let extended_asset = ExtendedAsset::from_asset(&asset, None);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(extended_asset));    
}

#[test]
fn wallet_asset_wiht_info() {
    let meta_data = "asset";
    let units = 6;
    let tax = 11;
    let (pub_key, _) = crypto::gen_keypair();
    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(tax, 0), &pub_key);

    let testkit = DmbcTestApiBuilder::new()
        .add_wallet_value(&pub_key, Wallet::new(0, vec![]))
        .add_asset_to_wallet(&pub_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();    

    let (status, response): (StatusCode, WalletAssetResponse) = api.get_with_status(
        &format!(
            "/v1/wallets/{}/assets/{}?{}=true", 
            pub_key.to_string(), 
            asset.id().to_string(), 
            wallet::PARAMETER_META_DATA_KEY
        ),
    );

    let extended_asset = ExtendedAsset::from_asset(&asset, Some(info));

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(extended_asset));    
}