extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;
extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate mount;
extern crate serde_json;

pub mod dmbc_testkit;

use dmbc_testkit::{DmbcTestApiBuilder, DmbcTestKitApi};
use exonum::crypto;
use exonum::messages::Message;
use hyper::status::StatusCode;

use dmbc::currency::assets::{AssetBundle, MetaAsset, TradeAsset};
use dmbc::currency::configuration::{Configuration, TransactionFees, TransactionPermissions, WalletPermissions};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::transactions::components::FeeStrategy;
use dmbc::currency::transactions::components::{PM_ADD_ASSETS, PM_DELETE_ASSETS, 
    PM_EXCHANGE, PM_EXCHANGE_INTERMEDIARY, PM_TRADE, PM_TRADE_INTERMEDIARY, 
    PM_TRANSFER, PM_TRANSFER_WITH_FEES_PAYER, PM_ASK, PM_BID, PM_ALL_ALLOWED};
use dmbc::currency::wallet::Wallet;
use dmbc::currency::offers::OpenOffers;

#[test]
fn add_assets_wallet_permissions() {
    let fixed = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);
    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();
    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&creator_public_key, PM_ALL_ALLOWED ^ PM_ADD_ASSETS)], 
        PM_ALL_ALLOWED);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(
        &receiver_key,
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
    );
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(85)
        .build();

    let _tx_hash = tx_add_assets.hash();

    let (status, _response) = api.post_tx(&tx_add_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn add_assets_global_permissions() {
    let fixed = 10;
    let meta_data = "asset";
    let units = 3;
    let balance = 100_000;
    let transaction_fee = 10;
    let per_asset_fee = 4;
    let config_fees = TransactionFees::with_default_key(transaction_fee, per_asset_fee, 0, 0, 0, 0);
    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();
    let permissions = TransactionPermissions::new(vec![], PM_ALL_ALLOWED ^ PM_ADD_ASSETS);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(
        &receiver_key,
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
    );
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .seed(85)
        .build();

    let _tx_hash = tx_add_assets.hash();

    let (status, _response) = api.post_tx(&tx_add_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn delete_assets_wallet_permissions() {
    let meta_data = "asset";
    let units = 5;
    let units_to_remove = 1;
    let transaction_fee = 100;
    let balance = 100_000;
    let fixed = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);
    let (public_key, secret_key) = crypto::gen_keypair();
    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&public_key, PM_ALL_ALLOWED ^ PM_DELETE_ASSETS)],
        PM_ALL_ALLOWED
    );
    
    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, units_to_remove)
        .seed(5)
        .build();

    let _tx_hash = tx_delete_assets.hash();

    let (status, _response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn delete_assets_global_permissions() {
    let meta_data = "asset";
    let units = 5;
    let units_to_remove = 1;
    let transaction_fee = 100;
    let balance = 100_000;
    let fixed = 10;
    let config_fees = TransactionFees::with_default_key(0, 0, transaction_fee, 0, 0, 0);
    let (public_key, secret_key) = crypto::gen_keypair();
    let permissions = TransactionPermissions::new(vec![], PM_ALL_ALLOWED ^ PM_DELETE_ASSETS);
    
    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info.clone()))
        .create();
    let api = testkit.api();

    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_del_assets()
        .add_asset(meta_data, units_to_remove)
        .seed(5)
        .build();

    let _tx_hash = tx_delete_assets.hash();

    let (status, _response) = api.post_tx(&tx_delete_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn exchange_assets_global_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let permissions = TransactionPermissions::new(
        vec![], PM_ALL_ALLOWED ^ PM_EXCHANGE
    );
    let fixed = 10;
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

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (creator_pk, _) = crypto::gen_keypair();

    let (asset1, info1) = dmbc_testkit::create_asset(
        meta_data1,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset2, info2) = dmbc_testkit::create_asset(
        meta_data2,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset3, info3) = dmbc_testkit::create_asset(
        meta_data3,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset4, info4) = dmbc_testkit::create_asset(
        meta_data4,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset5, info5) = dmbc_testkit::create_asset(
        meta_data5,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset6, info6) = dmbc_testkit::create_asset(
        meta_data6,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_to_wallet(&sender_pk, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_pk, (asset2.clone(), info2))
        .add_asset_to_wallet(&sender_pk, (asset3.clone(), info3))
        .add_asset_to_wallet(&recipient_pk, (asset4.clone(), info4))
        .add_asset_to_wallet(&recipient_pk, (asset5.clone(), info5))
        .add_asset_to_wallet(&recipient_pk, (asset6.clone(), info6))
        .create();
    let api = testkit.api();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk)
        .fee_strategy(FeeStrategy::Recipient)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), senders_units))
        .sender_add_asset_value(AssetBundle::new(asset2.id(), sender_unit_exchange))
        .sender_add_asset_value(AssetBundle::new(asset3.id(), senders_units))
        .recipient_add_asset_value(AssetBundle::new(asset4.id(), receiver_units))
        .recipient_add_asset_value(AssetBundle::new(asset5.id(), recipient_unit_exchange))
        .recipient_add_asset_value(AssetBundle::new(asset6.id(), recipient_unit_exchange))
        .build();

    let _tx_hash = tx_exchange_assets.hash();

    let (status, _response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn exchange_assets_sender_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let fixed = 10;
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

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (creator_pk, _) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&sender_pk, PM_ALL_ALLOWED ^ PM_EXCHANGE)], PM_ALL_ALLOWED
    );

    let (asset1, info1) = dmbc_testkit::create_asset(
        meta_data1,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset2, info2) = dmbc_testkit::create_asset(
        meta_data2,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset3, info3) = dmbc_testkit::create_asset(
        meta_data3,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset4, info4) = dmbc_testkit::create_asset(
        meta_data4,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset5, info5) = dmbc_testkit::create_asset(
        meta_data5,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset6, info6) = dmbc_testkit::create_asset(
        meta_data6,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_to_wallet(&sender_pk, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_pk, (asset2.clone(), info2))
        .add_asset_to_wallet(&sender_pk, (asset3.clone(), info3))
        .add_asset_to_wallet(&recipient_pk, (asset4.clone(), info4))
        .add_asset_to_wallet(&recipient_pk, (asset5.clone(), info5))
        .add_asset_to_wallet(&recipient_pk, (asset6.clone(), info6))
        .create();
    let api = testkit.api();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk)
        .fee_strategy(FeeStrategy::Recipient)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), senders_units))
        .sender_add_asset_value(AssetBundle::new(asset2.id(), sender_unit_exchange))
        .sender_add_asset_value(AssetBundle::new(asset3.id(), senders_units))
        .recipient_add_asset_value(AssetBundle::new(asset4.id(), receiver_units))
        .recipient_add_asset_value(AssetBundle::new(asset5.id(), recipient_unit_exchange))
        .recipient_add_asset_value(AssetBundle::new(asset6.id(), recipient_unit_exchange))
        .build();

    let _tx_hash = tx_exchange_assets.hash();

    let (status, _response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn exchange_assets_recipient_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let fixed = 10;
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

    let (sender_pk, sender_sk) = crypto::gen_keypair();
    let (recipient_pk, recipient_sk) = crypto::gen_keypair();
    let (creator_pk, _) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&recipient_pk, PM_ALL_ALLOWED ^ PM_EXCHANGE)], PM_ALL_ALLOWED
    );

    let (asset1, info1) = dmbc_testkit::create_asset(
        meta_data1,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset2, info2) = dmbc_testkit::create_asset(
        meta_data2,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset3, info3) = dmbc_testkit::create_asset(
        meta_data3,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset4, info4) = dmbc_testkit::create_asset(
        meta_data4,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset5, info5) = dmbc_testkit::create_asset(
        meta_data5,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset6, info6) = dmbc_testkit::create_asset(
        meta_data6,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_to_wallet(&sender_pk, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_pk, (asset2.clone(), info2))
        .add_asset_to_wallet(&sender_pk, (asset3.clone(), info3))
        .add_asset_to_wallet(&recipient_pk, (asset4.clone(), info4))
        .add_asset_to_wallet(&recipient_pk, (asset5.clone(), info5))
        .add_asset_to_wallet(&recipient_pk, (asset6.clone(), info6))
        .create();
    let api = testkit.api();

    let tx_exchange_assets = transaction::Builder::new()
        .keypair(recipient_pk, recipient_sk)
        .tx_exchange()
        .sender(sender_pk)
        .sender_secret(sender_sk)
        .fee_strategy(FeeStrategy::Recipient)
        .sender_add_asset_value(AssetBundle::new(asset1.id(), senders_units))
        .sender_add_asset_value(AssetBundle::new(asset2.id(), sender_unit_exchange))
        .sender_add_asset_value(AssetBundle::new(asset3.id(), senders_units))
        .recipient_add_asset_value(AssetBundle::new(asset4.id(), receiver_units))
        .recipient_add_asset_value(AssetBundle::new(asset5.id(), recipient_unit_exchange))
        .recipient_add_asset_value(AssetBundle::new(asset6.id(), recipient_unit_exchange))
        .build();

    let _tx_hash = tx_exchange_assets.hash();

    let (status, _response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn exchange_intermediary_global_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let permissions = TransactionPermissions::new(
        vec![], PM_ALL_ALLOWED ^ PM_EXCHANGE_INTERMEDIARY
    );
    let fixed = 10;
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

    let (asset1, info1) = dmbc_testkit::create_asset(
        meta_data1,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset2, info2) = dmbc_testkit::create_asset(
        meta_data2,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset3, info3) = dmbc_testkit::create_asset(
        meta_data3,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset4, info4) = dmbc_testkit::create_asset(
        meta_data4,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset5, info5) = dmbc_testkit::create_asset(
        meta_data5,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset6, info6) = dmbc_testkit::create_asset(
        meta_data6,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_to_wallet(&sender_pk, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_pk, (asset2.clone(), info2))
        .add_asset_to_wallet(&sender_pk, (asset3.clone(), info3))
        .add_asset_to_wallet(&recipient_pk, (asset4.clone(), info4))
        .add_asset_to_wallet(&recipient_pk, (asset5.clone(), info5))
        .add_asset_to_wallet(&recipient_pk, (asset6.clone(), info6))
        .create();
    let api = testkit.api();

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

    let _tx_hash = tx_exchange_assets.hash();

    let (status, _response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn exchange_intermediary_sender_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let fixed = 10;
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

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&sender_pk, PM_ALL_ALLOWED ^ PM_EXCHANGE_INTERMEDIARY)], PM_ALL_ALLOWED
    );

    let (asset1, info1) = dmbc_testkit::create_asset(
        meta_data1,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset2, info2) = dmbc_testkit::create_asset(
        meta_data2,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset3, info3) = dmbc_testkit::create_asset(
        meta_data3,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset4, info4) = dmbc_testkit::create_asset(
        meta_data4,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset5, info5) = dmbc_testkit::create_asset(
        meta_data5,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset6, info6) = dmbc_testkit::create_asset(
        meta_data6,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_to_wallet(&sender_pk, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_pk, (asset2.clone(), info2))
        .add_asset_to_wallet(&sender_pk, (asset3.clone(), info3))
        .add_asset_to_wallet(&recipient_pk, (asset4.clone(), info4))
        .add_asset_to_wallet(&recipient_pk, (asset5.clone(), info5))
        .add_asset_to_wallet(&recipient_pk, (asset6.clone(), info6))
        .create();
    let api = testkit.api();

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

    let _tx_hash = tx_exchange_assets.hash();

    let (status, _response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn exchange_intermediary_recipient_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let fixed = 10;
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

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&recipient_pk, PM_ALL_ALLOWED ^ PM_EXCHANGE_INTERMEDIARY)], PM_ALL_ALLOWED
    );

    let (asset1, info1) = dmbc_testkit::create_asset(
        meta_data1,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset2, info2) = dmbc_testkit::create_asset(
        meta_data2,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset3, info3) = dmbc_testkit::create_asset(
        meta_data3,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset4, info4) = dmbc_testkit::create_asset(
        meta_data4,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset5, info5) = dmbc_testkit::create_asset(
        meta_data5,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset6, info6) = dmbc_testkit::create_asset(
        meta_data6,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_to_wallet(&sender_pk, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_pk, (asset2.clone(), info2))
        .add_asset_to_wallet(&sender_pk, (asset3.clone(), info3))
        .add_asset_to_wallet(&recipient_pk, (asset4.clone(), info4))
        .add_asset_to_wallet(&recipient_pk, (asset5.clone(), info5))
        .add_asset_to_wallet(&recipient_pk, (asset6.clone(), info6))
        .create();
    let api = testkit.api();

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

    let _tx_hash = tx_exchange_assets.hash();

    let (status, _response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn exchange_intermediary_intermediary_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, transaction_fee, 0, 0);
    let fixed = 10;
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

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&intermediary_pk, PM_ALL_ALLOWED ^ PM_EXCHANGE_INTERMEDIARY)], PM_ALL_ALLOWED
    );

    let (asset1, info1) = dmbc_testkit::create_asset(
        meta_data1,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset2, info2) = dmbc_testkit::create_asset(
        meta_data2,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset3, info3) = dmbc_testkit::create_asset(
        meta_data3,
        senders_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset4, info4) = dmbc_testkit::create_asset(
        meta_data4,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset5, info5) = dmbc_testkit::create_asset(
        meta_data5,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );
    let (asset6, info6) = dmbc_testkit::create_asset(
        meta_data6,
        receiver_units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &creator_pk,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&sender_pk, Wallet::new(others_balance, vec![]))
        .add_wallet_value(&recipient_pk, Wallet::new(others_balance, vec![]))
        .add_asset_to_wallet(&sender_pk, (asset1.clone(), info1))
        .add_asset_to_wallet(&sender_pk, (asset2.clone(), info2))
        .add_asset_to_wallet(&sender_pk, (asset3.clone(), info3))
        .add_asset_to_wallet(&recipient_pk, (asset4.clone(), info4))
        .add_asset_to_wallet(&recipient_pk, (asset5.clone(), info5))
        .add_asset_to_wallet(&recipient_pk, (asset6.clone(), info6))
        .create();
    let api = testkit.api();

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

    let _tx_hash = tx_exchange_assets.hash();

    let (status, _response) = api.post_tx(&tx_exchange_assets);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn trade_global_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let permissions = TransactionPermissions::new(
        vec![],
        PM_ALL_ALLOWED ^ PM_TRADE
    );
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &seller_public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .build();

    let _tx_hash = tx_trade.hash();

    let (status, _response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn trade_seller_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&seller_public_key, PM_ALL_ALLOWED ^ PM_TRADE)],
        PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &seller_public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .build();

    let _tx_hash = tx_trade.hash();

    let (status, _response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn trade_buyer_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&buyer_public_key, PM_ALL_ALLOWED ^ PM_TRADE)],
        PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &seller_public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .build();

    let _tx_hash = tx_trade.hash();

    let (status, _response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn trade_intermediary_global_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let permissions = TransactionPermissions::new(
        vec![], PM_ALL_ALLOWED ^ PM_TRADE_INTERMEDIARY
    );
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let intermediary_commission = 50;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &seller_public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .commission(intermediary_commission)
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .data_info("trade_test")
        .build();

    let _tx_hash = tx_trade.hash();

    let (status, _response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn trade_intermediary_seller_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let intermediary_commission = 50;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&seller_public_key, PM_ALL_ALLOWED ^ PM_TRADE_INTERMEDIARY)], 
        PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &seller_public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .commission(intermediary_commission)
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .data_info("trade_test")
        .build();

    let _tx_hash = tx_trade.hash();

    let (status, _response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn trade_intermediary_buyer_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let intermediary_commission = 50;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&buyer_public_key, PM_ALL_ALLOWED ^ PM_TRADE_INTERMEDIARY)], 
        PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &seller_public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .commission(intermediary_commission)
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .data_info("trade_test")
        .build();

    let _tx_hash = tx_trade.hash();

    let (status, _response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn trade_intermediary_intermediary_permissions() {
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, transaction_fee, 0);
    let meta_data = "asset";
    let fixed = 10;
    let balance = 100_000;
    let units = 3;
    let intermediary_commission = 50;
    let price = 500;

    let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
    let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
    let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&intermediary_public_key, PM_ALL_ALLOWED ^ PM_TRADE_INTERMEDIARY)], 
        PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &seller_public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&buyer_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&seller_public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&seller_public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .commission(intermediary_commission)
        .add_asset_value(TradeAsset::from_bundle(asset.clone(), price))
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .data_info("trade_test")
        .build();

    let _tx_hash = tx_trade.hash();

    let (status, _response) = api.post_tx(&tx_trade);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn transfer_global_permissions() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let permissions = TransactionPermissions::new(
        vec![], PM_ALL_ALLOWED ^ PM_TRANSFER
    );
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer()
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .seed(42)
        .build();

    let _tx_hash = tx_transfer.hash();

    let (status, _response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn transfer_sender_permissions() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&public_key, PM_ALL_ALLOWED ^ PM_TRANSFER)], PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer()
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .seed(42)
        .build();

    let _tx_hash = tx_transfer.hash();

    let (status, _response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn transfer_recipient_permissions() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&recipient_key, PM_ALL_ALLOWED ^ PM_TRANSFER)], PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(
        meta_data,
        units,
        dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()),
        &public_key,
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer()
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .seed(42)
        .build();

    let _tx_hash = tx_transfer.hash();

    let (status, _response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn transfer_with_fees_payer_global_permissions() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let permissions = TransactionPermissions::new(
        vec![], PM_ALL_ALLOWED ^ PM_TRANSFER_WITH_FEES_PAYER
    );
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;
    let coins = 300;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (fees_payer_pk, fees_payer_sk) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&fees_payer_pk, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer_with_fees_payer()
        .fees_payer(fees_payer_pk, fees_payer_sk)
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .amount(coins)
        .seed(42)
        .build();

    let _tx_hash = tx_transfer.hash();

    let (status, _response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn transfer_with_fees_payer_sender_permissions() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;
    let coins = 300;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (fees_payer_pk, fees_payer_sk) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&public_key, PM_ALL_ALLOWED ^ PM_TRANSFER_WITH_FEES_PAYER)], 
        PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&fees_payer_pk, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer_with_fees_payer()
        .fees_payer(fees_payer_pk, fees_payer_sk)
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .amount(coins)
        .seed(42)
        .build();

    let _tx_hash = tx_transfer.hash();

    let (status, _response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn transfer_with_fees_payer_recipient_permissions() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;
    let coins = 300;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (fees_payer_pk, fees_payer_sk) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&recipient_key, PM_ALL_ALLOWED ^ PM_TRANSFER_WITH_FEES_PAYER)], 
        PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&fees_payer_pk, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer_with_fees_payer()
        .fees_payer(fees_payer_pk, fees_payer_sk)
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .amount(coins)
        .seed(42)
        .build();

    let _tx_hash = tx_transfer.hash();

    let (status, _response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn transfer_with_fees_payer_and_his_permissions() {
    let fixed = 10;
    let transaction_fee = 1000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, transaction_fee);
    let meta_data = "asset";
    let units = 5;
    let balance = 100_000;
    let coins = 300;

    let (public_key, secret_key) = crypto::gen_keypair();
    let (fees_payer_pk, fees_payer_sk) = crypto::gen_keypair();
    let (recipient_key, _) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&fees_payer_pk, PM_ALL_ALLOWED ^ PM_TRANSFER_WITH_FEES_PAYER)], 
        PM_ALL_ALLOWED
    );

    let (asset, info) = dmbc_testkit::create_asset(meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()), &public_key);

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&fees_payer_pk, Wallet::new(balance, vec![]))
        .add_asset_to_wallet(&public_key, (asset.clone(), info))
        .create();
    let api = testkit.api();

    let tx_transfer = transaction::Builder::new()
        .keypair(public_key, secret_key)
        .tx_transfer_with_fees_payer()
        .fees_payer(fees_payer_pk, fees_payer_sk)
        .add_asset_value(asset.clone())
        .recipient(recipient_key)
        .amount(coins)
        .seed(42)
        .build();

    let _tx_hash = tx_transfer.hash();

    let (status, _response) = api.post_tx(&tx_transfer);
    testkit.create_block();

    // check post response
    assert_eq!(status, StatusCode::BadRequest);
}

