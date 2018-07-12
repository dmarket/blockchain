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
fn capi_trade_intermediary() {
    let contents = utils::run("trade_intermediary");
    let inputs = utils::read_inputs("trade_intermediary").unwrap();

    let offer = inputs["offer"].as_object().unwrap();
    let seller = PublicKey::from_hex(offer["seller"].as_str().unwrap());
    let buyer = PublicKey::from_hex(offer["buyer"].as_str().unwrap());
    let fee_strategy = FeeStrategy::try_from(offer["fee_strategy"].as_u64().unwrap() as u8);
    let seed = inputs["seed"].as_u64().unwrap();
    let memo = inputs["memo"].as_str().unwrap();

    let intermediary = offer["intermediary"].as_object().unwrap();
    let intermediary_key = PublicKey::from_hex(intermediary["wallet"].as_str().unwrap());
    let intermediary_commission = intermediary["commission"].as_u64().unwrap();

    let mut builder = transaction::Builder::new()
        .keypair(buyer.unwrap(), SecretKey::zero())
        .tx_trade_assets_with_intermediary()
        .seller(seller.unwrap(), SecretKey::zero())
        .intermediary_key_pair(intermediary_key.unwrap(), SecretKey::zero())
        .commission(intermediary_commission)
        .fee_strategy(fee_strategy.unwrap())
        .data_info(memo)
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