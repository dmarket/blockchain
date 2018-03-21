extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::crypto;
use exonum::crypto::{PublicKey, Hash};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};
use exonum::encoding::serialize::reexport::Serialize;
use exonum::messages::Message;

use dmbc::currency::Service;
use dmbc::currency::SERVICE_NAME;
use dmbc::currency::assets::{AssetBundle, MetaAsset, AssetId};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::wallet::Wallet;
use dmbc::currency::api::transaction::{TransactionResponse, StatusResponse};
use dmbc::currency::api::asset::AssetResponse;

fn init_testkit() -> TestKit {
    TestKitBuilder::validator()
        .with_validators(4)
        .with_service(Service::new())
        .create()
}

fn get_wallet(api: &TestKitApi, pub_key: &PublicKey) -> Wallet {
    api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/wallets/{}", pub_key.to_string()),
    )
}

fn get_status(api: &TestKitApi, tx_hash: &Hash) -> StatusResponse {
    api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/transactions/{}", tx_hash.to_string()),
    )
}

fn get_asset_info(api: &TestKitApi, asset_id: &AssetId) -> AssetResponse {
    api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("/v1/assets/{}", asset_id.to_string()),
    )
}

fn post_tx<T>(api: &TestKitApi, tx: &T)
    where T:Message + Serialize
{
    let tx_response:TransactionResponse = api.post(
        ApiKind::Service(SERVICE_NAME),
        "v1/transactions",
        &tx
    );

    assert_eq!(tx_response.tx_hash, tx.hash());
}

#[test]
fn mine_wallet() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    let (public_key, secret_key) = crypto::gen_keypair();
    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &tx);

    testkit.create_block();

    let w = get_wallet(&api, &public_key);

    assert_eq!(1_00_000_000, w.balance()); //1 dmc
}

#[test]
fn add_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();

    let (public_key, secret_key) = crypto::gen_keypair();
    let (receiver_key, _) = crypto::gen_keypair();

    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_mine()
        .build();

    post_tx(&api, &tx);

    testkit.create_block();

    let fees = fee::Builder::new()
        .trade(10, 10)
        .exchange(10, 10)
        .transfer(10, 10)
        .build();

    let meta_data = r#"{"name":"test_item","type":"skin","category":"gun","image":"http://test.com/test_item.jpg"}"#;
    let asset = AssetBundle::from_data(meta_data, 3, &public_key);
    let meta_asset = MetaAsset::new(
        &receiver_key,
        meta_data,
        3,
        fees.clone()
    );

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
    let empty_assets: Vec<AssetBundle> = Vec::new();
    assert_eq!(empty_assets, mining_wallet.assets());

    let receiver_wallet = get_wallet(&api, &receiver_key);
    assert_eq!(vec![asset.clone()], receiver_wallet.assets());

    let bc_asset_info = get_asset_info(&api, &asset.id()).unwrap();
    assert_eq!(meta_asset.to_info(&public_key), bc_asset_info);
}

