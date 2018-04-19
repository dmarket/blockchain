extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate serde_json;
extern crate mount;

pub mod evo_testkit;

use hyper::status::StatusCode;
use exonum::messages::Message;
use exonum::crypto;
use evo_testkit::{EvoTestKit, EvoTestApiBuilder, EvoTestKitApi, asset_fees, create_asset, default_genesis_key};

use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::AssetBundle;
use dmbc::currency::error::Error;
use dmbc::currency::api::transaction::TransactionResponse;
use dmbc::currency::wallet::Wallet;
use dmbc::currency::transactions::components::FeeStrategy;

#[test]
fn exchange_i_assets() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let tax = 10;
    let creators_balance = 0;
    let intermediary_balance = 0;
    let others_balance = 100_000;
    let receiver_units = 10;
    let senders_units = 8;
    let sender_unit_exchange = senders_units - 2;
    let recipients_units = 5;
    let recipient_unit_exchange = recipients_units - 4;
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let meta_data3 = "asset3";
    let meta_data4 = "asset4";
    let meta_data5 = "asset5";
    let meta_data6 = "asset6";
    let intermediary_commision = 100;

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (creator_pk, _) = crypto::gen_keypair();
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();

    let (asset1, info1) = create_asset(meta_data1, senders_units, asset_fees(tax, 0), &creator_pk);
    let (asset2, info2) = create_asset(meta_data2, senders_units, asset_fees(tax, 0), &creator_pk);
    let (asset3, info3) = create_asset(meta_data3, senders_units, asset_fees(tax, 0), &creator_pk);
    let (asset4, info4) = create_asset(meta_data4, receiver_units, asset_fees(tax, 0), &creator_pk);
    let (asset5, info5) = create_asset(meta_data5, receiver_units, asset_fees(tax, 0), &creator_pk);
    let (asset6, info6) = create_asset(meta_data6, receiver_units, asset_fees(tax, 0), &creator_pk);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_value_to_wallet(&sender_pk, asset1.clone(), info1)
        .add_asset_value_to_wallet(&sender_pk, asset2.clone(), info2)
        .add_asset_value_to_wallet(&sender_pk, asset3.clone(), info3)
        .add_asset_value_to_wallet(&recipient_pk, asset4.clone(), info4)
        .add_asset_value_to_wallet(&recipient_pk, asset5.clone(), info5)
        .add_asset_value_to_wallet(&recipient_pk, asset6.clone(), info6)
        .create();
    let api = testkit.api();
    let genesis_balance = testkit.fetch_wallet(&default_genesis_key()).balance();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk)
        .commission(intermediary_commision)
        .sender_key_pair(sender_pk, sender_sk)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), senders_units))
        .sender_add_asset_value(AssetBundle::new(asset2.id(), sender_unit_exchange))
        .sender_add_asset_value(AssetBundle::new(asset3.id(), senders_units))
        .recipient_add_asset_value(AssetBundle::new(asset4.id(), receiver_units))
        .recipient_add_asset_value(AssetBundle::new(asset5.id(), recipient_unit_exchange))
        .recipient_add_asset_value(AssetBundle::new(asset6.id(), recipient_unit_exchange))
        .build();

    let tx_hash = tx_exchange_assets.hash();

    let (status, response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_exchange_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    let sender_wallet = testkit.fetch_wallet(&sender_pk);
    let recipient_wallet = testkit.fetch_wallet(&recipient_pk);
    let intermediary_wallet = testkit.fetch_wallet(&intermediary_pk);
    let genesis_wallet = testkit.fetch_wallet(&default_genesis_key());
    let creator_wallet = testkit.fetch_wallet(&creator_pk);

    let asset_fee = 
        senders_units * tax +
        sender_unit_exchange * tax +
        senders_units * tax +
        receiver_units * tax +
        recipient_unit_exchange * tax + 
        recipient_unit_exchange * tax;
    let expected_balance = others_balance - transaction_fee/2 - asset_fee / 2 - intermediary_commision / 2;
    let expected_intermediary_balance = intermediary_balance + intermediary_commision;
    let expected_genesis_balance = genesis_balance + transaction_fee;
    let expected_creator_balance = creators_balance + asset_fee;

    assert_eq!(sender_wallet.balance(), expected_balance);
    assert_eq!(recipient_wallet.balance(), expected_balance);
    assert_eq!(intermediary_wallet.balance(), expected_intermediary_balance);
    assert_eq!(genesis_wallet.balance(), expected_genesis_balance);
    assert_eq!(creator_wallet.balance(), expected_creator_balance);

    assert_eq!(sender_wallet.assets(), 
        vec![
            AssetBundle::new(asset2.id(), senders_units - sender_unit_exchange),
            AssetBundle::new(asset4.id(), receiver_units),
            AssetBundle::new(asset5.id(), recipient_unit_exchange),
            AssetBundle::new(asset6.id(), recipient_unit_exchange)
        ]
    );

    assert_eq!(recipient_wallet.assets(),
        vec![
            AssetBundle::new(asset5.id(), receiver_units - recipient_unit_exchange),
            AssetBundle::new(asset6.id(), receiver_units - recipient_unit_exchange),
            AssetBundle::new(asset1.id(), senders_units),
            AssetBundle::new(asset2.id(), sender_unit_exchange),
            AssetBundle::new(asset3.id(), senders_units)
        ]
    );
}

