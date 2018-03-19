extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::crypto;
use exonum::messages::Message;

use dmbc::currency::assets::{AssetBundle, MetaAsset, AssetId};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::error::Error;

use transactions::*;

#[test]
fn delete_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    set_configuration(&mut testkit, TransactionFees::new(0, 0, 100, 0, 0, 0));

    let (public_key, secret_key) = crypto::gen_keypair();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());
    let asset = meta_asset.to_bundle(asset_id.clone());
    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

}
