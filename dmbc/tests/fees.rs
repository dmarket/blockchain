extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate serde_json;

use std::collections::HashMap;

use exonum::crypto;
use exonum::crypto::PublicKey;
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};
use exonum::encoding::serialize::reexport::Serialize;
use exonum::messages::Message;

use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::Service;
use dmbc::currency::SERVICE_NAME;
use dmbc::currency::api::fees::{FeesResponseBody, FeesResponse};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::MetaAsset;

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

pub fn post_tx<T>(api: &TestKitApi, tx: &T) -> FeesResponse
    where T:Message + Serialize
{
    let response: FeesResponse = api.post(
        ApiKind::Service(SERVICE_NAME),
        "/v1/fees/transactions",
        &tx
    );

    response
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

    let response = post_tx(&api, &tx_add_assets);
    let mut expected: HashMap<PublicKey, u64> = HashMap::new();
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

    let response = post_tx(&api, &tx_delete_assets);
    let mut expected: HashMap<PublicKey, u64> = HashMap::new();
    expected.insert(public_key, transaction_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}