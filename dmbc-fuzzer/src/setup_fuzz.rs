extern crate exonum;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate dmbc;

mod fuzz_data;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};

use exonum::crypto;
use exonum::crypto::SecretKey;
use exonum::storage::StorageValue;
use dmbc::service::builders::transaction;
use dmbc::service::builders::fee;

use fuzz_data::FuzzData;

fn setup() -> Result<(), Box<Error>> {
    let data: FuzzData = File::create("./fuzz-data.toml")
        .and_then(|mut f| {
            let (genesis, _) = crypto::gen_keypair();
            let (alice, _) = crypto::gen_keypair();
            let (bob, _) = crypto::gen_keypair();
            let data = FuzzData {
                genesis,
                alice,
                bob,
            };
            f.write_all(&toml::to_vec(&data).unwrap())?;
            Ok(data)
        })
        .or_else(|e| {
            if e.kind() != ErrorKind::AlreadyExists {
                return Err(e);
            }
            let mut data = Vec::new();
            File::open("./fuzz-data.toml")?.read_to_end(&mut data)?;
            Ok(toml::from_slice(&data).unwrap())
        })
        .map_err(Box::new)?;

    tx_file("./fuzz-in/tx_add_assets.in")
        .and_then(|mut f| {
            let tx = transaction::Builder::new()
                .keypair(data.alice, SecretKey::zero())
                .tx_add_assets()
                .add_asset(
                    "foo",
                    9,
                    fee::Builder::new()
                        .trade(1, 10)
                        .exchange(2, 10)
                        .transfer(3, 10)
                        .build(),
                )
                .seed(3)
                .build()
                .into_bytes();
            f.write_all(&tx).map_err(|e| e.into())
        })
        .unwrap_or_else(|e| eprintln!("{}", e));

    tx_file("./fuzz-in/tx_create_wallet.in")
        .and_then(|mut f| {
            let tx = transaction::Builder::new()
                .keypair(data.alice, SecretKey::zero())
                .tx_create_wallet()
                .build()
                .into_bytes();
            f.write_all(&tx).map_err(|e| e.into())
        })
        .unwrap_or_else(|e| eprintln!("{}", e));

    tx_file("./fuzz-in/tx_del_assets.in")
        .and_then(|mut f| {
            let tx = transaction::Builder::new()
                .keypair(data.alice, SecretKey::zero())
                .tx_del_assets()
                .add_asset("foo", 9)
                .seed(6)
                .build()
                .into_bytes();
            f.write_all(&tx).map_err(|e| e.into())
        })
        .unwrap_or_else(|e| eprintln!("{}", e));

    tx_file("./fuzz-in/tx_exchange.in")
        .and_then(|mut f| {
            let tx = transaction::Builder::new()
                .keypair(data.alice, SecretKey::zero())
                .tx_exchange()
                .sender_add_asset("alice_asset", 10)
                .sender_value(1000)
                .recipient(data.bob)
                .recipient_add_asset("bob_asset", 10)
                .recipient_value(1000)
                .fee_strategy(1)
                .seed(83)
                .build()
                .into_bytes();
            f.write_all(&tx).map_err(|e| e.into())
        })
        .unwrap_or_else(|e| eprintln!("{}", e));

    tx_file("./fuzz-in/tx_mining.in")
        .and_then(|mut f| {
            let tx = transaction::Builder::new()
                .keypair(data.alice, SecretKey::zero())
                .tx_mining()
                .seed(3)
                .build()
                .into_bytes();
            f.write_all(&tx).map_err(|e| e.into())
        })
        .unwrap_or_else(|e| eprintln!("{}", e));

    tx_file("./fuzz-in/tx_trade_assets.in")
        .and_then(|mut f| {
            let tx = transaction::Builder::new()
                .keypair(data.alice, SecretKey::zero())
                .tx_trade_assets()
                .buyer(data.bob)
                .add_asset("alice_asset", 10, 9001)
                .seed(38)
                .build()
                .into_bytes();
            f.write_all(&tx).map_err(|e| e.into())
        })
        .unwrap_or_else(|e| eprintln!("{}", e));

    tx_file("./fuzz-in/tx_transfer.in")
        .and_then(|mut f| {
            let tx = transaction::Builder::new()
                .keypair(data.alice, SecretKey::zero())
                .tx_transfer()
                .recipient(data.bob)
                .amount(9)
                .add_asset("alice_asset", 10)
                .seed(42)
                .build()
                .into_bytes();
            f.write_all(&tx).map_err(|e| e.into())
        })
        .unwrap_or_else(|e| eprintln!("{}", e));

    Ok(())
}

fn tx_file(path: &str) -> Result<File, Box<Error>> {
    let data_modified = fs::metadata("./fuzz-data.toml")
        .and_then(|md| md.modified())
        .map_err(Box::new)?;
    let tx_is_newer = fs::metadata(path)
        .and_then(|md| md.modified())
        .map(|modified| modified > data_modified);

    if let Ok(true) = tx_is_newer {
        return Err(format!("File is newer than fuzz-data.toml: {}", path).into());
    }

    File::create(path).map_err(|e| e.into())
}

fn main() {
    setup().unwrap();
}
