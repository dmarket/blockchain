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
use evo_testkit::{EvoTestKit, EvoTestApiBuilder, EvoTestKitApi, asset_fees, create_asset, default_genesis_key};

use dmbc::currency::api::wallet::{self, ExtendedAsset, WalletAssetsResponse, WalletAssetsResponseBody,
                                  WalletInfo, WalletResponse, WalletsResponse, WalletsResponseBody};
use dmbc::currency::wallet::Wallet;
use dmbc::currency::api::error::ApiError;

#[test]
fn wallet() {
    let tax = 10;
    let units = 2;
    let balance = 1000;
    let meta_data = "asset";

    let (pub_key, _) = crypto::gen_keypair();
    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &pub_key);

    let testkit = EvoTestApiBuilder::new()
        .add_wallet_value(&pub_key, Wallet::new(balance, vec![]))
        .add_asset_value_to_wallet(&pub_key, asset.clone(), info)
        .create();
    let api = testkit.api();

    let (status, response): (StatusCode, WalletResponse) = api.get_with_status(
        &format!("/v1/wallets/{}", pub_key.to_string())
    );

    let wallet = Wallet::new(balance, vec![asset]);

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response, Ok(wallet));
}

#[test]
fn wallets() {
    let tax = 10;
    let units = 2;
    let balance = 1000;
    let meta_data = "asset";

    let (pub_key1, _) = crypto::gen_keypair();
    let (pub_key2, _) = crypto::gen_keypair();

    let (asset1, info1) = create_asset(meta_data, units, asset_fees(tax, 0), &pub_key1);
    let (asset2, info2) = create_asset(meta_data, units, asset_fees(tax, 0), &pub_key2);

    let mut testkit = EvoTestApiBuilder::new()
        .add_wallet_value(&pub_key1, Wallet::new(balance, vec![]))
        .add_wallet_value(&pub_key2, Wallet::new(balance, vec![]))
        .add_asset_value_to_wallet(&pub_key1, asset1, info1)
        .add_asset_value_to_wallet(&pub_key2, asset2, info2)
        .create();
    let api = testkit.api();

    let genesis_key = default_genesis_key();
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

    let testkit = EvoTestApiBuilder::new()
        .add_asset_to_wallet(meta_data, units, asset_fees(tax, 0), &pub_key1, &pub_key1)
        .add_asset_to_wallet(meta_data, units, asset_fees(tax, 0), &pub_key2, &pub_key2)
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
    let (asset0, info0) = create_asset(meta_data0, units, asset_fees(tax, 0), &pub_key);
    let (asset1, info1) = create_asset(meta_data1, units, asset_fees(tax, 0), &pub_key);
    let (asset2, info2) = create_asset(meta_data2, units, asset_fees(tax, 0), &pub_key);

    let testkit = EvoTestApiBuilder::new()
        .add_asset_value_to_wallet(&pub_key, asset0.clone(), info0.clone())
        .add_asset_value_to_wallet(&pub_key, asset1.clone(), info1.clone())
        .add_asset_value_to_wallet(&pub_key, asset2.clone(), info2.clone())
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
    let (asset, info) = create_asset(meta_data, units, asset_fees(tax, 0), &pub_key);

    let testkit = EvoTestApiBuilder::new()
        .add_asset_value_to_wallet(&pub_key, asset.clone(), info.clone())
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
fn wallet_invalid_public_key() {
    let testkit = EvoTestApiBuilder::new().create();
    let api = testkit.api();

    let (status, response): (StatusCode, WalletResponse) = api.get_with_status(
        "/v1/wallets/invalidpubkey"
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}

#[test]
fn wallet_assets_invalid_public_key() {
    let testkit = EvoTestApiBuilder::new().create();
    let api = testkit.api();

    let (status, response): (StatusCode, WalletAssetsResponse) = api.get_with_status(
        "/v1/wallets/invalidpubkey/assets"
    );

    assert_eq!(status, StatusCode::BadRequest);
    assert_eq!(response, Err(ApiError::WalletHexInvalid));
}