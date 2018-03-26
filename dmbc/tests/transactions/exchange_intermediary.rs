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

const BC_FEE:u64 = 1000;
const INTER_COMMISSION:u64 = 100;

#[test]
fn exchange_i_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, BC_FEE, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let (creator_pk, creator_sk) = crypto::gen_keypair();
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();
    let mut sender_balance = DMC_1;
    let mut recipient_balance = DMC_1;
    let mut creator_balance = 0u64;
    let mut intermediary_balance = 0u64;
    let mut genesis_balance = 137_000_000_00000000u64;

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
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk.clone())
        .commission(INTER_COMMISSION)
        .sender_key_pair(sender_pk, sender_sk.clone())
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(AssetBundle::from_data("asset1", 6, &creator_pk))
        .sender_add_asset_value(AssetBundle::from_data("asset2", 10, &creator_pk))
        .sender_add_asset_value(AssetBundle::from_data("asset3", 5, &creator_pk))
        .sender_add_asset_value(AssetBundle::from_data("asset6", 3, &creator_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset4", 2, &creator_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset5", 5, &creator_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset6", 4, &creator_pk))
        .build();

    let assets_fee = 6*1 + 10*2 + 5*3 + 2*4 + 5*5 + 6*7;

    sender_balance = sender_balance - (BC_FEE + INTER_COMMISSION + assets_fee) / 2;
    recipient_balance = recipient_balance - (BC_FEE + INTER_COMMISSION + assets_fee) / 2;
    intermediary_balance += INTER_COMMISSION;
    creator_balance += assets_fee;
    genesis_balance += BC_FEE;

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();


    let status = get_status(&api, &tx_exchange_assets.hash());
    let sender_wallet = get_wallet(&api, &sender_pk);
    let recipient_wallet = get_wallet(&api, &recipient_pk);
    let creator_wallet = get_wallet(&api, &creator_pk);
    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());
    let intermediary_wallet = get_wallet(&api, &intermediary_pk);

    assert_eq!(Ok(Ok(())), status);
    assert_eq!(sender_balance, sender_wallet.balance());
    assert_eq!(recipient_balance, recipient_wallet.balance());
    assert_eq!(creator_balance, creator_wallet.balance());
    assert_eq!(genesis_balance, genesis_wallet.balance());
    assert_eq!(intermediary_balance, intermediary_wallet.balance());

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
}

#[test]
fn exchange_i_assets_creator_is_sender() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, BC_FEE, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();
    let mut sender_balance = DMC_1;
    let mut recipient_balance = DMC_1;
    let mut intermediary_balance = 0u64;
    let mut genesis_balance = 137_000_000_00000000u64;

    let tx_add_assets = transaction::Builder::new()
        .keypair(sender_pk, sender_sk.clone())
        .tx_add_assets()
        .add_asset_receiver(sender_pk, "asset1", 10, exchange_fee(1))
        .add_asset_receiver(recipient_pk, "asset6", 5, exchange_fee(6))
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(vec![AssetBundle::from_data("asset1", 10, &sender_pk),], sender_wallet.assets());
    assert_eq!(DMC_1, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(vec![AssetBundle::from_data("asset6", 5, &sender_pk), ], recipient_wallet.assets());
    assert_eq!(DMC_1, recipient_wallet.balance());

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk.clone())
        .commission(INTER_COMMISSION)
        .sender_key_pair(sender_pk, sender_sk.clone())
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(AssetBundle::from_data("asset1", 6, &sender_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset6", 4, &sender_pk))
        .build();

    let assets_fee = 6*1 + 6*4;

    sender_balance = sender_balance - (BC_FEE + INTER_COMMISSION + assets_fee) / 2 + assets_fee;
    recipient_balance = recipient_balance - (BC_FEE + INTER_COMMISSION + assets_fee) / 2;
    intermediary_balance += INTER_COMMISSION;
    genesis_balance += BC_FEE;

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();


    let status = get_status(&api, &tx_exchange_assets.hash());
    let sender_wallet = get_wallet(&api, &sender_pk);
    let recipient_wallet = get_wallet(&api, &recipient_pk);
    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());
    let intermediary_wallet = get_wallet(&api, &intermediary_pk);

    assert_eq!(Ok(Ok(())), status);
    assert_eq!(sender_balance, sender_wallet.balance());
    assert_eq!(recipient_balance, recipient_wallet.balance());
    assert_eq!(genesis_balance, genesis_wallet.balance());
    assert_eq!(intermediary_balance, intermediary_wallet.balance());

    assert_eq!(
        vec![
            AssetBundle::from_data("asset1", 4, &sender_pk),
            AssetBundle::from_data("asset6", 4, &sender_pk),
        ],
        sender_wallet.assets()
    );

    assert_eq!(
        vec![
            AssetBundle::from_data("asset6", 1, &sender_pk),
            AssetBundle::from_data("asset1", 6, &sender_pk),
        ],
        recipient_wallet.assets()
    );
}

