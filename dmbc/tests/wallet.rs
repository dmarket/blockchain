extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate hyper;

use std::collections::HashMap;
use iron::headers::Headers;
use iron_test::{request, response};

use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};
use exonum::encoding::serialize::reexport::Serialize;
use exonum::messages::Message;

use dmbc::currency::Service;
use dmbc::currency::SERVICE_NAME;
use dmbc::currency::api::wallet::{WalletResponse, WalletsResponse, WalletInfo, 
                                WalletsResponseBody, WalletAssetsResponse, WalletAssetsResponseBody, ExtendedAsset};
use dmbc::currency::api::wallet;
use dmbc::currency::assets::{Fees, MetaAsset, AssetBundle, AssetInfo};
use dmbc::currency::api::transaction::{TxPostResponse, TransactionResponse};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::wallet::Wallet;
use dmbc::currency::api::error::ApiError;

pub fn init_testkit() -> TestKit {
    TestKitBuilder::validator()
        .with_validators(4)
        .with_service(Service::new())
        .create()
}

pub fn post_tx<T>(api: &TestKitApi, tx: &T)
    where T:Message + Serialize
{
    let tx_response:TxPostResponse = api.post(
        ApiKind::Service(SERVICE_NAME),
        "v1/transactions",
        &tx
    );

    assert_eq!(tx_response, Ok(Ok(TransactionResponse{tx_hash:tx.hash()})));
}

pub struct WalletMiner {
    public_key: PublicKey,
    secret_key: SecretKey,
    assets: Vec<MetaAsset>,
}

impl WalletMiner {
    pub fn new() -> Self {
        let (public_key, secret_key) = crypto::gen_keypair();
        WalletMiner {
            public_key,
            secret_key,
            assets: Vec::new(),
        }
    }

    pub fn add_asset(self, name: &str, count: u64, fees: Fees) -> Self {
        let asset = MetaAsset::new(&self.public_key, name, count, fees);
        self.add_asset_value(asset)
    }

    pub fn add_asset_value(mut self, asset: MetaAsset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn mine_empty(self, _testkit: &mut TestKit) -> (PublicKey, SecretKey) {
        (self.public_key, self.secret_key)
    }

    pub fn mine(self, testkit: &mut TestKit) -> (PublicKey, SecretKey) {
        let mine_1_dmc = transaction::Builder::new()
            .keypair(self.public_key, self.secret_key.clone())
            .tx_mine()
            .build();

        post_tx(&testkit.api(), &mine_1_dmc);
        testkit.create_block();

        if !self.assets.is_empty() {
            let mut tx_add_assets_builder = transaction::Builder::new()
            .keypair(self.public_key, self.secret_key.clone())
            .tx_add_assets()
            .seed(85);

            for asset in self.assets {
                tx_add_assets_builder = tx_add_assets_builder.add_asset_value(asset);
            }

            let tx_add_assets = tx_add_assets_builder.build();

            post_tx(&testkit.api(), &tx_add_assets);
            testkit.create_block();
        }

        (self.public_key, self.secret_key)
    }   
}

pub fn asset_fee(t: u64, r: u64) -> Fees {
    fee::Builder::new()
        .trade(t, r)
        .exchange(t, r)
        .transfer(t, r)
        .build()
}

fn genesis_wallet(api: &TestKitApi) -> Wallet {
    let response: WalletResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/wallets/{}", Service::genesis_wallet().to_string()),
    );

    response.unwrap()
}

#[test]
fn wallet() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let tax = 10;
    let units = 2;
    let meta_data = "asset";

    let (pub_key, _) = WalletMiner::new()
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

    let (pub_key1, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (pub_key2, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let genesis_key = Service::genesis_wallet();
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

    let (_, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let (_, _) = WalletMiner::new()
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

    let (pub_key, _) = WalletMiner::new()
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

    let (pub_key, _) = WalletMiner::new()
        .add_asset(meta_data, units, asset_fee(tax, 0))
        .mine(&mut testkit);

    let response: WalletAssetsResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/wallets/{}/assets?{}=true", pub_key.to_string(), wallet::PARAMETER_META_DATA_KEY),
    );

    let asset = AssetBundle::from_data(meta_data, units, &pub_key);
    let info = AssetInfo::new(&pub_key, units, asset_fee(tax, 0));

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