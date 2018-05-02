extern crate exonum;
extern crate exonum_testkit;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate dmbc;

mod fuzz_data;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Read;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::process;

use exonum::blockchain::Transaction;
use exonum::crypto::SecretKey;
use exonum::messages::{MessageBuffer, RawMessage, Message};
use exonum_testkit::TestKitBuilder;

use dmbc::currency::transactions::builders::fee;
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::transactions::{AddAssets, DeleteAssets, Exchange, Trade, Transfer,
                                   ADD_ASSETS_ID, DELETE_ASSETS_ID, EXCHANGE_ID,
                                   TRADE_ID, TRANSFER_ID};
use dmbc::currency::Service;

use fuzz_data::FuzzData;

fn main() {
    fuzz(|| {
        let mut data_vec = Vec::new();
        File::open("./fuzz-data.toml")
            .expect("Unable to open fuzz-data.toml")
            .read_to_end(&mut data_vec)
            .unwrap();
        let data: FuzzData = toml::from_slice(&data_vec).unwrap();
        let setup = setup_transactions(&data);

        let mut testkit = TestKitBuilder::validator()
            .with_validators(1)
            .with_service(Service())
            .create();

        testkit.create_block();

        let mut data = Vec::new();
        io::stdin().read_to_end(&mut data).unwrap();
        let message = RawMessage::new(MessageBuffer::from_vec(data));
        let tx = tx_from_raw(message.clone());

        if let Ok(tx) = tx {
            let iter = setup.into_iter().chain(Some(tx).into_iter());
            testkit.create_block_with_transactions(iter);
        }
    });
}

#[derive(Debug)]
struct TxFromRawError;

impl Error for TxFromRawError {
    fn description(&self) -> &str {
        "Unknown message type"
    }
}

impl ::std::fmt::Display for TxFromRawError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "{}.", self.description())
    }
}

fn tx_from_raw(rm: RawMessage) -> Result<Box<Transaction>, Box<Error>> {
    match rm.message_type() {
        ADD_ASSETS_ID => AddAssets::from_raw(rm)
            .map(|t| t.into())
            .map_err(|e| e.into()),
        DELETE_ASSETS_ID => DeleteAssets::from_raw(rm)
            .map(|t| t.into())
            .map_err(|e| e.into()),
        EXCHANGE_ID => Exchange::from_raw(rm)
            .map(|t| t.into())
            .map_err(|e| e.into()),
        TRADE_ID => Trade::from_raw(rm).map(|t| t.into()).map_err(|e| e.into()),
        TRANSFER_ID => Transfer::from_raw(rm)
            .map(|t| t.into())
            .map_err(|e| e.into()),
        _ => Err(Box::new(TxFromRawError)),
    }
}

fn setup_transactions(fuzz: &FuzzData) -> Vec<Box<Transaction>> {
    let mut transactions: Vec<Box<Transaction>> = Vec::new();

    let zero_fee = fee::Builder::new()
        .trade(0, 1)
        .exchange(0, 1)
        .transfer(0, 1)
        .build();

    // setup alice
    transactions.push(Box::new(
        transaction::Builder::new()
            .keypair(fuzz.genesis, SecretKey::zero())
            .tx_transfer()
            .recipient(fuzz.alice)
            .amount(10_000_000_000)
            .build(),
    ));

    transactions.push(Box::new(
        transaction::Builder::new()
            .keypair(fuzz.alice, SecretKey::zero())
            .tx_add_assets()
            .add_asset("alice_asset", 10, zero_fee.clone())
            .build(),
    ));

    // setup bob
    transactions.push(Box::new(
        transaction::Builder::new()
            .keypair(fuzz.genesis, SecretKey::zero())
            .tx_transfer()
            .recipient(fuzz.bob)
            .amount(10_000_000_000)
            .build(),
    ));

    transactions.push(Box::new(
        transaction::Builder::new()
            .keypair(fuzz.bob, SecretKey::zero())
            .tx_add_assets()
            .add_asset("bob_asset", 10, zero_fee.clone())
            .build(),
    ));

    transactions
}

fn fuzz<F>(f: F)
where
    F: FnOnce(),
{
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    if let Err(error) = result {
        if let Some(e) = error.downcast_ref::<&str>() {
            eprintln!("{}", e);
        } else if let Some(e) = error.downcast_ref::<String>() {
            eprintln!("{}", e);
        } else if let Some(e) = error.downcast_ref::<::std::io::Error>() {
            eprintln!("{}", e);
        } else {
            eprintln!("Unknown error.");
        }
        process::abort();
    }
}