#[test]
fn exchange_i_assets_creator_is_intermediary() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, BC_FEE, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();
    let mut sender_balance = DMC_1;
    let mut recipient_balance = DMC_1;
    let mut intermediary_balance = 0u64;
    let mut genesis_balance = 137_000_000_00000000u64;

    let tx_add_assets = transaction::Builder::new()
        .keypair(intermediary_pk, intermediary_sk.clone())
        .tx_add_assets()
        .add_asset_receiver(sender_pk, "asset1", 10, exchange_fee(1))
        .add_asset_receiver(recipient_pk, "asset6", 5, exchange_fee(6))
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(vec![AssetBundle::from_data("asset1", 10, &intermediary_pk), ], sender_wallet.assets());
    assert_eq!(DMC_1, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(vec![AssetBundle::from_data("asset6", 5, &intermediary_pk), ], recipient_wallet.assets());
    assert_eq!(DMC_1, recipient_wallet.balance());

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk.clone())
        .commission(INTER_COMMISSION)
        .sender_key_pair(sender_pk, sender_sk.clone())
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(AssetBundle::from_data("asset1", 6, &intermediary_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset6", 4, &intermediary_pk))
        .build();

    let assets_fee = 6*1 + 6*4;

    sender_balance = sender_balance - (BC_FEE + INTER_COMMISSION + assets_fee) / 2 ;
    recipient_balance = recipient_balance - (BC_FEE + INTER_COMMISSION + assets_fee) / 2;
    intermediary_balance += INTER_COMMISSION + assets_fee;
    genesis_balance += BC_FEE;

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();


    let status = get_status(&api, &tx_exchange_assets.hash());
    let sender_wallet = get_wallet(&api, &sender_pk);
    let recipient_wallet = get_wallet(&api, &recipient_pk);
    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());
    let intermediary_wallet = get_wallet(&api, &intermediary_pk);

    assert_eq!(Ok(Ok(())), status);
    assert_eq!(sender_balance, sender_wallet.balance());
    assert_eq!(recipient_balance, recipient_wallet.balance());
    assert_eq!(genesis_balance, genesis_wallet.balance());
    assert_eq!(intermediary_balance, intermediary_wallet.balance());

    assert_eq!(
        vec![
            AssetBundle::from_data("asset1", 4, &intermediary_pk),
            AssetBundle::from_data("asset6", 4, &intermediary_pk),
        ],
        sender_wallet.assets()
    );

    assert_eq!(
        vec![
            AssetBundle::from_data("asset6", 1, &intermediary_pk),
            AssetBundle::from_data("asset1", 6, &intermediary_pk),
        ],
        recipient_wallet.assets()
    );
}