#[test]
fn exchange_i_assets_creator_is_sender() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let tax = 10;
    let intermediary_balance = 0;
    let others_balance = 100_000;
    let receiver_units = 10;
    let senders_units = 8;
    let sender_unit_exchange = senders_units - 2;
    let recipients_units = 5;
    let recipient_unit_exchange = recipients_units - 4;
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let intermediary_commision = 100;

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();

    let (asset1, info1) = create_asset(meta_data1, senders_units, asset_fees(tax, 0), &sender_pk);
    let (asset2, info2) = create_asset(meta_data2, receiver_units, asset_fees(tax, 0), &sender_pk);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_value_to_wallet(&sender_pk, asset1.clone(), info1)
        .add_asset_value_to_wallet(&recipient_pk, asset2.clone(), info2)
        .create();
    let api = testkit.api();
    let genesis_balance = testkit.fetch_wallet(&default_genesis_key()).balance();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk)
        .commission(intermediary_commision)
        .sender_key_pair(sender_pk, sender_sk)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), sender_unit_exchange))
        .recipient_add_asset_value(AssetBundle::new(asset2.id(), recipient_unit_exchange))
        .build();

    let tx_hash = tx_exchange_assets.hash();

    let (status, response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_exchange_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    let sender_wallet = testkit.fetch_wallet(&sender_pk);
    let recipient_wallet = testkit.fetch_wallet(&recipient_pk);
    let intermediary_wallet = testkit.fetch_wallet(&intermediary_pk);
    let genesis_wallet = testkit.fetch_wallet(&default_genesis_key());

    let asset_fee = 
        sender_unit_exchange * tax + 
        recipient_unit_exchange * tax;
    let expected_senders_balance = others_balance - transaction_fee / 2 - intermediary_commision / 2 + asset_fee / 2;
    let expected_recipients_balance = others_balance - transaction_fee / 2 - intermediary_commision / 2 - asset_fee / 2;
    let expected_intermediary_balance = intermediary_balance + intermediary_commision;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(sender_wallet.balance(), expected_senders_balance);
    assert_eq!(recipient_wallet.balance(), expected_recipients_balance);
    assert_eq!(intermediary_wallet.balance(), expected_intermediary_balance);
    assert_eq!(genesis_wallet.balance(), expected_genesis_balance);

    assert_eq!(sender_wallet.assets(), 
        vec![
            AssetBundle::new(asset1.id(), senders_units - sender_unit_exchange),
            AssetBundle::new(asset2.id(), recipient_unit_exchange),
        ]
    );

    assert_eq!(recipient_wallet.assets(),
        vec![
            AssetBundle::new(asset2.id(), receiver_units - recipient_unit_exchange),
            AssetBundle::new(asset1.id(), sender_unit_exchange),
        ]
    );
}

