extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::crypto;

use dmbc::currency::transactions::builders::transaction;
use transactions::*;

#[test]
fn mine_wallet() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    let (public_key, secret_key) = crypto::gen_keypair();
    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &tx);

    testkit.create_block();

    let w = get_wallet(&api, &public_key);

    assert_eq!(1_00_000_000, w.balance()); //1 dmc
}
