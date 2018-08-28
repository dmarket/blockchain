extern crate dmbc;
extern crate exonum;

pub mod utils;

use exonum::blockchain::Transaction;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;

use dmbc::currency::assets::{AssetBundle, AssetId};
use dmbc::currency::transactions::builders::transaction;

#[test]
fn capi_transfer_fees_payer() {
    let contents = utils::run("transfer_fees_payer");
    let inputs = utils::read_inputs("transfer_fees_payer").unwrap();

    let offer = inputs["offer"].as_object().unwrap();
    let from = PublicKey::from_hex(offer["from"].as_str().unwrap());
    let to = PublicKey::from_hex(offer["to"].as_str().unwrap());
    let fees_payer = PublicKey::from_hex(offer["fees_payer"].as_str().unwrap());

    let mut builder = transaction::Builder::new()
        .keypair(from.unwrap(), SecretKey::zero())
        .tx_transfer_with_fees_payer()
        .recipient(to.unwrap())
        .fees_payer(fees_payer.unwrap(), SecretKey::zero())
        .amount(offer["amount"].as_u64().unwrap())
        .seed(offer["seed"].as_u64().unwrap())
        .data_info(offer["memo"].as_str().unwrap());

    for asset in offer["assets"].as_array().unwrap() {
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
