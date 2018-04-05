use std::collections::HashMap;

use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::configuration::TransactionFees;

use fees::test_api::*;

#[test]
fn fees_for_delete_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 1000;
    set_configuration(&mut testkit, TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0));

    let meta_data = "asset";
    let (public_key, secret_key) = WalletMiner::new().mine_empty(&mut testkit);

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, 5)
        .seed(85)
        .build();

    let response = post_fee(&api, &tx_delete_assets);
    let mut expected = HashMap::new();
    expected.insert(public_key, transaction_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}