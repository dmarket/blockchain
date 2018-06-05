extern crate dmbc;
extern crate exonum;

pub mod utils;

use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;
use exonum::blockchain::Transaction;

use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::{AssetId, AssetBundle};

#[test]
fn capi_transfer() {
    let contents = utils::run("transfer");
    let inputs = utils::read_inputs("transfer").unwrap();

    let from = PublicKey::from_hex(inputs["from"].as_str().unwrap());
    let to = PublicKey::from_hex(inputs["to"].as_str().unwrap());

    let mut builder = transaction::Builder::new()
        .keypair(from.unwrap(), SecretKey::zero())
        .tx_transfer()
        .recipient(to.unwrap())
        .amount(inputs["amount"].as_u64().unwrap())
        .seed(inputs["seed"].as_u64().unwrap())
        .data_info(inputs["memo"].as_str().unwrap());
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