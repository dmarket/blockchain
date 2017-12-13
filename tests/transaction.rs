extern crate exonum;
extern crate dmbc;

use exonum::crypto;
use exonum::blockchain::Transaction;
use exonum::storage::{Database, MemoryDB};

use dmbc::service::asset::{AssetID, AssetInfo};
use dmbc::service::schema::asset::AssetSchema;
use dmbc::service::schema::wallet::WalletSchema;
use dmbc::service::builders::transaction;
use dmbc::service::builders::wallet;

#[test]
fn add_assets_execute() {
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

    let db = Box::new(MemoryDB::new());
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

    let existing_info = AssetSchema::map(fork, |mut s| {
        s.info(&existing_id).unwrap()
    });

    assert_eq!(20, existing_info.amount());

    let wallet = WalletSchema::map(fork, |mut s| {
        s.wallet(tx.pub_key()).unwrap()
    });

    assert_eq!(2000 - tx.get_fee(), wallet.balance());
    assert_eq!(20, wallet.asset(existing_id).unwrap().amount());
    assert_eq!(45, wallet.asset(absent_id).unwrap().amount());
}