#[test]
fn bid_global_permissions() {
    let fixed = 1;
    let meta_data = "asset";
    let units = 100;
    let balance = 100_000;
    let permissions = TransactionPermissions::new(
        vec![], PM_ALL_ALLOWED ^ PM_BID
    );
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (user1_pk, user1_sk) = crypto::gen_keypair();
    let (user2_pk, _user2_sk) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&user1_pk, Wallet::new(balance, vec![]))
        .add_wallet_value(&user2_pk, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&user1_pk, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .build();

    let (status, _) = api.post_tx(&tx_add_assets);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);
    let seller_assets = api
        .get_wallet_assets(&user1_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(units, seller_assets[0].amount());

    let creator_wallet = api.get_wallet(&creator_public_key);
    assert_eq!(balance, creator_wallet.balance);

    let _sample_offers = OpenOffers::new_open_offers();

    let bid_amount = 2;
    for bid_price in vec![10, 30, 50] {
        let asset_bundle = AssetBundle::from_data(meta_data, bid_amount, &creator_public_key);
        let trade_asset = TradeAsset::from_bundle(asset_bundle, bid_price);
        let tx_bid_offer = transaction::Builder::new()
            .keypair(user1_pk, user1_sk.clone())
            .tx_offer()
            .asset(trade_asset.clone())
            .data_info("bid")
            .bid_build();

        let (status, _) = api.post_tx(&tx_bid_offer);
        testkit.create_block();
        assert_eq!(status, StatusCode::BadRequest);
    }
}

