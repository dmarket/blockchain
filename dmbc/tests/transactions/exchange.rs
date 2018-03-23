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
use dmbc::currency::error::Error;

use transactions::*;

fn exchange_fee(t: u64) -> Fees {
    fee::Builder::new()
        .trade(0, 0)
        .exchange(t, 0)
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
        .add_asset_receiver(sender_pk, "asset1", 10, exchange_fee(1))
        .add_asset_receiver(sender_pk, "asset2", 10, exchange_fee(2))
        .add_asset_receiver(sender_pk, "asset3", 10, exchange_fee(3))
        .add_asset_receiver(sender_pk, "asset5", 10, exchange_fee(5))
        .add_asset_receiver(sender_pk, "asset6", 10, exchange_fee(6))
        .add_asset_receiver(recipient_pk, "asset3", 5, exchange_fee(3))
        .add_asset_receiver(recipient_pk, "asset4", 5, exchange_fee(4))
        .add_asset_receiver(recipient_pk, "asset5", 5, exchange_fee(5))
        .add_asset_receiver(recipient_pk, "asset6", 5, exchange_fee(6))
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
    let assets_fee = 6*1 + 10*2 + 5*3 + 2*4 + 5*5 + 6*7;

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
    assert_eq!(DMC_1 - 1000 - assets_fee, recipient_wallet.balance());
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
        .add_asset_receiver(sender_pk, "asset1", 1, exchange_fee(10))
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

#[test]
fn exchange_asset_insufficient_funds_fee_asset_very_big() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 1000, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let (creator_pk, creator_sk) = crypto::gen_keypair();
    let sender_balance = DMC_1;
    let mut recipient_balance = DMC_1;
    let creator_balance = 0u64;
    let mut genesis_balance = 137_000_000_00000000u64;

    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_pk, creator_sk.clone())
        .tx_add_assets()
        .add_asset_receiver(sender_pk, "asset1", 1, exchange_fee(DMC_1))
        .add_asset_receiver(sender_pk, "asset2", 10, exchange_fee(100))
        .build();
    let asset1 = AssetBundle::from_data("asset1", 1, &creator_pk);
    let asset2 = AssetBundle::from_data("asset2", 10, &creator_pk);

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let e:Vec<AssetBundle> = Vec::new();
    let a = vec![asset1.clone(), asset2.clone()];

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
        .sender_add_asset_value(asset1.clone())
        .sender_add_asset_value(asset2.clone())
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    recipient_balance -= 1000;
    genesis_balance += 1000;

    let s = get_status(&api, &tx_exchange_assets.hash());
    assert_eq!(Ok(Err(Error::InsufficientFunds)), s);

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
}

#[test]
fn exchange_asset_insufficient_funds_bc_fee_big() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, DMC_1 + 1, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let (creator_pk, creator_sk) = crypto::gen_keypair();
    let sender_balance = DMC_1;
    let recipient_balance = DMC_1;
    let creator_balance = 0u64;
    let genesis_balance = 137_000_000_00000000u64;

    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_pk, creator_sk.clone())
        .tx_add_assets()
        .add_asset_receiver(sender_pk, "asset1", 1, exchange_fee(0))
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

    let s = get_status(&api, &tx_exchange_assets.hash());
    assert_eq!(Ok(Err(Error::InsufficientFunds)), s);

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
}

#[test]
fn exchange_asset_insufficient_fee_strategy_recip_and_send() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 1000, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let (creator_pk, creator_sk) = crypto::gen_keypair();
    let mut sender_balance = DMC_1;
    let mut recipient_balance = DMC_1;
    let creator_balance = 0u64;
    let mut genesis_balance = 137_000_000_00000000u64;

    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_pk, creator_sk.clone())
        .tx_add_assets()
        .add_asset_receiver(sender_pk, "asset1", 2, exchange_fee(DMC_1))
        .build();
    let asset = AssetBundle::from_data("asset1", 2, &creator_pk);

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
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(asset.clone())
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();
    sender_balance -= 1000 / 2;
    recipient_balance -= 1000 / 2;
    genesis_balance += 1000;

    let s = get_status(&api, &tx_exchange_assets.hash());
    assert_eq!(Ok(Err(Error::InsufficientFunds)), s);

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
}

#[test]
fn exchange_asset_send_value() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 1000, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let mut sender_balance = DMC_1;
    let mut recipient_balance = DMC_1;
    let mut genesis_balance = 137_000_000_00000000u64;

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk.clone())
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_value(1000)
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    sender_balance = sender_balance - 1000 - (1000 / 2);
    recipient_balance = recipient_balance + 1000 - (1000 / 2);
    genesis_balance += 1000;

    let status = get_status(&api, &tx_exchange_assets.hash());
    let sender_wallet = get_wallet(&api, &sender_pk);
    let recipient_wallet = get_wallet(&api, &recipient_pk);
    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());

    assert_eq!(Ok(Ok(())), status);
    assert_eq!(sender_balance, sender_wallet.balance());
    assert_eq!(recipient_balance, recipient_wallet.balance());
    assert_eq!(genesis_balance, genesis_wallet.balance());


    let tx_exchange_assets = transaction::Builder::new()
        .keypair(sender_pk, sender_sk.clone())
        .tx_exchange()
        .sender(recipient_pk)
        .sender_secret(recipient_sk.clone())
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_value(DMC_1)
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    sender_balance = sender_balance + DMC_1 - (1000 / 2);           //  199998000
    recipient_balance = recipient_balance - DMC_1 - (1000 / 2);     //  0
    genesis_balance += 1000;

    let status = get_status(&api, &tx_exchange_assets.hash());
    let sender_wallet = get_wallet(&api, &sender_pk);
    let recipient_wallet = get_wallet(&api, &recipient_pk);
    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());

    assert_eq!(Ok(Ok(())), status);
    assert_eq!(sender_balance, sender_wallet.balance());
    assert_eq!(recipient_balance, recipient_wallet.balance());
    assert_eq!(genesis_balance, genesis_wallet.balance());

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk.clone())
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(123)
        .sender_value(1000)
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    let status = get_status(&api, &tx_exchange_assets.hash());
    let sender_wallet = get_wallet(&api, &sender_pk);
    let recipient_wallet = get_wallet(&api, &recipient_pk);
    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());

    assert_eq!(Ok(Err(Error::InsufficientFunds)), status);
    assert_eq!(sender_balance, sender_wallet.balance());
    assert_eq!(recipient_balance, recipient_wallet.balance());
    assert_eq!(genesis_balance, genesis_wallet.balance());

}