#[test]
fn exchange_i_assets_creator_is_intermediary() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let tax = 10;
    let intermediary_balance = 0;
    let others_balance = 100_000;
    let receiver_units = 10;
    let senders_units = 8;
    let sender_unit_exchange = senders_units - 2;
    let recipients_units = 5;
    let recipient_unit_exchange = recipients_units - 4;
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let intermediary_commision = 100;

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();

    let (asset1, info1) = create_asset(meta_data1, senders_units, asset_fees(tax, 0), &intermediary_pk);
    let (asset2, info2) = create_asset(meta_data2, receiver_units, asset_fees(tax, 0), &intermediary_pk);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_value_to_wallet(&sender_pk, asset1.clone(), info1)
        .add_asset_value_to_wallet(&recipient_pk, asset2.clone(), info2)
        .create();
    let api = testkit.api();
    let genesis_balance = testkit.fetch_wallet(&default_genesis_key()).balance();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk)
        .commission(intermediary_commision)
        .sender_key_pair(sender_pk, sender_sk)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), sender_unit_exchange))
        .recipient_add_asset_value(AssetBundle::new(asset2.id(), recipient_unit_exchange))
        .build();

    let tx_hash = tx_exchange_assets.hash();

    let (status, response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_exchange_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    let sender_wallet = testkit.fetch_wallet(&sender_pk);
    let recipient_wallet = testkit.fetch_wallet(&recipient_pk);
    let intermediary_wallet = testkit.fetch_wallet(&intermediary_pk);
    let genesis_wallet = testkit.fetch_wallet(&default_genesis_key());

    let asset_fee = 
        sender_unit_exchange * tax + 
        recipient_unit_exchange * tax;
    let expected_balance = others_balance - transaction_fee/2 - asset_fee / 2 - intermediary_commision / 2;
    let expected_intermediary_balance = intermediary_balance + intermediary_commision + asset_fee;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(sender_wallet.balance(), expected_balance);
    assert_eq!(recipient_wallet.balance(), expected_balance);
    assert_eq!(intermediary_wallet.balance(), expected_intermediary_balance);
    assert_eq!(genesis_wallet.balance(), expected_genesis_balance);

    assert_eq!(sender_wallet.assets(), 
        vec![
            AssetBundle::new(asset1.id(), senders_units - sender_unit_exchange),
            AssetBundle::new(asset2.id(), recipient_unit_exchange),
        ]
    );

    assert_eq!(recipient_wallet.assets(),
        vec![
            AssetBundle::new(asset2.id(), receiver_units - recipient_unit_exchange),
            AssetBundle::new(asset1.id(), sender_unit_exchange),
        ]
    );
}

#[test]
fn exchange_i_assets_payer_fee_intermediary() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let tax = 10;
    let intermediary_balance = 100_000;
    let others_balance = 100_000;
    let receiver_units = 10;
    let senders_units = 8;
    let sender_unit_exchange = senders_units - 2;
    let recipients_units = 5;
    let recipient_unit_exchange = recipients_units - 4;
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let intermediary_commision = 100;

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();

    let (asset1, info1) = create_asset(meta_data1, senders_units, asset_fees(tax, 0), &intermediary_pk);
    let (asset2, info2) = create_asset(meta_data2, receiver_units, asset_fees(tax, 0), &intermediary_pk);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&intermediary_pk, Wallet::new(intermediary_balance, vec![]))
        .add_asset_value_to_wallet(&sender_pk, asset1.clone(), info1)
        .add_asset_value_to_wallet(&recipient_pk, asset2.clone(), info2)
        .create();
    let api = testkit.api();
    let genesis_balance = testkit.fetch_wallet(&default_genesis_key()).balance();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk)
        .commission(intermediary_commision)
        .sender_key_pair(sender_pk, sender_sk)
        .fee_strategy(FeeStrategy::Intermediary)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), sender_unit_exchange))
        .recipient_add_asset_value(AssetBundle::new(asset2.id(), recipient_unit_exchange))
        .build();

    let tx_hash = tx_exchange_assets.hash();

    let (status, response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_exchange_assets);
    assert_eq!(tx_status, Ok(Ok(())));

    let sender_wallet = testkit.fetch_wallet(&sender_pk);
    let recipient_wallet = testkit.fetch_wallet(&recipient_pk);
    let intermediary_wallet = testkit.fetch_wallet(&intermediary_pk);
    let genesis_wallet = testkit.fetch_wallet(&default_genesis_key());

    let expected_balance = others_balance;
    let expected_intermediary_balance = intermediary_balance - transaction_fee;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(sender_wallet.balance(), expected_balance);
    assert_eq!(recipient_wallet.balance(), expected_balance);
    assert_eq!(intermediary_wallet.balance(), expected_intermediary_balance);
    assert_eq!(genesis_wallet.balance(), expected_genesis_balance);

    assert_eq!(sender_wallet.assets(), 
        vec![
            AssetBundle::new(asset1.id(), senders_units - sender_unit_exchange),
            AssetBundle::new(asset2.id(), recipient_unit_exchange),
        ]
    );

    assert_eq!(recipient_wallet.assets(),
        vec![
            AssetBundle::new(asset2.id(), receiver_units - recipient_unit_exchange),
            AssetBundle::new(asset1.id(), sender_unit_exchange),
        ]
    );
}

