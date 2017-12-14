extern crate exonum;
extern crate dmbc;

use exonum::blockchain::Transaction;
use exonum::crypto;
use exonum::storage::{Database, MemoryDB};

use dmbc::service::asset::{Asset, AssetID, AssetInfo};
use dmbc::service::builders::transaction;
use dmbc::service::builders::wallet;
use dmbc::service::schema::asset::AssetSchema;
use dmbc::service::schema::wallet::WalletSchema;

#[test]
fn add_assets() {
    let (public_key, secret_key) = crypto::gen_keypair();

    let absent_data = "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f";
    let existing_data = "a8d5c97d-9978-4111-9947-7a95dcb31d0f";

    let absent_id = AssetID::new(absent_data, &public_key).unwrap();
    let existing_id = AssetID::new(existing_data, &public_key).unwrap();

    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_add_assets()
        .add_asset(absent_data, 45)
        .add_asset(existing_data, 17)
        .seed(85)
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    AssetSchema::map(fork, |mut s| {
        s.assets().put(&existing_id, AssetInfo::new(&public_key, 3))
    });

    let wallet = wallet::Builder::new()
        .key(public_key)
        .balance(2000)
        .add_asset(existing_data, 3)
        .build();

    WalletSchema::map(fork, |mut s| s.wallets().put(&public_key, wallet));

    tx.execute(fork);

    let existing_info = AssetSchema::map(fork, |mut s| s.info(&existing_id).unwrap());

    assert_eq!(20, existing_info.amount());

    let wallet = WalletSchema::map(fork, |mut s| s.wallet(tx.pub_key()).unwrap());

    assert_eq!(2000 - tx.get_fee(), wallet.balance());
    assert_eq!(20, wallet.asset(existing_id).unwrap().amount());
    assert_eq!(45, wallet.asset(absent_id).unwrap().amount());
}

#[test]
fn create_wallet() {
    use dmbc::service::transaction::INIT_BALANCE;

    let (public_key, secret_key) = crypto::gen_keypair();
    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_create_wallet()
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    let wallet = wallet::Builder::new()
        .key(public_key)
        .balance(INIT_BALANCE)
        .build();

    tx.execute(fork);

    WalletSchema::map(fork, |mut schema| {
        assert_eq!(Some(wallet), schema.wallet(tx.pub_key()));
    });
}

#[test]
fn delete_assets() {
    let (public_key, secret_key) = crypto::gen_keypair();

    let data_1 = "deleted";
    let data_2 = "removed from wallet";
    let data_3 = "removed from network";

    let tx = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset(data_1, 10)
        .add_asset(data_2, 20)
        .add_asset(data_3, 30)
        .seed(113)
        .build();

    let wallet = wallet::Builder::new()
        .key(public_key)
        .balance(2000)
        .add_asset(data_1, 20)
        .add_asset(data_2, 20)
        .add_asset(data_3, 30)
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    let id_1 = AssetID::new(data_1, &public_key).unwrap();
    let id_2 = AssetID::new(data_2, &public_key).unwrap();
    let id_3 = AssetID::new(data_3, &public_key).unwrap();

    AssetSchema::map(fork, |mut s| {
        s.add_asset(&id_1, &public_key, 30);
        s.add_asset(&id_2, &public_key, 30);
        s.add_asset(&id_3, &public_key, 30);
    });

    WalletSchema::map(fork, move |mut s| s.wallets().put(&public_key, wallet));

    tx.execute(fork);

    AssetSchema::map(fork, |mut s| {
        assert_eq!(Some(20), s.info(&id_1).map(|a| a.amount()));
        assert_eq!(Some(10), s.info(&id_2).map(|a| a.amount()));
        assert_eq!(None, s.info(&id_3).map(|a| a.amount()));
    });

    WalletSchema::map(fork, |mut s| {
        let wallet = s.wallet(&public_key).unwrap();
        assert_eq!(Some(Asset::new(id_1, 10)), wallet.asset(id_1));
        assert_eq!(None, wallet.asset(id_2));
        assert_eq!(None, wallet.asset(id_3));
    });
}

#[test]
fn delete_assets_fails() {
    let (public_key, secret_key) = crypto::gen_keypair();

    let data = "asset";
    let id = AssetID::new(data, &public_key).unwrap();

    let wallet = wallet::Builder::new()
        .key(public_key)
        .balance(2000)
        .add_asset(data, 20)
        .build();

    let tx_too_many = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset(data, 30)
        .seed(9)
        .build();

    let _tx_doesnt_exist = transaction::Builder::new()
        .keypair(public_key, secret_key.clone())
        .tx_del_assets()
        .add_asset("absent", 999)
        .seed(9)
        .build();

    let db = MemoryDB::new();
    let fork = &mut db.fork();

    AssetSchema::map(fork, |mut s| s.add_asset(&id, &public_key, 20));
    WalletSchema::map(fork, |mut s| s.wallets().put(&public_key, wallet));

    tx_too_many.execute(fork);

    AssetSchema::map(fork, |mut s| {
        assert_eq!(Some(20), s.info(&id).map(|a| a.amount()));
    });

    WalletSchema::map(fork, |mut s| {
        assert_eq!(
            Some(20),
            s.wallet(&public_key)
             .and_then(|w| w.asset(id))
             .map(|a| a.amount()));
    });
}

