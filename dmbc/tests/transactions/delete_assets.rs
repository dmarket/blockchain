extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::crypto;
use exonum::messages::Message;

use dmbc::currency::assets::{AssetBundle, AssetId, AssetInfo, MetaAsset};
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::error::Error;
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;

use transactions::*;

#[test]
fn delete_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(
        &mut testkit,
        TransactionFees::with_default_key(0, 0, 100, 0, 0, 0),
    );

    let (public_key, secret_key) = crypto::gen_keypair();
    let mine_1_dmc = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &mine_1_dmc);
    testkit.create_block();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 2, fees.clone());
    //Maйним ассет в колличестве 2.
    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    let tx_hash = tx_add_assets.hash();
    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let bc_asset_info = get_asset_info(&api, &asset_id).unwrap().unwrap();
    assert_eq!(meta_asset.to_info(&public_key, &tx_hash), bc_asset_info);

    let mining_wallet = get_wallet(&api, &public_key);
    assert_eq!(
        vec![meta_asset.to_bundle(asset_id.clone())],
        mining_wallet.assets()
    );

    //Удаляем ассет в количестве 1
    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset(meta_data, 1)
        .seed(5)
        .build();

    let tx_hash = tx_add_assets.hash();
    post_tx(&api, &tx_delete_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_delete_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let bc_asset_info = get_asset_info(&api, &asset_id).unwrap().unwrap();
    let a = AssetInfo::new(&public_key, &tx_hash, 1, fees.clone());
    assert_eq!(a, bc_asset_info);

    let mining_wallet = get_wallet(&api, &public_key);
    let a: Vec<AssetBundle> = vec![AssetBundle::new(asset_id.clone(), 1)];
    assert_eq!(a, mining_wallet.assets());
    assert_eq!(DMC_1 - 100, mining_wallet.balance());

    //Удаляем ассет в количестве 1
    let tx_delete_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset(meta_data, 1)
        .seed(6)
        .build();

    post_tx(&api, &tx_delete_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_delete_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let bc_asset_info = get_asset_info(&api, &asset_id).unwrap();
    assert_eq!(None, bc_asset_info);

    let mining_wallet = get_wallet(&api, &public_key);
    let e: Vec<AssetBundle> = vec![];
    assert_eq!(e, mining_wallet.assets());
    assert_eq!(DMC_1 - 100 - 100, mining_wallet.balance());
}

#[test]
fn delete_nonexistent_asset() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(
        &mut testkit,
        TransactionFees::with_default_key(0, 0, 100, 0, 0, 0),
    );

    let (public_key, secret_key) = crypto::gen_keypair();
    let mine_1_dmc = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &mine_1_dmc);
    testkit.create_block();

    let asset = AssetBundle::from_data("meta_data", 1, &public_key);

    //Удаляем ассет.
    let tx_del_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset_value(asset.clone())
        .seed(1)
        .build();

    post_tx(&api, &tx_del_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_del_assets.hash());
    assert_eq!(Ok(Err(Error::AssetNotFound)), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let e: Vec<AssetBundle> = vec![];
    assert_eq!(e, mining_wallet.assets());
    assert_eq!(DMC_1 - 100, mining_wallet.balance());
}

#[test]
fn delete_nonexistent_asset2() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(
        &mut testkit,
        TransactionFees::with_default_key(0, 0, 100, 0, 0, 0),
    );

    let (public_key, secret_key) = crypto::gen_keypair();
    let mine_1_dmc = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &mine_1_dmc);
    testkit.create_block();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();
    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());

    //Maйним другой ассет который не будем удалять
    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    //Удаляем ассет.
    let tx_del_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset_value(AssetBundle::from_data("meta_data2", 1, &public_key))
        .seed(1)
        .build();

    post_tx(&api, &tx_del_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_del_assets.hash());
    assert_eq!(Ok(Err(Error::AssetNotFound)), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let a: Vec<AssetBundle> = vec![meta_asset.to_bundle(asset_id.clone())];
    assert_eq!(a, mining_wallet.assets());
    assert_eq!(DMC_1 - 100, mining_wallet.balance());
}

#[test]
fn delete_asset_more_than_on_balance() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(
        &mut testkit,
        TransactionFees::with_default_key(0, 0, 100, 0, 0, 0),
    );

    let (public_key, secret_key) = crypto::gen_keypair();
    let mine_1_dmc = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &mine_1_dmc);
    testkit.create_block();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();
    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());

    //Maйним другой ассет который не будем удалять
    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    //Удаляем ассет.
    let tx_del_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset_value(AssetBundle::from_data(meta_data, 2, &public_key))
        .seed(1)
        .build();

    post_tx(&api, &tx_del_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_del_assets.hash());
    assert_eq!(Ok(Err(Error::InsufficientAssets)), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let a: Vec<AssetBundle> = vec![meta_asset.to_bundle(asset_id.clone())];
    assert_eq!(a, mining_wallet.assets());
    assert_eq!(DMC_1 - 100, mining_wallet.balance());
}

#[test]
fn delete_asset_insufficient_funds() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(
        &mut testkit,
        TransactionFees::with_default_key(0, 0, DMC_1 + 1, 0, 0, 0),
    );

    let (public_key, secret_key) = crypto::gen_keypair();
    let mine_1_dmc = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &mine_1_dmc);
    testkit.create_block();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();
    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());

    //Maйним другой ассет который не будем удалять
    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    //Удаляем ассет.
    let tx_del_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset_value(meta_asset.to_bundle(asset_id.clone()))
        .seed(1)
        .build();

    post_tx(&api, &tx_del_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_del_assets.hash());
    assert_eq!(Ok(Err(Error::InsufficientFunds)), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let a: Vec<AssetBundle> = vec![meta_asset.to_bundle(asset_id.clone())];
    assert_eq!(a, mining_wallet.assets());
    assert_eq!(DMC_1, mining_wallet.balance());
}

