use api::*;
use std::collections::HashMap;
use iron::headers::Headers;
use iron_test::{request, response};

use exonum_testkit::ApiKind;

use dmbc::currency::SERVICE_NAME;
use dmbc::currency::api::wallet::{WalletResponse, WalletsResponse, WalletInfo, 
                                WalletsResponseBody, WalletAssetsResponse, WalletAssetsResponseBody, ExtendedAsset};
use dmbc::currency::api::wallet;
use dmbc::currency::assets::{AssetBundle, AssetInfo};
use dmbc::currency::wallet::Wallet;
use dmbc::currency::api::error::ApiError;

use common;

#[test]
fn wallet() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let tax = 10;
    let units = 2;
    let meta_data = "asset";

    let (pub_key, _, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let response: WalletResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/wallets/{}", pub_key.to_string()),
    );

    let asset = AssetBundle::from_data(meta_data, units, &pub_key);
    let wallet = Wallet::new(100000000, vec![asset]);

    assert_eq!(response, Ok(wallet));
}

#[test]
fn wallets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let tax = 10;
    let units = 2;
    let meta_data = "asset";

    let (pub_key1, _, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (pub_key2, _, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let genesis_key = common::default_genesis_wallet();
    let genesis = genesis_wallet(&api);
    let genesis_count_assets = genesis.assets().len() as u64;

    let mut wallets = HashMap::new();
    wallets.insert(genesis_key, WalletInfo {balance: genesis.balance(), count_assets: genesis_count_assets});
    wallets.insert(pub_key1, WalletInfo {balance: 100000000, count_assets: 1});
    wallets.insert(pub_key2, WalletInfo {balance: 100000000, count_assets: 1});
    let total = wallets.len() as u64;
    let count = wallets.len() as u64;

    let response: WalletsResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        "v1/wallets",
    );

    assert_eq!(response, Ok(WalletsResponseBody { total, count, wallets } ));
}

#[test]
fn wallets_pagination() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let tax = 10;
    let units = 2;
    let meta_data = "asset";

    let (_, _, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (_, _, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let total = 3;
    let count = 1;

    let response: WalletsResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        "v1/wallets?offset=0&limit=1",
    );

    assert!(response.is_ok());

    let body = response.unwrap();
    assert_eq!(body.total, total);
    assert_eq!(body.count, count);
}

#[test]
fn wallet_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let tax = 10;
    let units = 2;
    let meta_data0 = "asset0";
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";

    let (pub_key, _, _) = WalletMiner::new()
        .add_asset(meta_data0, units, asset_fee(tax, 0))
        .add_asset(meta_data1, units, asset_fee(tax, 0))
        .add_asset(meta_data2, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let response: WalletAssetsResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/wallets/{}/assets", pub_key.to_string()),
    );

    let asset0 = AssetBundle::from_data(meta_data0, units, &pub_key);
    let asset1 = AssetBundle::from_data(meta_data1, units, &pub_key);
    let asset2 = AssetBundle::from_data(meta_data2, units, &pub_key);

    let assets = vec![ExtendedAsset::from_asset(&asset0, None),
                        ExtendedAsset::from_asset(&asset1, None),
                        ExtendedAsset::from_asset(&asset2, None)];
    let total = assets.len() as u64;
    let count = assets.len() as u64;

    assert_eq!(response, Ok(WalletAssetsResponseBody { total, count, assets }));
}

#[test]
fn wallet_assets_meta_data() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let tax = 10;
    let units = 2;
    let meta_data = "asset";

    let (pub_key, _, hash) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let response: WalletAssetsResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/wallets/{}/assets?{}=true", pub_key.to_string(), wallet::PARAMETER_META_DATA_KEY),
    );

    let asset = AssetBundle::from_data(meta_data, units, &pub_key);
    let info = AssetInfo::new(&pub_key, &hash.unwrap(), units, asset_fee(tax, 0));

    let assets = vec![ExtendedAsset::from_asset(&asset, Some(info))];
    let total = assets.len() as u64;
    let count = assets.len() as u64;

    assert_eq!(response, Ok(WalletAssetsResponseBody { total, count, assets }));
}

#[test]
fn wallet_invalid_public_key() {
    let testkit = init_testkit();
    let api = testkit.api();

    let url = format!("http://localhost:3000/{}/{}", "api/services/cryptocurrency", "v1/wallets/invalidpubkey");
    let res = request::get(&url, Headers::new(), api.public_mount());
    let iron_response = res.unwrap();
    let status = iron_response.status;
    assert_eq!(Some(ApiError::IncorrectRequest.to_status()), status);
    let body = response::extract_body_to_string(iron_response);
    let iron_body_response: WalletResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(Err(ApiError::WalletHexInvalid), iron_body_response);
}

#[test]
fn wallet_assets_invalid_public_key() {
    let testkit = init_testkit();
    let api = testkit.api();

    let url = format!("http://localhost:3000/{}/{}", "api/services/cryptocurrency", "v1/wallets/invalidpubkey/assets");
    let res = request::get(&url, Headers::new(), api.public_mount());
    let iron_response = res.unwrap();
    let status = iron_response.status;
    assert_eq!(Some(ApiError::IncorrectRequest.to_status()), status);
    let body = response::extract_body_to_string(iron_response);
    let iron_body_response: WalletAssetsResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(Err(ApiError::WalletHexInvalid), iron_body_response);
}