//#[test]
//fn add_assets_fails() {
//    let (public_key, secret_key) = crypto::gen_keypair();
//    let (receiver_key, _) = crypto::gen_keypair();
//
//    let data = "a8d5c97d-9978-4111-9947-7a95dcb31d0f";
//    let id = AssetId::new(data, &public_key).unwrap();
//
//    let fees = fee::Builder::new()
//        .trade(10, 10)
//        .exchange(10, 10)
//        .transfer(10, 10)
//        .build();
//
//    let tx_fees = fee::Builder::new()
//        .trade(1000, 1000)
//        .exchange(10, 10)
//        .transfer(10, 10)
//        .build();
//
//    let tx = transaction::Builder::new()
//        .keypair(public_key, secret_key.clone())
//        .tx_add_assets()
//        .add_asset_receiver(receiver_key, data, 45, tx_fees.clone())
//        .seed(85)
//        .build();
//
//    let mut testkit = TestKitBuilder::validator()
//        .with_service(CurrencyService::new())
//        .create();
//
//    let fork = &mut testkit.blockchain_mut().fork();
//
//    let wallet = wallet::Builder::new().key(public_key).balance(2000).build();
//    WalletSchema::map(fork, |mut s| {
//        s.wallets().put(&public_key, wallet);
//    });
//
//    AssetSchema::map(fork, |mut s| {
//        s.add_asset(&id, &public_key, 1, fees.clone());
//    });
//    fork.checkpoint();
//    tx.execute(fork);
//
//    let wallet = WalletSchema::map(fork, |mut s| s.wallet(tx.pub_key()));
//    let tx_status = TxStatusSchema::map(fork, |mut s| s.get_status(&tx.hash())).unwrap();
//    let asset_info = AssetSchema::map(fork, |mut s| s.assets().get(&id));
//
//    let expected_status = TxStatus::Fail;
//    assert_eq!(tx_status, expected_status);
//    assert_eq!(2000 - tx.get_fee(fork).amount(), wallet.balance());
//    assert_eq!(asset_info.unwrap().amount(), 1);
//}

//#[test]
//fn delete_assets() {
//    let (public_key, secret_key) = crypto::gen_keypair();
//
//    let data_1 = "deleted";
//    let data_2 = "removed from wallet";
//    let data_3 = "removed from network";
//
//    let tx = transaction::Builder::new()
//        .keypair(public_key, secret_key.clone())
//        .tx_del_assets()
//        .add_asset(data_1, 10)
//        .add_asset(data_2, 20)
//        .add_asset(data_3, 30)
//        .seed(113)
//        .build();
//
//    let wallet = wallet::Builder::new()
//        .key(public_key)
//        .balance(2000)
//        .add_asset(data_1, 20)
//        .add_asset(data_2, 20)
//        .add_asset(data_3, 30)
//        .build();
//
//    let mut testkit = TestKitBuilder::validator()
//        .with_service(CurrencyService::new())
//        .create();
//
//    let fork = &mut testkit.blockchain_mut().fork();
//
//    let id_1 = AssetId::new(data_1, &public_key).unwrap();
//    let id_2 = AssetId::new(data_2, &public_key).unwrap();
//    let id_3 = AssetId::new(data_3, &public_key).unwrap();
//
//    let fee = fee::Builder::new()
//        .trade(10, 10)
//        .exchange(10, 10)
//        .transfer(10, 10)
//        .build();
//
//    AssetSchema::map(fork, |mut s| {
//        s.add_asset(&id_1, &public_key, 30, fee.clone());
//        s.add_asset(&id_2, &public_key, 30, fee.clone());
//        s.add_asset(&id_3, &public_key, 30, fee.clone());
//    });
//
//    WalletSchema::map(fork, move |mut s| s.wallets().put(&public_key, wallet));
//
//    tx.execute(fork);
//
//    AssetSchema::map(fork, |mut s| {
//        assert_eq!(Some(20), s.info(&id_1).map(|a| a.amount()));
//        assert_eq!(Some(10), s.info(&id_2).map(|a| a.amount()));
//        assert_eq!(None, s.info(&id_3).map(|a| a.amount()));
//    });
//
//    WalletSchema::map(fork, |mut s| {
//        let wallet = s.wallet(&public_key);
//        assert_eq!(Some(Asset::new(id_1, 10)), wallet.asset(id_1));
//        assert_eq!(None, wallet.asset(id_2));
//        assert_eq!(None, wallet.asset(id_3));
//    });
//}
//
//#[test]
//fn delete_assets_fails() {
//    let (public_key, secret_key) = crypto::gen_keypair();
//
//    let data = "asset";
//    let id = AssetId::new(data, &public_key).unwrap();
//
//    let wallet = wallet::Builder::new()
//        .key(public_key)
//        .balance(2000)
//        .add_asset(data, 20)
//        .build();
//
//    let tx_too_many = transaction::Builder::new()
//        .keypair(public_key, secret_key.clone())
//        .tx_del_assets()
//        .add_asset(data, 30)
//        .seed(9)
//        .build();
//
//    let tx_doesnt_exist = transaction::Builder::new()
//        .keypair(public_key, secret_key.clone())
//        .tx_del_assets()
//        .add_asset("absent", 999)
//        .seed(9)
//        .build();
//
//    let mut testkit = TestKitBuilder::validator()
//        .with_service(CurrencyService::new())
//        .create();
//
//    let fork = &mut testkit.blockchain_mut().fork();
//
//    let fee = fee::Builder::new()
//        .trade(10, 10)
//        .exchange(10, 10)
//        .transfer(10, 10)
//        .build();
//
//    AssetSchema::map(fork, |mut s| s.add_asset(&id, &public_key, 20, fee));
//    WalletSchema::map(fork, |mut s| s.wallets().put(&public_key, wallet));
//
//    tx_too_many.execute(fork);
//
//    AssetSchema::map(fork, |mut s| {
//        assert_eq!(Some(20), s.info(&id).map(|a| a.amount()));
//    });
//
//    WalletSchema::map(fork, |mut s| {
//        assert_eq!(
//            Some(20),
//            s.wallet(&public_key).asset(id).map(|a| a.amount())
//        );
//    });
//
//    tx_doesnt_exist.execute(fork);
//
//    TxStatusSchema::map(fork, |mut s| {
//        assert_eq!(Some(TxStatus::Fail), s.get_status(&tx_doesnt_exist.hash()));
//    });
//}