#[test]
fn delete_asset_with_different_creator() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(
        &mut testkit,
        TransactionFees::with_default_key(0, 0, 100, 0, 0, 0),
    );

    let (creator_pk_key, creator_sk_key) = crypto::gen_keypair();
    let (public_key, secret_key) = crypto::gen_keypair();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();
    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &creator_pk_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());
    let asset = meta_asset.to_bundle(asset_id.clone());
    //Maйним creator_pk_key ассет и отправляем его на public_key кошелек
    let tx_add_assets = transaction::Builder::new()
        .keypair(creator_pk_key, creator_sk_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    let mine_1_dmc = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &mine_1_dmc);
    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &mine_1_dmc.hash());
    assert_eq!(Ok(Ok(())), s);
    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let wallet = get_wallet(&api, &public_key);
    assert_eq!(vec![asset.clone()], wallet.assets());
    assert_eq!(DMC_1, wallet.balance());

    //Удаляем ассет.
    let tx_del_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset_value(asset.clone())
        .seed(1)
        .build();

    post_tx(&api, &tx_del_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_del_assets.hash());
    assert_eq!(Ok(Err(Error::InvalidTransaction)), s);

    let wallet = get_wallet(&api, &public_key);
    assert_eq!(vec![asset.clone()], wallet.assets());
    assert_eq!(DMC_1 - 100, wallet.balance());
}

#[test]
fn delete_2_assets_1_asset_more_than_on_balance() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(
        &mut testkit,
        TransactionFees::with_default_key(0, 0, 100, 0, 0, 0),
    );

    let (public_key, secret_key) = crypto::gen_keypair();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();
    let asset_id1 = AssetId::from_data("meta_asset_1", &public_key);
    let asset_id2 = AssetId::from_data("meta_asset_2", &public_key);
    let meta_asset1 = MetaAsset::new(&public_key, "meta_asset_1", 5, fees.clone());
    let meta_asset2 = MetaAsset::new(&public_key, "meta_asset_2", 5, fees.clone());
    let asset1 = meta_asset1.to_bundle(asset_id1.clone());
    let asset2 = meta_asset2.to_bundle(asset_id2.clone());

    //Maйним 2 ассета по 5 штук
    let mine_1_dmc = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset1.clone())
        .add_asset_value(meta_asset2.clone())
        .seed(1)
        .build();

    post_tx(&api, &mine_1_dmc);
    post_tx(&api, &tx_add_assets);
    testkit.create_block();

    let s = get_status(&api, &mine_1_dmc.hash());
    assert_eq!(Ok(Ok(())), s);
    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let wallet = get_wallet(&api, &public_key);
    assert_eq!(vec![asset1.clone(), asset2.clone()], wallet.assets());
    assert_eq!(DMC_1, wallet.balance());

    //Удаляем ассеты.
    let tx_del_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset_value(AssetBundle::new(asset_id1.clone(), 3))
        .add_asset_value(AssetBundle::new(asset_id2.clone(), 7))
        .seed(1)
        .build();

    post_tx(&api, &tx_del_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_del_assets.hash());
    assert_eq!(Ok(Err(Error::InsufficientAssets)), s);

    let wallet = get_wallet(&api, &public_key);
    assert_eq!(vec![asset1.clone(), asset2.clone()], wallet.assets());
    assert_eq!(DMC_1 - 100, wallet.balance());
}

#[test]
fn delete_2_assets_1_asset_have_other_creator() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    set_configuration(
        &mut testkit,
        TransactionFees::with_default_key(0, 0, 100, 0, 0, 0),
    );

    let (creator_pk_key, creator_sk_key) = crypto::gen_keypair();
    let (public_key, secret_key) = crypto::gen_keypair();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();
    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id1 = AssetId::from_data(meta_data, &creator_pk_key);
    let asset_id2 = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());
    let asset1 = meta_asset.to_bundle(asset_id1.clone());
    let asset2 = meta_asset.to_bundle(asset_id2.clone());

    //Maйним creator_pk_key ассет и отправляем его на public_key кошелек
    let mine_1_dmc = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    let tx_add_assets1 = transaction::Builder::new()
        .keypair(creator_pk_key, creator_sk_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(1)
        .build();

    let tx_add_assets2 = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(1)
        .build();

    post_tx(&api, &mine_1_dmc);
    testkit.create_block();
    post_tx(&api, &tx_add_assets1);
    testkit.create_block();
    post_tx(&api, &tx_add_assets2);
    testkit.create_block();

    let s = get_status(&api, &mine_1_dmc.hash());
    assert_eq!(Ok(Ok(())), s);
    let s = get_status(&api, &tx_add_assets1.hash());
    assert_eq!(Ok(Ok(())), s);
    let s = get_status(&api, &tx_add_assets2.hash());
    assert_eq!(Ok(Ok(())), s);

    let wallet = get_wallet(&api, &public_key);
    assert_eq!(vec![asset1.clone(), asset2.clone()], wallet.assets());
    assert_eq!(DMC_1, wallet.balance());

    //Удаляем ассет.
    let tx_del_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset_value(asset1.clone())
        .add_asset_value(asset2.clone())
        .seed(1)
        .build();

    post_tx(&api, &tx_del_assets);
    testkit.create_block();

    let s = get_status(&api, &tx_del_assets.hash());
    assert_eq!(Ok(Err(Error::InvalidTransaction)), s);

    let wallet = get_wallet(&api, &public_key);
    assert_eq!(vec![asset1.clone(), asset2.clone()], wallet.assets());
    assert_eq!(DMC_1 - 100, wallet.balance());
}
