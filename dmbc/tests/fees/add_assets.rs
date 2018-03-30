use dmbc::currency::api::fees::FeesResponseBody;
use dmbc::currency::assets::MetaAsset;
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::transactions::components::FeesTable;

use fees::test_api::*;

#[test]
fn fees_for_add_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 100;
    let per_asset_fee = 4;
    let amount = 5;
    set_configuration(&mut testkit, TransactionFees::new(transaction_fee, per_asset_fee, 0, 0, 0, 0));

    let (public_key, secret_key) = WalletMiner::new().mine_empty(&mut testkit);
    let (receiver_key, _) = WalletMiner::new().mine_empty(&mut testkit);

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = "asset";
    let meta_asset = MetaAsset::new(&receiver_key, meta_data, amount, fees);

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(85)
        .build();

    let response = post_fee(&api, &tx_add_assets);
    let mut expected = FeesTable::new();
    expected.insert(public_key, transaction_fee + amount * per_asset_fee);

    assert_eq!(Ok(Ok(FeesResponseBody{fees: expected})), response);
}