#[test]
fn bid_wallet_permissions() {
    let fixed = 1;
    let meta_data = "asset";
    let units = 100;
    let balance = 100_000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (user1_pk, user1_sk) = crypto::gen_keypair();
    let (user2_pk, _user2_sk) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&user1_pk, PM_ALL_ALLOWED ^ PM_BID)], PM_ALL_ALLOWED
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&user1_pk, Wallet::new(balance, vec![]))
        .add_wallet_value(&user2_pk, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&user1_pk, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .build();

    let (status, _) = api.post_tx(&tx_add_assets);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);
    let seller_assets = api
        .get_wallet_assets(&user1_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(units, seller_assets[0].amount());

    let creator_wallet = api.get_wallet(&creator_public_key);
    assert_eq!(balance, creator_wallet.balance);

    let _sample_offers = OpenOffers::new_open_offers();

    let bid_amount = 2;
    for bid_price in vec![10, 30, 50] {
        let asset_bundle = AssetBundle::from_data(meta_data, bid_amount, &creator_public_key);
        let trade_asset = TradeAsset::from_bundle(asset_bundle, bid_price);
        let tx_bid_offer = transaction::Builder::new()
            .keypair(user1_pk, user1_sk.clone())
            .tx_offer()
            .asset(trade_asset.clone())
            .data_info("bid")
            .bid_build();

        let (status, _) = api.post_tx(&tx_bid_offer);
        testkit.create_block();
        assert_eq!(status, StatusCode::BadRequest);
    }
}