//#[test]
//fn exchange() {
//    let (sender_public, sender_secret) = crypto::gen_keypair();
//    let (recipient_public, _) = crypto::gen_keypair();
//
//    let sender_data_1 = "sender asset 1";
//    let sender_id_1 = AssetId::new(sender_data_1, &sender_public).unwrap();
//
//    let sender_data_2 = "sender asset 2";
//    let sender_id_2 = AssetId::new(sender_data_2, &sender_public).unwrap();
//
//    let recipient_data_1 = "recipient asset 1";
//    let recipient_id_1 = AssetId::new(recipient_data_1, &recipient_public).unwrap();
//
//    let recipient_data_2 = "recipient asset 2";
//    let recipient_id_2 = AssetId::new(recipient_data_2, &recipient_public).unwrap();
//
//    let sender = wallet::Builder::new()
//        .key(sender_public)
//        .balance(100)
//        .add_asset(sender_data_1, 10)
//        .add_asset(sender_data_2, 30)
//        .build();
//
//    let recipient = wallet::Builder::new()
//        .key(recipient_public)
//        .balance(100)
//        .add_asset(recipient_data_1, 30)
//        .add_asset(recipient_data_2, 50)
//        .build();
//
//    let tx = transaction::Builder::new()
//        .keypair(sender_public, sender_secret.clone())
//        .tx_exchange()
//        .sender_add_asset(sender_data_1, 10)
//        .sender_add_asset(sender_data_2, 15)
//        .sender_value(50)
//        .recipient(recipient_public)
//        .recipient_add_asset(recipient_data_1, 30)
//        .recipient_add_asset(recipient_data_2, 25)
//        .fee_strategy(1)
//        .data_info("test_transaction")
//        .build();
//
//    let mut testkit = TestKitBuilder::validator()
//        .with_service(CurrencyService::new())
//        .create();
//
//    let fork = &mut testkit.blockchain_mut().fork();
//
//    let fee = fee::Builder::new()
//        .trade(10, 10)
//        .exchange(10, 10)
//        .transfer(10, 10)
//        .build();
//
//    AssetSchema::map(fork, |mut s| {
//        s.add_asset(&sender_id_1, &sender_public, 10, fee.clone());
//        s.add_asset(&sender_id_2, &sender_public, 30, fee.clone());
//        s.add_asset(&recipient_id_1, &recipient_public, 30, fee.clone());
//        s.add_asset(&recipient_id_2, &recipient_public, 50, fee.clone());
//    });
//
//    WalletSchema::map(fork, |mut s| {
//        s.wallets().put(&sender_public, sender);
//        s.wallets().put(&recipient_public, recipient);
//    });
//
//    fork.checkpoint();
//    tx.execute(fork);
//
//    WalletSchema::map(fork, |mut s| {
//        let sender = s.wallet(&sender_public);
//        let recipient = s.wallet(&recipient_public);
//
//        assert_eq!(None, sender.asset(sender_id_1).map(|a| a.amount()));
//        assert_eq!(Some(15), sender.asset(sender_id_2).map(|a| a.amount()));
//        assert_eq!(Some(30), sender.asset(recipient_id_1).map(|a| a.amount()));
//        assert_eq!(Some(25), sender.asset(recipient_id_2).map(|a| a.amount()));
//
//        assert_eq!(None, recipient.asset(recipient_id_1).map(|a| a.amount()));
//        assert_eq!(
//            Some(25),
//            recipient.asset(recipient_id_2).map(|a| a.amount())
//        );
//        assert_eq!(Some(10), recipient.asset(sender_id_1).map(|a| a.amount()));
//        assert_eq!(Some(15), recipient.asset(sender_id_2).map(|a| a.amount()));
//    });
//}

