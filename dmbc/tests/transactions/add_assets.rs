extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::crypto;
use exonum::messages::Message;

use dmbc::currency::assets::{AssetBundle, MetaAsset, AssetId};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::error::Error;

use transactions::*;

#[test]
fn add_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    set_configuration(&mut testkit, TransactionFees::new(10, 1, 0, 0, 0, 0));

    let (public_key, secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &tx);
    let mut current_balance = 1_00_000_000u64;
    testkit.create_block();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let meta_asset_receiver = MetaAsset::new(&receiver_key, meta_data, 3, fees.clone());

    // Maйним ассет с отсылкой его на другой кошелек. Ассета нет в сети, ранее не майнили,
    // Кошелек на который майнят нет других ассетов.
    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset_receiver.clone())
        .seed(85)
        .build();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let empty_assets: Vec<AssetBundle> = Vec::new();
    assert_eq!(empty_assets, mining_wallet.assets());
    current_balance -= 10 + 1*3; // 10 bc.fee + 1 asset_id * 3 asset_id.amount
    assert_eq!(current_balance , mining_wallet.balance());

    let receiver_wallet = get_wallet(&api, &receiver_key);
    let asset = AssetBundle::from_data(meta_data, 3, &public_key);
    assert_eq!(vec![asset.clone()], receiver_wallet.assets());

    let bc_asset_info = get_asset_info(&api, &asset.id()).unwrap();
    assert_eq!(Some(meta_asset_receiver.to_info(&public_key)), bc_asset_info);



    // Майним ассет который был замайнен ранее и отправлен на кошелек на котором есть замайненный ассет.
    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset_receiver.clone())
        .seed(86)
        .build();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let empty_assets: Vec<AssetBundle> = Vec::new();
    assert_eq!(empty_assets, mining_wallet.assets());
    current_balance -= 10 + 1*3; // 10 bc.fee + 1 asset_id * 3 asset_id.amount

    assert_eq!(current_balance , mining_wallet.balance());

    let receiver_wallet = get_wallet(&api, &receiver_key);
    let asset = AssetBundle::from_data(meta_data, 6, &public_key);
    assert_eq!(vec![asset.clone()], receiver_wallet.assets());

    let bc_asset_info = get_asset_info(&api, &asset.id()).unwrap().unwrap();
    assert_eq!(meta_asset_receiver.to_info(&public_key).creator(), bc_asset_info.creator());
    assert_eq!(3 + 3, bc_asset_info.amount());


    // Майним уже замайненный ассет. Оставляем его на кошельке майнера.
    let meta_asset = MetaAsset::new(&public_key, meta_data, 5, fees.clone());

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(87)
        .build();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let asset = AssetBundle::from_data(meta_data, 5, &public_key);
    assert_eq!(vec![asset.clone()], mining_wallet.assets());
    current_balance -= 10 + 1*5; // 10 bc.fee + 1 asset_id * 5 asset_id.amount
    assert_eq!(current_balance, mining_wallet.balance());

    let bc_asset_info = get_asset_info(&api, &asset.id()).unwrap().unwrap();
    assert_eq!(meta_asset.to_info(&public_key).creator(), bc_asset_info.creator());
    assert_eq!(3 + 3 + 5, bc_asset_info.amount());

    // Майним уже замайненный ассет. Оставляем его на кошельке майнера.
    // Майним уже замайненный ассет c указанием другого кошелька получателя.
    let meta_miners_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());
    let meta_receivers_asset = MetaAsset::new(&receiver_key, meta_data, 2,fees.clone());

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_miners_asset.clone())
        .add_asset_value(meta_receivers_asset.clone())
        .seed(88)
        .build();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let bc_asset_info = get_asset_info(&api, &asset.id()).unwrap().unwrap();
    assert_eq!(meta_asset.to_info(&public_key).creator(), bc_asset_info.creator());
    assert_eq!(3 + 3 + 5 + 1 + 2, bc_asset_info.amount());

    let mining_wallet = get_wallet(&api, &public_key);
    let assets: Vec<AssetBundle> = vec![AssetBundle::from_data(meta_data, 6, &public_key), ];
    assert_eq!(assets, mining_wallet.assets());
    current_balance -= 10 + 1*1 + 1*2;
    assert_eq!(current_balance, mining_wallet.balance());

    let receiver_wallet = get_wallet(&api, &receiver_key);
    let assets: Vec<AssetBundle> = vec![AssetBundle::from_data(meta_data, 8, &public_key), ];
    assert_eq!(assets, receiver_wallet.assets());


    let fees2 = fee::Builder::new()
        .trade(100, 33)
        .exchange(67, 20)
        .transfer(59, 55)
        .build();

    let meta_data2 = r#"{"name":"test_item2","type":"skin","category":"revolver","image":"http://test.com/test_item2.jpg"}"#;
    let asset_id2 = AssetId::from_data(meta_data2, &public_key);
    let meta_asset_receiver2 = MetaAsset::new(&receiver_key, meta_data2, 4, fees2.clone());

    // Maйним ассет с отсылкой его на другой кошелек. Ассета нет в сети, ранее не майнили,
    // Кошелек на который майнят eсть другой ассет.
    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset_receiver2.clone())
        .seed(99)
        .build();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let assets: Vec<AssetBundle> = vec![AssetBundle::from_data(meta_data, 6, &public_key), ];
    assert_eq!(assets, mining_wallet.assets());
    current_balance -= 10 + 1*4; // 10 bc.fee + 1 asset_id * 4 asset_id.amount
    assert_eq!(current_balance , mining_wallet.balance());

    let receiver_wallet = get_wallet(&api, &receiver_key);
    let assets: Vec<AssetBundle> = vec![
        AssetBundle::from_data(meta_data, 8, &public_key),
        AssetBundle::from_data(meta_data2, 4, &public_key),
    ];
    assert_eq!(assets, receiver_wallet.assets());

    let bc_asset_info = get_asset_info(&api, &asset_id2).unwrap().unwrap();
    assert_eq!(meta_asset_receiver2.to_info(&public_key), bc_asset_info);



    // Майним уже замайненный ассет. Оставляем его на кошельке майнера.
    let meta_asset2 = MetaAsset::new(&public_key, meta_data2, 7, fees2.clone());

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset2.clone())
        .seed(97)
        .build();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let assets: Vec<AssetBundle> = vec![
        AssetBundle::from_data(meta_data, 6, &public_key),
        AssetBundle::from_data(meta_data2, 7, &public_key),
    ];
    assert_eq!(assets, mining_wallet.assets());
    current_balance -= 10 + 1*7; // 10 bc.fee + 1 asset_id * 5 asset_id.amount
    assert_eq!(current_balance, mining_wallet.balance());

    let bc_asset_info = get_asset_info(&api, &asset_id2).unwrap().unwrap();
    assert_eq!(meta_asset2.to_info(&public_key).creator(), bc_asset_info.creator());
    assert_eq!(4 + 7, bc_asset_info.amount());

    // Майним уже замайненный ассет. Оставляем его на кошельке майнера.
    // Майним уже замайненный ассет c указанием другого кошелька получателя.
    let meta_miners_asset2 = MetaAsset::new(&public_key, meta_data2, 2, fees2.clone());
    let meta_receivers_asset2 = MetaAsset::new(&receiver_key, meta_data2, 1, fees2.clone());

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_miners_asset2.clone())
        .add_asset_value(meta_receivers_asset2.clone())
        .seed(93)
        .build();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Ok(())), s);

    let bc_asset_info = get_asset_info(&api, &asset_id2).unwrap().unwrap();
    assert_eq!(meta_asset2.to_info(&public_key).creator(), bc_asset_info.creator());
    assert_eq!(4 + 7 + 2 + 1, bc_asset_info.amount());

    let mining_wallet = get_wallet(&api, &public_key);
    let assets: Vec<AssetBundle> = vec![
        AssetBundle::from_data(meta_data, 6, &public_key),
        AssetBundle::from_data(meta_data2, 7 + 2, &public_key),
    ];
    assert_eq!(assets, mining_wallet.assets());
    current_balance -= 10 + 1*2 + 1*1;
    assert_eq!(current_balance, mining_wallet.balance());

    let receiver_wallet = get_wallet(&api, &receiver_key);
    let assets: Vec<AssetBundle> = vec![
        AssetBundle::from_data(meta_data, 8, &public_key),
        AssetBundle::from_data(meta_data2, 4 + 1, &public_key),
    ];
    assert_eq!(assets, receiver_wallet.assets());
}

