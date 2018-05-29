extern crate dmbc;
extern crate exonum;

use std::process::Command;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use exonum::crypto;
use exonum::crypto::PublicKey;
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
    let current_dir = env::current_dir().unwrap();
    let current_dir = current_dir.as_path();
    
    let output = Command::new("./compile.sh")
        .current_dir(current_dir.join("ctest"))
        .output();
    assert!(output.is_ok());

    let output = Command::new("./test")
        .current_dir(current_dir.join("ctest"))
        .arg("add_assets")
        .output();
    assert!(output.is_ok());

    let file = File::open(current_dir.join("ctest").join("output").join("add_assets.txt"));
    assert!(file.is_ok());

    let mut buf_reader = BufReader::new(file.unwrap());
    let mut contents = String::new();
    let res = buf_reader.read_to_string(&mut contents);
    assert!(res.is_ok());

    let (_, secret) = crypto::gen_keypair();
    let fees = fee::Builder::new()
        .trade(10, "0.1".parse().unwrap())
        .exchange(20, "0.2".parse().unwrap())
        .transfer(9, "0.999999".parse().unwrap())
        .build();
    let public = PublicKey::from_hex("4e298e435018ab0a1430b6ebd0a0656be15493966d5ce86ed36416e24c411b9f");
    let tx = transaction::Builder::new()
        .keypair(public.unwrap(), secret)
        .tx_add_assets()
        .add_asset("Asset#10", 10, fees.clone())
        .add_asset("Asset#00", 1000, fees)
        .seed(123)
        .build();

    let tx: Box<Transaction> = tx.into();
    let hex = hex_string(tx.raw().body().to_vec());

    assert_eq!(contents, hex);
}
