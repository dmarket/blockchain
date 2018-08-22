extern crate dmbc;
extern crate exonum;

pub mod utils;

use exonum::blockchain::Transaction;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;

use dmbc::currency::assets::{AssetId, TradeAsset};
use dmbc::currency::transactions::builders::transaction;

#[test]
fn capi_bid_offer() {
    let contents = utils::run("bid_offer");
    let inputs = utils::read_inputs("bid_offer").unwrap();

    let public = PublicKey::from_hex(inputs["pub_key"].as_str().unwrap());

    let asset = inputs["asset"].as_object().unwrap();
    let id = AssetId::from_hex(asset["id"].as_str().unwrap());
    let amount = asset["amount"].as_u64().unwrap();
    let price = asset["price"].as_u64().unwrap();
    let trade_asset = TradeAsset::new(id.unwrap(), amount, price);

    let builder = transaction::Builder::new()
        .keypair(public.unwrap(), SecretKey::zero())
        .tx_offer()
        .asset(trade_asset)
        .seed(inputs["seed"].as_u64().unwrap())
        .data_info(inputs["data_info"].as_str().unwrap());

    let tx = builder.bid_build();

    let tx: Box<Transaction> = tx.into();
    let hex = utils::hex_string(tx.raw().body().to_vec());

    assert_eq!(contents, hex);
}