#[test]
fn add_assets_with_different_fees() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    set_configuration(&mut testkit, TransactionFees::new(10, 1, 0, 0, 0, 0));

    let (public_key, secret_key) = crypto::gen_keypair();

    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &tx);
    let mut current_balance = 1_00_000_000u64;
    testkit.create_block();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());

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

    let mining_wallet = get_wallet(&api, &public_key);
    let assets: Vec<AssetBundle> = vec![
        meta_asset.to_bundle(asset_id.clone()),
    ];
    assert_eq!(assets, mining_wallet.assets());
    current_balance -= 10 + 1*1; // 10 bc.fee + 1 asset_id * 1 asset_id.amount
    assert_eq!(current_balance , mining_wallet.balance());

    let bc_asset_info = get_asset_info(&api, &asset_id).unwrap().unwrap();
    assert_eq!(meta_asset.to_info(&public_key), bc_asset_info);

    let fees2 = fee::Builder::new()
        .trade(20, 10)
        .exchange(0, 10)
        .transfer(10, 10)
        .build();

    let meta_asset2 = MetaAsset::new(&public_key, meta_data, 1, fees2.clone());

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset2.clone())
        .seed(85)
        .build();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Err(Error::InvalidAssetInfo)), s);

    let mining_wallet = get_wallet(&api, &public_key);
    let assets: Vec<AssetBundle> = vec![
        meta_asset.to_bundle(asset_id.clone()),
    ];
    assert_eq!(assets, mining_wallet.assets());
    current_balance -= 10; // 10 bc.fee + 1 asset_id * 1 asset_id.amount
    assert_eq!(current_balance , mining_wallet.balance());

    let bc_asset_info = get_asset_info(&api, &asset_id).unwrap().unwrap();
    assert_eq!(meta_asset.to_info(&public_key), bc_asset_info);
}

