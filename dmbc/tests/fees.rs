extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate serde_json;

use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};
use exonum::encoding::serialize::reexport::Serialize;
use exonum::messages::Message;

use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::Service;
use dmbc::currency::SERVICE_NAME;
use dmbc::currency::api::transaction::{TxPostResponse, TransactionResponse};
use dmbc::currency::api::fees::{FeesResponseBody, FeesResponse};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::transactions::components::FeesTable;
use dmbc::currency::assets::{Fees, MetaAsset, AssetBundle};
use dmbc::currency::error::Error;

pub fn init_testkit() -> TestKit {
    TestKitBuilder::validator()
        .with_validators(4)
        .with_service(Service::new())
        .create()
}

pub fn set_configuration(testkit: &mut TestKit, fees: TransactionFees) {
    let configuration = Configuration::new(fees);
    let cfg_change_height = testkit.height().next();
    let proposal = {
        let mut cfg = testkit.configuration_change_proposal();
        cfg.set_service_config(&SERVICE_NAME, configuration.clone());
        cfg.set_actual_from(cfg_change_height);
        cfg
    };
    testkit.commit_configuration_change(proposal);
    testkit.create_block();
}

pub fn post_request<T>(api: &TestKitApi, tx: &T) -> FeesResponse
    where T:Message + Serialize
{
    let response: FeesResponse = api.post(
        ApiKind::Service(SERVICE_NAME),
        "/v1/fees/transactions",
        &tx
    );

    response
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
    fn new() -> Self {
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

    pub fn mine_empty(self) -> (PublicKey, SecretKey) {
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

#[test]
fn fees_for_add_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 100;
    let per_asset_fee = 4;
    let amount = 5;
    set_configuration(&mut testkit, TransactionFees::new(transaction_fee, per_asset_fee, 0, 0, 0, 0));

    let (public_key, secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = "asset";
    let meta_asset = MetaAsset::new(&receiver_key, meta_data, amount, fees.clone());

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    let response = post_request(&api, &tx_add_assets);
    let mut expected = FeesTable::new();
    expected.insert(public_key, transaction_fee + amount * per_asset_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_delete_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, transaction_fee, 0, 0, 0));

    let meta_data = "asset";
    let (public_key, secret_key) = crypto::gen_keypair();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, 5)
        .seed(85)
        .build();

    let response = post_request(&api, &tx_delete_assets);
    let mut expected = FeesTable::new();
    expected.insert(public_key, transaction_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

fn transefer_fee(t: u64) -> Fees {
    fee::Builder::new()
        .trade(0, 0)
        .exchange(0, 0)
        .transfer(t, 0)
        .build()
}

#[test]
fn fees_for_transfer() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let amount = 2;
    let tax = 10;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, 0, transaction_fee));

    let meta_data = "asset";
    let (public_key, secret_key) = WalletMiner::new()
        .add_asset(meta_data, amount, transefer_fee(tax))
        .mine(&mut testkit);

    let (recipient_key, _) = crypto::gen_keypair();
        
    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer()
        .add_asset(meta_data, amount)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let response = post_request(&api, &tx_transfer);
    let mut expected = FeesTable::new();
    expected.insert(public_key, transaction_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);

    // sender is not asset creator
    let asset = AssetBundle::from_data(meta_data, amount, &public_key);
    let (sender_pub_key, sender_sec_key) = crypto::gen_keypair();

    let tx_transfer = transaction::Builder::new()
        .keypair(sender_pub_key, sender_sec_key)
        .tx_transfer()
        .add_asset_value(asset)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let response = post_request(&api, &tx_transfer);
    let mut expected = FeesTable::new();
    let expected_fee = transaction_fee + amount * tax;
    expected.insert(sender_pub_key, expected_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}

#[test]
fn fees_for_transfer_asset_not_found() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    let amount = 2;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, 0, transaction_fee));

    let meta_data = "asset";
    let (public_key, secret_key) = WalletMiner::new()
        .mine_empty();

    let (recipient_key, _) = crypto::gen_keypair();
        
    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer()
        .add_asset(meta_data, amount)
        .recipient(recipient_key)
        .seed(42)
        .build();

    let response = post_request(&api, &tx_transfer);
    assert_eq!(Ok(Err(Error::AssetNotFound)), response);
}