#[test]
fn ask_global_permissions() {
    let fixed = 1;
    let meta_data = "asset";
    let units = 100;
    let balance = 100_000;
    let permissions = TransactionPermissions::new(
        vec![], PM_ALL_ALLOWED ^ PM_ASK
    );
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (user1_pk, _user1_sk) = crypto::gen_keypair();
    let (user2_pk, user2_sk) = crypto::gen_keypair();

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&user1_pk, Wallet::new(balance, vec![]))
        .add_wallet_value(&user2_pk, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&user1_pk, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .build();

    let (status, _) = api.post_tx(&tx_add_assets);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);
    let seller_assets = api
        .get_wallet_assets(&user1_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(units, seller_assets[0].amount());

    let creator_wallet = api.get_wallet(&creator_public_key);
    assert_eq!(balance, creator_wallet.balance);

    let ask_amount = 2;
    let asset_bundle = AssetBundle::from_data(meta_data, ask_amount, &creator_public_key);
    for ask_price in vec![10, 30, 50] {
        let trade_asset = TradeAsset::from_bundle(asset_bundle.clone(), ask_price);
        let tx_ask_offer = transaction::Builder::new()
            .keypair(user2_pk, user2_sk.clone())
            .tx_offer()
            .asset(trade_asset)
            .data_info("ask")
            .ask_build();

        let (status, _) = api.post_tx(&tx_ask_offer);
        testkit.create_block();
        assert_eq!(status, StatusCode::BadRequest);
    }
}