//#[test]
//fn trade_assets() {
//    let (creator_public, _) = crypto::gen_keypair();
//    let (seller_public, seller_secret) = crypto::gen_keypair();
//    let (buyer_public, _) = crypto::gen_keypair();
//
//    let full_data = "fully transferred asset";
//    let full_id = AssetId::new(full_data, &creator_public).unwrap();
//
//    let half_data = "partially transferred asset";
//    let half_id = AssetId::new(half_data, &creator_public).unwrap();
//
//    let asset1 = Asset::new(full_id, 20);
//    let asset2 = Asset::new(half_id, 20);
//
//    let creator = wallet::Builder::new()
//        .key(creator_public)
//        .balance(0)
//        .build();
//
//    let seller = wallet::Builder::new()
//        .key(seller_public)
//        .balance(2000)
//        .add_asset_value(asset1.clone())
//        .add_asset_value(asset2.clone())
//        .build();
//
//    let buyer = wallet::Builder::new()
//        .key(buyer_public)
//        .balance(2000)
//        .build();
//
//    let full_asset = Asset::new(asset1.id(), asset1.amount());
//    let hald_asset = Asset::new(asset2.id(), asset1.amount() / 2);
//
//    let tx = transaction::Builder::new()
//        .keypair(seller_public, seller_secret)
//        .tx_trade_assets()
//        .buyer(buyer_public)
//        .add_asset_value(full_asset.into_trade_asset(60))
//        .add_asset_value(hald_asset.into_trade_asset(20))
//        .seed(4)
//        .build();
//
//    let price = tx.offer().total_price();
//    assert_eq!(price, 1400);
//
//    let mut testkit = TestKitBuilder::validator()
//        .with_service(CurrencyService::new())
//        .create();
//
//    let fork = &mut testkit.blockchain_mut().fork();
//
//    let fee = fee::Builder::new()
//        .trade(10, 10)
//        .exchange(10, 10)
//        .transfer(10, 10)
//        .build();
//
//    AssetSchema::map(fork, |mut s| {
//        s.assets()
//            .put(&full_id, AssetInfo::new(&creator_public, 20, fee.clone()));
//        s.assets()
//            .put(&half_id, AssetInfo::new(&creator_public, 20, fee.clone()));
//    });
//
//    WalletSchema::map(fork, |mut s| {
//        s.wallets().put(&creator_public, creator);
//        s.wallets().put(&seller_public, seller);
//        s.wallets().put(&buyer_public, buyer);
//    });
//
//    tx.execute(fork);
//
//    let fee = tx.get_fee(fork);
//    let creators_fee = fee.assets_fees_total();
//    let tx_status = TxStatusSchema::map(fork, |mut s| s.get_status(&tx.hash())).unwrap();
//    assert_eq!(tx_status, TxStatus::Success);
//
//    WalletSchema::map(fork, |mut s| {
//        let seller = s.wallet(&seller_public);
//        let buyer = s.wallet(&buyer_public);
//        let creator = s.wallet(&creator_public);
//
//        assert_eq!(None, seller.asset(full_id).map(|a| a.amount()));
//        assert_eq!(Some(10), seller.asset(half_id).map(|a| a.amount()));
//
//        assert_eq!(Some(20), buyer.asset(full_id).map(|a| a.amount()));
//        assert_eq!(Some(10), buyer.asset(half_id).map(|a| a.amount()));
//
//        assert_eq!(2000 - price, buyer.balance());
//        assert_eq!(2000 + price - fee.amount(), seller.balance());
//
//        assert_eq!(creators_fee, creator.balance());
//    });
//}

