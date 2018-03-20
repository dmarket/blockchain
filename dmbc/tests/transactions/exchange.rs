extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::crypto;
use exonum::messages::Message;

use dmbc::currency::assets::{AssetBundle, MetaAsset, AssetId, AssetInfo, Fees};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::error::Error;

use transactions::*;

fn fees() -> Fees {
    fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build()
}

#[test]
fn exchange_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 1000, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);

    let tx_add_assets = transaction::Builder::new()
        .keypair(sender_pk, sender_sk.clone())
        .tx_add_assets()
        .add_asset("asset1", 10, fees())
        .add_asset("asset2", 10, fees())
        .add_asset("asset3", 10, fees())
        .add_asset("asset5", 10, fees())
        .add_asset("asset6", 10, fees())
        .add_asset_receiver(recipient_pk, "asset3", 5, fees())
        .add_asset_receiver(recipient_pk, "asset4", 5, fees())
        .add_asset_receiver(recipient_pk, "asset5", 5, fees())
        .add_asset_receiver(recipient_pk, "asset6", 5, fees())
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(
        vec![
            AssetBundle::from_data("asset1", 10, &sender_pk),
            AssetBundle::from_data("asset2", 10, &sender_pk),
            AssetBundle::from_data("asset3", 10, &sender_pk),
            AssetBundle::from_data("asset5", 10, &sender_pk),
            AssetBundle::from_data("asset6", 10, &sender_pk),
        ],
        sender_wallet.assets()
    );
    assert_eq!(DMC_1, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(
        vec![
            AssetBundle::from_data("asset3", 5, &sender_pk),
            AssetBundle::from_data("asset4", 5, &sender_pk),
            AssetBundle::from_data("asset5", 5, &sender_pk),
            AssetBundle::from_data("asset6", 5, &sender_pk),
        ],
        recipient_wallet.assets()
    );
    assert_eq!(DMC_1, recipient_wallet.balance());

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk.clone())
        .fee_strategy(1)
        .sender_add_asset_value(AssetBundle::from_data("asset1", 6, &sender_pk))
        .sender_add_asset_value(AssetBundle::from_data("asset2", 10, &sender_pk))
        .sender_add_asset_value(AssetBundle::from_data("asset3", 5, &sender_pk))
        .sender_add_asset_value(AssetBundle::from_data("asset6", 3, &sender_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset4", 2, &sender_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset5", 5, &sender_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset6", 4, &sender_pk))
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_exchange_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(
        vec![
            AssetBundle::from_data("asset1", 4, &sender_pk),
            AssetBundle::from_data("asset3", 5, &sender_pk),
            AssetBundle::from_data("asset5", 15, &sender_pk),
            AssetBundle::from_data("asset6", 11, &sender_pk),
            AssetBundle::from_data("asset4", 2, &sender_pk),
        ],
        sender_wallet.assets()
    );

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(
        vec![
            AssetBundle::from_data("asset3", 10, &sender_pk),
            AssetBundle::from_data("asset4", 3, &sender_pk),
            AssetBundle::from_data("asset6", 4, &sender_pk),
            AssetBundle::from_data("asset1", 6, &sender_pk),
            AssetBundle::from_data("asset2", 10, &sender_pk),
        ],
        recipient_wallet.assets()
    );
}