#[test]
fn exchange_i_assets_payer_fee_intermediary() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, BC_FEE, 0, 0));

    let (sender_pk, sender_sk) = mine_wallet(&mut testkit);
    let (recipient_pk, recipient_sk) = mine_wallet(&mut testkit);
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();
    let sender_balance = DMC_1;
    let recipient_balance = DMC_1;
    let mut intermediary_balance = 0u64;
    let mut genesis_balance = 137_000_000_00000000u64;

    let tx_add_assets = transaction::Builder::new()
        .keypair(intermediary_pk, intermediary_sk.clone())
        .tx_add_assets()
        .add_asset_receiver(sender_pk, "asset1", 10, exchange_fee(1))
        .add_asset_receiver(recipient_pk, "asset6", 5, exchange_fee(6))
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let sender_wallet = get_wallet(&api, &sender_pk);
    assert_eq!(vec![AssetBundle::from_data("asset1", 10, &intermediary_pk), ], sender_wallet.assets());
    assert_eq!(DMC_1, sender_wallet.balance());

    let recipient_wallet = get_wallet(&api, &recipient_pk);
    assert_eq!(vec![AssetBundle::from_data("asset6", 5, &intermediary_pk), ], recipient_wallet.assets());
    assert_eq!(DMC_1, recipient_wallet.balance());

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk.clone())
        .commission(INTER_COMMISSION)
        .sender_key_pair(sender_pk, sender_sk.clone())
        .fee_strategy(FeeStrategy::Intermediary)
        .sender_add_asset_value(AssetBundle::from_data("asset1", 6, &intermediary_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset6", 4, &intermediary_pk))
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();

    let status = get_status(&api, &tx_exchange_assets.hash());
    let sender_wallet = get_wallet(&api, &sender_pk);
    let recipient_wallet = get_wallet(&api, &recipient_pk);
    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());
    let intermediary_wallet = get_wallet(&api, &intermediary_pk);

    assert_eq!(Ok(Err(Error::InsufficientFunds)), status);
    assert_eq!(sender_balance, sender_wallet.balance());
    assert_eq!(recipient_balance, recipient_wallet.balance());
    assert_eq!(genesis_balance, genesis_wallet.balance());
    assert_eq!(intermediary_balance, intermediary_wallet.balance());
    assert_eq!(vec![AssetBundle::from_data("asset1", 10, &intermediary_pk), ], sender_wallet.assets());
    assert_eq!(vec![AssetBundle::from_data("asset6", 5, &intermediary_pk), ], recipient_wallet.assets());

    let mine_1_dmc = transaction::Builder::new()
        .keypair(intermediary_pk, intermediary_sk.clone())
        .tx_mine()
        .build();

    post_tx(&testkit.api(), &mine_1_dmc);
    testkit.create_block();
    intermediary_balance = DMC_1;

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk.clone())
        .commission(INTER_COMMISSION)
        .sender_key_pair(sender_pk, sender_sk.clone())
        .fee_strategy(FeeStrategy::Intermediary)
        .sender_add_asset_value(AssetBundle::from_data("asset1", 6, &intermediary_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset6", 4, &intermediary_pk))
        .seed(123)
        .build();

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();
    intermediary_balance -= BC_FEE;
    genesis_balance += BC_FEE;

    let status = get_status(&api, &tx_exchange_assets.hash());
    let sender_wallet = get_wallet(&api, &sender_pk);
    let recipient_wallet = get_wallet(&api, &recipient_pk);
    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());
    let intermediary_wallet = get_wallet(&api, &intermediary_pk);

    assert_eq!(Ok(Ok(())), status);
    assert_eq!(sender_balance, sender_wallet.balance());
    assert_eq!(recipient_balance, recipient_wallet.balance());
    assert_eq!(genesis_balance, genesis_wallet.balance());
    assert_eq!(intermediary_balance, intermediary_wallet.balance());
    assert_eq!(
        vec![
            AssetBundle::from_data("asset1", 4, &intermediary_pk),
            AssetBundle::from_data("asset6", 4, &intermediary_pk),
        ],
        sender_wallet.assets()
    );

    assert_eq!(
        vec![
            AssetBundle::from_data("asset6", 1, &intermediary_pk),
            AssetBundle::from_data("asset1", 6, &intermediary_pk),
        ],
        recipient_wallet.assets()
    );

    let (creator_pk, creator_sk) = crypto::gen_keypair();
    let mut creator_balance = 0u64;
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_pk, creator_sk.clone())
        .tx_add_assets()
        .add_asset_receiver(sender_pk, "asset1", 10, exchange_fee(1))
        .add_asset_receiver(recipient_pk, "asset6", 5, exchange_fee(6))
        .seed(133)
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk.clone())
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk.clone())
        .commission(INTER_COMMISSION)
        .sender_key_pair(sender_pk, sender_sk.clone())
        .fee_strategy(FeeStrategy::Intermediary)
        .sender_add_asset_value(AssetBundle::from_data("asset1", 6, &creator_pk))
        .recipient_add_asset_value(AssetBundle::from_data("asset6", 4, &creator_pk))
        .seed(333)
        .build();

    let assets_fee = 6*1 + 6*4;

    post_tx(&api, &tx_exchange_assets);
    testkit.create_block();
    intermediary_balance -= BC_FEE + assets_fee;
    genesis_balance += BC_FEE;
    creator_balance += assets_fee;

    let status = get_status(&api, &tx_exchange_assets.hash());
    let sender_wallet = get_wallet(&api, &sender_pk);
    let recipient_wallet = get_wallet(&api, &recipient_pk);
    let genesis_wallet = get_wallet(&api, &Service::genesis_wallet());
    let intermediary_wallet = get_wallet(&api, &intermediary_pk);
    let creator_wallet = get_wallet(&api, &creator_pk);

    assert_eq!(Ok(Ok(())), status);
    assert_eq!(sender_balance, sender_wallet.balance());
    assert_eq!(recipient_balance, recipient_wallet.balance());
    assert_eq!(genesis_balance, genesis_wallet.balance());
    assert_eq!(intermediary_balance, intermediary_wallet.balance());
    assert_eq!(creator_balance, creator_wallet.balance());

    assert_eq!(intermediary_balance, intermediary_wallet.balance());
    assert_eq!(
        vec![
            AssetBundle::from_data("asset1", 4, &intermediary_pk),
            AssetBundle::from_data("asset6", 4, &intermediary_pk),
            AssetBundle::from_data("asset1", 4, &creator_pk),
            AssetBundle::from_data("asset6", 4, &creator_pk),
        ],
        sender_wallet.assets()
    );

    assert_eq!(
        vec![
            AssetBundle::from_data("asset6", 1, &intermediary_pk),
            AssetBundle::from_data("asset1", 6, &intermediary_pk),
            AssetBundle::from_data("asset6", 1, &creator_pk),
            AssetBundle::from_data("asset1", 6, &creator_pk),
        ],
        recipient_wallet.assets()
    );

}