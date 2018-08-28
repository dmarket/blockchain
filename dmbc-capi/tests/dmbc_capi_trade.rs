extern crate dmbc;
extern crate exonum;

pub mod utils;

use exonum::blockchain::Transaction;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;

use dmbc::currency::assets::{AssetId, TradeAsset};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::transactions::components::FeeStrategy;

#[test]
fn capi_trade() {
    let contents = utils::run("trade");
    let inputs = utils::read_inputs("trade").unwrap();

    let offer = inputs["offer"].as_object().unwrap();
    let seller = PublicKey::from_hex(offer["seller"].as_str().unwrap());
    let buyer = PublicKey::from_hex(offer["buyer"].as_str().unwrap());
    let fee_strategy = FeeStrategy::try_from(offer["fee_strategy"].as_u64().unwrap() as u8);
    let seed = offer["seed"].as_u64().unwrap();
    let data_info = offer["data_info"].as_str().unwrap();

    let mut builder = transaction::Builder::new()
        .keypair(buyer.unwrap(), SecretKey::zero())
        .tx_trade_assets()
        .seller(seller.unwrap(), SecretKey::zero())
        .fee_strategy(fee_strategy.unwrap())
        .data_info(data_info)
        .seed(seed);

    for asset in offer["assets"].as_array().unwrap() {
        let id = AssetId::from_hex(asset["id"].as_str().unwrap());
        let amount = asset["amount"].as_u64().unwrap();
        let price = asset["price"].as_u64().unwrap();
        let trade_asset = TradeAsset::new(id.unwrap(), amount, price);

        builder.add_asset_value_ref(trade_asset);
    }

    let tx = builder.build();

    let tx: Box<Transaction> = tx.into();
    let hex = utils::hex_string(tx.raw().body().to_vec());

    assert_eq!(contents, hex);
}