#[test]
fn exchange_i_assets_asset_not_found() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let tax = 10;
    let intermediary_balance = 100_000;
    let others_balance = 100_000;
    let receiver_units = 10;
    let senders_units = 8;
    let sender_unit_exchange = senders_units - 2;
    let recipients_units = 5;
    let recipient_unit_exchange = recipients_units - 4;
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let intermediary_commision = 100;

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();

    let (asset1, _) = create_asset(meta_data1, senders_units, asset_fees(tax, 0), &intermediary_pk);
    let (asset2, _) = create_asset(meta_data2, receiver_units, asset_fees(tax, 0), &intermediary_pk);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&intermediary_pk, Wallet::new(intermediary_balance, vec![]))
        .create();
    let api = testkit.api();
    let genesis_balance = testkit.fetch_wallet(&default_genesis_key()).balance();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk)
        .commission(intermediary_commision)
        .sender_key_pair(sender_pk, sender_sk)
        .fee_strategy(FeeStrategy::Intermediary)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), sender_unit_exchange))
        .recipient_add_asset_value(AssetBundle::new(asset2.id(), recipient_unit_exchange))
        .build();

    let tx_hash = tx_exchange_assets.hash();

    let (status, response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_exchange_assets);
    assert_eq!(tx_status, Ok(Err(Error::AssetNotFound)));

    let sender_wallet = testkit.fetch_wallet(&sender_pk);
    let recipient_wallet = testkit.fetch_wallet(&recipient_pk);
    let intermediary_wallet = testkit.fetch_wallet(&intermediary_pk);
    let genesis_wallet = testkit.fetch_wallet(&default_genesis_key());

    let expected_balance = others_balance;
    let expected_intermediary_balance = intermediary_balance - transaction_fee;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(sender_wallet.balance(), expected_balance);
    assert_eq!(recipient_wallet.balance(), expected_balance);
    assert_eq!(intermediary_wallet.balance(), expected_intermediary_balance);
    assert_eq!(genesis_wallet.balance(), expected_genesis_balance);
}

