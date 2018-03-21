extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::crypto;
use exonum::messages::Message;

use dmbc::currency::Service;
use dmbc::currency::assets::{AssetBundle, Fees};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::transactions::components::FeeStrategy;

use transactions::*;

fn fees() -> Fees {
    fee::Builder::new()
        .trade(0, 0)
        .exchange(0, 0)
        .transfer(0, 0)
        .build()
}

#[test]
fn exchange_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 1000, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let (creator_pk, creator_sk) = crypto::gen_keypair();

    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_pk, creator_sk.clone())
        .tx_add_assets()
        .add_asset_receiver(sender_pk, "asset1", 10, fees())
        .add_asset_receiver(sender_pk, "asset2", 10, fees())
        .add_asset_receiver(sender_pk, "asset3", 10, fees())
        .add_asset_receiver(sender_pk, "asset5", 10, fees())
        .add_asset_receiver(sender_pk, "asset6", 10, fees())
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
            AssetBundle::from_data("asset1", 10, &creator_pk),
            AssetBundle::from_data("asset2", 10, &creator_pk),
            AssetBundle::from_data("asset3", 10, &creator_pk),
            AssetBundle::from_data("asset5", 10, &creator_pk),
            AssetBundle::from_data("asset6", 10, &creator_pk),
        ],
        sender_wallet.assets()
    );
    assert_eq!(DMC_1, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(
        vec![
            AssetBundle::from_data("asset3", 5, &creator_pk),
            AssetBundle::from_data("asset4", 5, &creator_pk),
            AssetBundle::from_data("asset5", 5, &creator_pk),
            AssetBundle::from_data("asset6", 5, &creator_pk),
        ],
        recipient_wallet.assets()
    );
    assert_eq!(DMC_1, recipient_wallet.balance());

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk.clone())
        .fee_strategy(FeeStrategy::Recipient)
        .sender_add_asset_value(AssetBundle::from_data("asset1", 6, &creator_pk))
        .sender_add_asset_value(AssetBundle::from_data("asset2", 10, &creator_pk))
        .sender_add_asset_value(AssetBundle::from_data("asset3", 5, &creator_pk))
        .sender_add_asset_value(AssetBundle::from_data("asset6", 3, &creator_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset4", 2, &creator_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset5", 5, &creator_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset6", 4, &creator_pk))
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_exchange_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(
        vec![
            AssetBundle::from_data("asset1", 4, &creator_pk),
            AssetBundle::from_data("asset3", 5, &creator_pk),
            AssetBundle::from_data("asset5", 15, &creator_pk),
            AssetBundle::from_data("asset6", 11, &creator_pk),
            AssetBundle::from_data("asset4", 2, &creator_pk),
        ],
        sender_wallet.assets()
    );
    assert_eq!(DMC_1, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(
        vec![
            AssetBundle::from_data("asset3", 10, &creator_pk),
            AssetBundle::from_data("asset4", 3, &creator_pk),
            AssetBundle::from_data("asset6", 4, &creator_pk),
            AssetBundle::from_data("asset1", 6, &creator_pk),
            AssetBundle::from_data("asset2", 10, &creator_pk),
        ],
        recipient_wallet.assets()
    );
    assert_eq!(DMC_1 - 1000, recipient_wallet.balance());
}

fn exchange_fee(t: u64, r:u64) -> Fees {
    fee::Builder::new()
        .trade(0, 0)
        .exchange(t, r)
        .transfer(0, 0)
        .build()
}

#[test]
fn exchange_asset_fee_strategy() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 1000, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let (creator_pk, creator_sk) = crypto::gen_keypair();
    let mut sender_balance = DMC_1;
    let mut recipient_balance = DMC_1;
    let mut creator_balance = 0u64;
    let mut genesis_balance = 137_000_000_00000000u64;

    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_pk, creator_sk.clone())
        .tx_add_assets()
        .add_asset_receiver(sender_pk, "asset1", 1, exchange_fee(10,1000))
        .build();
    let asset = AssetBundle::from_data("asset1", 1, &creator_pk);

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let e:Vec<AssetBundle> = Vec::new();
    let a = vec![asset.clone()];

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(a, sender_wallet.assets());
    assert_eq!(DMC_1, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(e, recipient_wallet.assets());
    assert_eq!(DMC_1, recipient_wallet.balance());

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk.clone())
        .fee_strategy(FeeStrategy::Recipient)
        .sender_add_asset_value(asset.clone())
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    recipient_balance -= 1000 + 10;
    genesis_balance += 1000;
    creator_balance += 10;


    let s = get_status(&api, &tx_exchange_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(e, sender_wallet.assets());
    assert_eq!(sender_balance, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(a, recipient_wallet.assets());
    assert_eq!(recipient_balance, recipient_wallet.balance());

    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());
    assert_eq!(genesis_balance, genesis_wallet.balance());

    let creator_wallet = get_wallet(&api, &creator_pk);
    assert_eq!(creator_balance, creator_wallet.balance());


    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk.clone())
        .fee_strategy(FeeStrategy::Sender)
        .recipient_add_asset_value(asset.clone())
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    sender_balance -= 1000 + 10;
    genesis_balance += 1000;
    creator_balance += 10;


    let s = get_status(&api, &tx_exchange_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(a, sender_wallet.assets());
    assert_eq!(sender_balance, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(e, recipient_wallet.assets());
    assert_eq!(recipient_balance, recipient_wallet.balance());

    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());
    assert_eq!(genesis_balance, genesis_wallet.balance());

    let creator_wallet = get_wallet(&api, &creator_pk);
    assert_eq!(creator_balance, creator_wallet.balance());

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk.clone())
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(asset.clone())
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    recipient_balance -= (1000 + 10) / 2;
    sender_balance -= (1000 + 10) / 2;
    genesis_balance += 1000;
    creator_balance += 10;


    let s = get_status(&api, &tx_exchange_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(e, sender_wallet.assets());
    assert_eq!(sender_balance, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(a, recipient_wallet.assets());
    assert_eq!(recipient_balance, recipient_wallet.balance());

    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());
    assert_eq!(genesis_balance, genesis_wallet.balance());

    let creator_wallet = get_wallet(&api, &creator_pk);
    assert_eq!(creator_balance, creator_wallet.balance());
}