#[test]
fn add_assets_insufficient_funds() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    set_configuration(&mut testkit, TransactionFees::new(10, 1_00_000_000, 0, 0, 0, 0));

    let (public_key, secret_key) = crypto::gen_keypair();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());

    let tx_add_assets = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset_value(meta_asset.clone())
        .seed(85)
        .build();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Err(Error::InsufficientFunds)), s);


    let mine_1_dmc = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &mine_1_dmc);
    testkit.create_block();

    post_tx(&api, &tx_add_assets);

    testkit.create_block();

    let s = get_status(&api, &tx_add_assets.hash());
    assert_eq!(Ok(Err(Error::InsufficientFunds)), s);
}

#[test]
fn add_assets_to_empty_wallet_without_meta_info() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    let (public_key, secret_key) = crypto::gen_keypair();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());

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

    let bc_asset_info = get_asset_info(&api, &asset_id).unwrap().unwrap();
    assert_eq!(meta_asset.to_info(&public_key), bc_asset_info);

    let mining_wallet = get_wallet(&api, &public_key);
    let assets: Vec<AssetBundle> = vec![
        meta_asset.to_bundle(asset_id.clone()),
    ];
    assert_eq!(assets, mining_wallet.assets());
}

#[test]
fn add_assets_to_empty_wallet_with_exist_meta_info() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    let (public_key, secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset_id = AssetId::from_data(meta_data, &public_key);
    let meta_asset = MetaAsset::new(&receiver_key, meta_data, 1, fees.clone());

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

    let bc_asset_info = get_asset_info(&api, &asset_id).unwrap().unwrap();
    assert_eq!(meta_asset.to_info(&public_key), bc_asset_info);

    let mining_wallet = get_wallet(&api, &public_key);
    let empty_assets: Vec<AssetBundle> = vec![];
    assert_eq!(empty_assets, mining_wallet.assets());

    let receiver_wallet = get_wallet(&api, &receiver_key);
    let assets: Vec<AssetBundle> = vec![meta_asset.to_bundle(asset_id.clone())];
    assert_eq!(assets, receiver_wallet.assets());


    let meta_asset = MetaAsset::new(&public_key, meta_data, 1, fees.clone());

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

    let bc_asset_info = get_asset_info(&api, &asset_id).unwrap().unwrap();
    assert_eq!(meta_asset.to_info(&public_key).creator(), bc_asset_info.creator());
    assert_eq!(2, bc_asset_info.amount());

    let mining_wallet = get_wallet(&api, &public_key);
    assert_eq!(assets, mining_wallet.assets());

    let receiver_wallet = get_wallet(&api, &receiver_key);
    assert_eq!(assets, receiver_wallet.assets());
}
