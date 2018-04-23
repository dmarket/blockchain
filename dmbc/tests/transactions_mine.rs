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
use exonum::messages::Message;
use exonum::crypto;
use dmbc_testkit::{DmbcTestKit, DmbcTestApiBuilder, DmbcTestKitApi};

use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::api::transaction::TransactionResponse;

#[test]
fn mine_wallet() {
    let (public_key, secret_key) = crypto::gen_keypair();
    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    let mut testkit = DmbcTestApiBuilder::new()
        .create();
    let api = testkit.api();

    let tx_hash = tx.hash();

    let (status, response) = api.post_tx(&tx);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx);
    assert_eq!(tx_status, Ok(Ok(())));

    let wallet = testkit.fetch_wallet(&public_key);
    assert_eq!(wallet.balance(), 100_000_000);
}