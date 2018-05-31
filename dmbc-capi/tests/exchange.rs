extern crate dmbc;
extern crate exonum;

pub mod utils;

use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;
use exonum::blockchain::Transaction;

use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::{AssetId, AssetBundle};
use dmbc::currency::transactions::components::FeeStrategy;

#[test]
fn capi_exchange() {
    let contents = utils::run("exchange");
    let inputs = utils::read_inputs("exchange").unwrap();

    let offer = inputs["offer"].as_object().unwrap();
    let sender = PublicKey::from_hex(offer["sender"].as_str().unwrap());
    let recipient = PublicKey::from_hex(offer["recipient"].as_str().unwrap());
    let fee_strategy = FeeStrategy::try_from(offer["fee_strategy"].as_u64().unwrap() as u8);
    let memo = inputs["memo"].as_str().unwrap();
    let sender_value = offer["sender_value"].as_u64().unwrap();
    let seed = inputs["seed"].as_u64().unwrap();

    let mut builder = transaction::Builder::new()
        .keypair(recipient.unwrap(), SecretKey::zero())
        .tx_exchange()
        .sender(sender.unwrap())
        .sender_secret(SecretKey::zero())
        .sender_value(sender_value)
        .fee_strategy(fee_strategy.unwrap())
        .seed(seed)
        .data_info(memo);

    for asset in offer["recipient_assets"].as_array().unwrap() {        
        let id = AssetId::from_hex(asset["id"].as_str().unwrap());

        let amount = asset["amount"].as_u64().unwrap();
        let bundle = AssetBundle::new(id.unwrap(), amount);

        builder.recipient_add_asset_value_ref(bundle);
    }

    for asset in offer["sender_assets"].as_array().unwrap() {
        let id = AssetId::from_hex(asset["id"].as_str().unwrap());

        let amount = asset["amount"].as_u64().unwrap();
        let bundle = AssetBundle::new(id.unwrap(), amount);

        builder.sender_add_asset_value_ref(bundle);
    }

    let tx = builder.build();

    let tx: Box<Transaction> = tx.into();
    let hex = utils::hex_string(tx.raw().body().to_vec());

    assert_eq!(contents, hex);
}