//#[test]
//fn transfer() {
//    let (sender_public, sender_secret) = crypto::gen_keypair();
//    let (recipient_public, _) = crypto::gen_keypair();
//
//    let full_data = "fully transferred asset";
//    let full_id = AssetId::new(full_data, &sender_public).unwrap();
//
//    let half_data = "partially transferred asset";
//    let half_id = AssetId::new(half_data, &sender_public).unwrap();
//
//    let sender = wallet::Builder::new()
//        .key(sender_public)
//        .balance(2000)
//        .add_asset(full_data, 20)
//        .add_asset(half_data, 20)
//        .build();
//
//    let recipient = wallet::Builder::new()
//        .key(recipient_public)
//        .balance(2000)
//        .build();
//
//    let tx = transaction::Builder::new()
//        .keypair(sender_public, sender_secret)
//        .tx_transfer()
//        .recipient(recipient_public)
//        .amount(100)
//        .add_asset(full_data, 20)
//        .add_asset(half_data, 10)
//        .seed(123)
//        .data_info("transfer_transaction")
//        .build();
//
//    let mut testkit = TestKitBuilder::validator()
//        .with_service(CurrencyService::new())
//        .create();
//
//    let fork = &mut testkit.blockchain_mut().fork();
//
//    let fee = fee::Builder::new()
//        .trade(10, 10)
//        .exchange(10, 10)
//        .transfer(10, 10)
//        .build();
//
//    AssetSchema::map(fork, |mut s| {
//        s.assets()
//            .put(&full_id, AssetInfo::new(&sender_public, 20, fee.clone()));
//        s.assets()
//            .put(&half_id, AssetInfo::new(&sender_public, 20, fee.clone()));
//    });
//
//    WalletSchema::map(fork, |mut s| {
//        s.wallets().put(&sender_public, sender);
//        s.wallets().put(&recipient_public, recipient);
//    });
//
//    fork.checkpoint();
//    tx.execute(fork);
//
//    let tx_status = TxStatusSchema::map(fork, |mut s| s.get_status(&tx.hash())).unwrap();
//    assert_eq!(tx_status, TxStatus::Success);
//
//    let (sender, recipient) = WalletSchema::map(fork, |mut s| {
//        (s.wallet(&sender_public), s.wallet(&recipient_public))
//    });
//
//    assert_eq!(None, sender.asset(full_id).map(|a| a.amount()));
//    assert_eq!(Some(10), sender.asset(half_id).map(|a| a.amount()));
//
//    assert_eq!(Some(20), recipient.asset(full_id).map(|a| a.amount()));
//    assert_eq!(Some(10), recipient.asset(half_id).map(|a| a.amount()));
//
//    assert_eq!(2000 - tx.get_fee(fork).amount() - 100, sender.balance());
//    assert_eq!(2000 + 100, recipient.balance());
//}
