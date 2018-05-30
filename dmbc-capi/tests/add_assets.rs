extern crate dmbc;
extern crate exonum;

pub mod utils;

use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;
use exonum::blockchain::Transaction;

use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::transactions::builders::fee;

pub fn hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    strs.join("")
}

#[test]
fn capi_add_assets() {
    let contents = utils::run("add_assets");
    let inputs = utils::read_inputs("add_assets").unwrap();

    let public = PublicKey::from_hex(inputs["public_key"].as_str().unwrap());

    let mut builder = transaction::Builder::new()
        .keypair(public.unwrap(), SecretKey::zero())
        .tx_add_assets()
        .seed(inputs["seed"].as_u64().unwrap());
    for asset in inputs["assets"].as_array().unwrap() {
        let fees_json = asset["fees"].as_object().unwrap();
        let trade = fees_json["trade"].as_object().unwrap();
        let exchange = fees_json["exchange"].as_object().unwrap();
        let transfer = fees_json["transfer"].as_object().unwrap();

        let fees = fee::Builder::new()
            .trade(trade["fixed"].as_u64().unwrap(), trade["fraction"].as_str().unwrap().parse().unwrap())
            .exchange(exchange["fixed"].as_u64().unwrap(), exchange["fraction"].as_str().unwrap().parse().unwrap())
            .transfer(transfer["fixed"].as_u64().unwrap(), transfer["fraction"].as_str().unwrap().parse().unwrap())
            .build();

        let receiver = PublicKey::from_hex(asset["receiver"].as_str().unwrap());
        builder.add_asset_receiver_ref(
            receiver.unwrap(),
            asset["data"].as_str().unwrap(), 
            asset["amount"].as_u64().unwrap(), 
            fees
        );
    }

    let tx = builder.build();

    let tx: Box<Transaction> = tx.into();
    let hex = hex_string(tx.raw().body().to_vec());

    assert_eq!(contents, hex);
}