#[test]
fn exchange_i_assets_insufficient_assets() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let tax = 10;
    let intermediary_balance = 100_000;
    let others_balance = 100_000;
    let receiver_units = 10;
    let senders_units = 8;
    let sender_unit_exchange = senders_units - 2;
    let recipients_units = 5;
    let recipient_unit_exchange = recipients_units - 4;
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let intermediary_commision = 100;

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();

    let (asset1, info1) = create_asset(meta_data1, senders_units, asset_fees(tax, 0), &intermediary_pk);
    let (asset2, info2) = create_asset(meta_data2, receiver_units, asset_fees(tax, 0), &intermediary_pk);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&intermediary_pk, Wallet::new(intermediary_balance, vec![]))
        .add_asset_value_to_wallet(&sender_pk, asset1.clone(), info1)
        .add_asset_value_to_wallet(&recipient_pk, asset2.clone(), info2)
        .create();
    let api = testkit.api();
    let genesis_balance = testkit.fetch_wallet(&default_genesis_key()).balance();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk)
        .commission(intermediary_commision)
        .sender_key_pair(sender_pk, sender_sk)
        .fee_strategy(FeeStrategy::Intermediary)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), sender_unit_exchange*2))
        .recipient_add_asset_value(AssetBundle::new(asset2.id(), recipient_unit_exchange))
        .build();

    let tx_hash = tx_exchange_assets.hash();

    let (status, response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_exchange_assets);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientAssets)));

    let sender_wallet = testkit.fetch_wallet(&sender_pk);
    let recipient_wallet = testkit.fetch_wallet(&recipient_pk);
    let intermediary_wallet = testkit.fetch_wallet(&intermediary_pk);
    let genesis_wallet = testkit.fetch_wallet(&default_genesis_key());

    let expected_balance = others_balance;
    let expected_intermediary_balance = intermediary_balance - transaction_fee;
    let expected_genesis_balance = genesis_balance + transaction_fee;

    assert_eq!(sender_wallet.balance(), expected_balance);
    assert_eq!(recipient_wallet.balance(), expected_balance);
    assert_eq!(intermediary_wallet.balance(), expected_intermediary_balance);
    assert_eq!(genesis_wallet.balance(), expected_genesis_balance);
}

#[test]
fn exchange_i_assets_insufficient_funds() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let tax = 10;
    let intermediary_balance = 0;
    let others_balance = 100_000;
    let receiver_units = 10;
    let senders_units = 8;
    let sender_unit_exchange = senders_units - 2;
    let recipients_units = 5;
    let recipient_unit_exchange = recipients_units - 4;
    let meta_data1 = "asset1";
    let meta_data2 = "asset2";
    let intermediary_commision = 100;

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (intermediary_pk, intermediary_sk) = crypto::gen_keypair();

    let (asset1, info1) = create_asset(meta_data1, senders_units, asset_fees(tax, 0), &intermediary_pk);
    let (asset2, info2) = create_asset(meta_data2, receiver_units, asset_fees(tax, 0), &intermediary_pk);

    let mut testkit = EvoTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&intermediary_pk, Wallet::new(intermediary_balance, vec![]))
        .add_asset_value_to_wallet(&sender_pk, asset1.clone(), info1)
        .add_asset_value_to_wallet(&recipient_pk, asset2.clone(), info2)
        .create();
    let api = testkit.api();
    let genesis_balance = testkit.fetch_wallet(&default_genesis_key()).balance();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange_with_intermediary()
        .intermediary_key_pair(intermediary_pk, intermediary_sk)
        .commission(intermediary_commision)
        .sender_key_pair(sender_pk, sender_sk)
        .fee_strategy(FeeStrategy::Intermediary)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), sender_unit_exchange*2))
        .recipient_add_asset_value(AssetBundle::new(asset2.id(), recipient_unit_exchange))
        .build();

    let tx_hash = tx_exchange_assets.hash();

    let (status, response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::Created);
    assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash })));

    let (_, tx_status) = api.get_tx_status(&tx_exchange_assets);
    assert_eq!(tx_status, Ok(Err(Error::InsufficientFunds)));

    let sender_wallet = testkit.fetch_wallet(&sender_pk);
    let recipient_wallet = testkit.fetch_wallet(&recipient_pk);
    let intermediary_wallet = testkit.fetch_wallet(&intermediary_pk);
    let genesis_wallet = testkit.fetch_wallet(&default_genesis_key());

    let expected_balance = others_balance;
    let expected_intermediary_balance = intermediary_balance;
    let expected_genesis_balance = genesis_balance;

    assert_eq!(sender_wallet.balance(), expected_balance);
    assert_eq!(recipient_wallet.balance(), expected_balance);
    assert_eq!(intermediary_wallet.balance(), expected_intermediary_balance);
    assert_eq!(genesis_wallet.balance(), expected_genesis_balance);
}