#[test]
fn ask_user_permissions() {
    let fixed = 1;
    let meta_data = "asset";
    let units = 100;
    let balance = 100_000;
    let config_fees = TransactionFees::with_default_key(0, 0, 0, 0, 0, 0);

    let (creator_public_key, creator_secret_key) = crypto::gen_keypair();
    let (user1_pk, user1_sk) = crypto::gen_keypair();

    let permissions = TransactionPermissions::new(
        vec![WalletPermissions::new(&user1_pk, PM_ALL_ALLOWED ^ PM_ASK)], 
        PM_ALL_ALLOWED
    );

    let mut testkit = DmbcTestApiBuilder::new()
        .with_configuration(Configuration::new(config_fees, permissions))
        .add_wallet_value(&creator_public_key, Wallet::new(balance, vec![]))
        .add_wallet_value(&user1_pk, Wallet::new(balance, vec![]))
        .create();
    let api = testkit.api();

    // post the transaction
    let meta_asset = MetaAsset::new(&user1_pk, meta_data, units, dmbc_testkit::asset_fees(fixed, "0.0".parse().unwrap()));
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_public_key, creator_secret_key)
        .tx_add_assets()
        .add_asset_value(meta_asset)
        .build();

    let (status, _) = api.post_tx(&tx_add_assets);
    testkit.create_block();
    assert_eq!(status, StatusCode::Created);
    let seller_assets = api
        .get_wallet_assets(&user1_pk)
        .iter()
        .map(|a| a.into())
        .collect::<Vec<AssetBundle>>();
    assert_eq!(units, seller_assets[0].amount());

    let creator_wallet = api.get_wallet(&creator_public_key);
    assert_eq!(balance, creator_wallet.balance);

    let ask_amount = 2;
    let asset_bundle = AssetBundle::from_data(meta_data, ask_amount, &creator_public_key);
    for ask_price in vec![10, 30, 50] {
        let trade_asset = TradeAsset::from_bundle(asset_bundle.clone(), ask_price);
        let tx_ask_offer = transaction::Builder::new()
            .keypair(user1_pk, user1_sk.clone())
            .tx_offer()
            .asset(trade_asset)
            .data_info("ask")
            .ask_build();

        let (status, _) = api.post_tx(&tx_ask_offer);
        testkit.create_block();
        assert_eq!(status, StatusCode::BadRequest);
    }
}