extern crate dmbc;
extern crate exonum;

pub mod utils;

use exonum::blockchain::Transaction;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;

use dmbc::currency::assets::{AssetBundle, AssetId};
use dmbc::currency::transactions::builders::transaction;

#[test]
fn capi_delete_assets() {
    let contents = utils::run("delete_assets");
    let inputs = utils::read_inputs("delete_assets").unwrap();

    let public = PublicKey::from_hex(inputs["public_key"].as_str().unwrap());

    let mut builder = transaction::Builder::new()
        .keypair(public.unwrap(), SecretKey::zero())
        .tx_del_assets()
        .seed(inputs["seed"].as_u64().unwrap());
    for asset in inputs["assets"].as_array().unwrap() {
        let id = AssetId::from_hex(asset["id"].as_str().unwrap());

        let amount = asset["amount"].as_u64().unwrap();
        let bundle = AssetBundle::new(id.unwrap(), amount);

        builder.add_asset_value_ref(bundle);
    }

    let tx = builder.build();

    let tx: Box<Transaction> = tx.into();
    let hex = utils::hex_string(tx.raw().body().to_vec());

    assert_eq!(contents, hex);
}
