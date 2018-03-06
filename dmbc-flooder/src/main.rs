extern crate dmbc;
extern crate exonum;
extern crate fnv;
extern crate mio;
extern crate mio_httpc;
extern crate rand;
extern crate serde;
extern crate serde_json;

use std::time::Duration;

use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey};
use fnv::FnvHashMap;
use mio::{Events, Poll};
use mio_httpc::{CallBuilder, Httpc, RecvState, Request};
use rand::Rng;
use serde::Serialize;

use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::assets::{AssetBundle, AssetId, MetaAsset, TradeAsset};

type Wallet = (PublicKey, SecretKey);

const MAX_AMOUNT: u64 = 10_000;
const ASSET_NAME: &str = "RAXXLA";

#[derive(Clone, Copy)]
enum OpState {
    CreateWallet = 0,
    AddAssets = 1,
    DelAssets = 2,
    Exchange = 3,
    ExchangeWithIntermediary = 4,
    Mining = 5,
    TradeAssets = 6,
    TradeAssetsWithIntermediary = 7,
    Transfer = 8,
}

impl OpState {
    fn next(self) -> OpState {
        use OpState::*;
        match self {
            CreateWallet => AddAssets,
            AddAssets => DelAssets,
            DelAssets => Exchange,
            Exchange => ExchangeWithIntermediary,
            ExchangeWithIntermediary => Mining,
            Mining => TradeAssets,
            TradeAssets => TradeAssetsWithIntermediary,
            TradeAssetsWithIntermediary => Transfer,
            Transfer => CreateWallet,
        }
    }

    fn advance(&mut self) {
        *self = self.next();
    }
}

struct Flooder {
    rng: rand::ThreadRng,
    wallets: Vec<Wallet>,
    assets: Vec<AssetBundle>,
    op_state: OpState,
}

impl Flooder {
    fn new() -> Self {
        Flooder {
            rng: rand::thread_rng(),
            wallets: Vec::new(),
            assets: Vec::new(),
            op_state: OpState::CreateWallet,
        }
    }

    fn next_tx(&mut self) -> String {
        match self.op_state {
            OpState::CreateWallet => {
                let wallet = crypto::gen_keypair();
                self.wallets.push(wallet.clone());
                let tx = transaction::Builder::new()
                    .keypair(wallet.0, wallet.1)
                    .tx_create_wallet()
                    .build();

                serialize(tx)
            }

            OpState::AddAssets => {
                let wallet = self.pick_wallet();
                let amount = self.rng.gen_range(0, MAX_AMOUNT);
                let fees = fee::Builder::new()
                    .trade(10, 10)
                    .exchange(10, 10)
                    .transfer(10, 10)
                    .build();
                let asset = MetaAsset::new(&wallet.0, ASSET_NAME, amount, fees);
                let id = AssetId::from_data(ASSET_NAME, &wallet.0);
                self.assets.push(asset.to_bundle(id));
                let tx = transaction::Builder::new()
                    .keypair(wallet.0, wallet.1)
                    .tx_add_assets()
                    .add_asset_value(asset)
                    .build();

                serialize(tx)
            }

            OpState::DelAssets => {
                let wallet = self.pick_wallet();
                let asset = self.split_asset();
                let tx = transaction::Builder::new()
                    .keypair(wallet.0, wallet.1)
                    .tx_del_assets()
                    .add_asset_value(asset)
                    .build();

                serialize(tx)
            }

            OpState::Exchange => {
                let sender = self.pick_wallet();
                let receiver = self.pick_wallet();

                let s_asset = self.pick_asset();
                let r_asset = self.pick_asset();

                let tx = transaction::Builder::new()
                    .keypair(sender.0, sender.1)
                    .tx_exchange()
                    .sender_add_asset_value(s_asset)
                    .sender_value(9)
                    .recipient(receiver.0)
                    .recipient_add_asset_value(r_asset)
                    .fee_strategy(1)
                    .build();

                serialize(tx)
            }

            OpState::ExchangeWithIntermediary => {
                let sender = self.pick_wallet();
                let receiver = self.pick_wallet();
                let intermediary = self.pick_wallet();

                let s_asset = self.pick_asset();
                let r_asset = self.pick_asset();

                let tx = transaction::Builder::new()
                    .keypair(sender.0, sender.1)
                    .tx_exchange_with_intermediary()
                    .intermediary_key_pair(intermediary.0, intermediary.1)
                    .commision(10)
                    .sender_add_asset_value(s_asset)
                    .sender_value(9)
                    .recipient(receiver.0)
                    .recipient_add_asset_value(r_asset)
                    .fee_strategy(1)
                    .build();

                serialize(tx)
            }

            OpState::Mining => {
                let wallet = self.pick_wallet();
                let tx = transaction::Builder::new()
                    .keypair(wallet.0, wallet.1)
                    .tx_mining()
                    .build();

                serialize(tx)
            }

            OpState::TradeAssets => {
                let seller = self.pick_wallet();
                let buyer = self.pick_wallet();
                let asset = self.pick_asset();

                let tx = transaction::Builder::new()
                    .keypair(seller.0, seller.1)
                    .tx_trade_assets()
                    .buyer(buyer.0)
                    .add_asset_value(TradeAsset::from_bundle(asset, 50))
                    .build();

                serialize(tx)
            }

            OpState::TradeAssetsWithIntermediary => {
                let seller = self.pick_wallet();
                let buyer = self.pick_wallet();
                let intermediary = self.pick_wallet();
                let asset = self.pick_asset();

                let tx = transaction::Builder::new()
                    .keypair(seller.0, seller.1)
                    .tx_trade_assets_with_intermediary()
                    .buyer(buyer.0)
                    .intermediary_key_pair(intermediary.0, intermediary.1)
                    .commision(1_0000_0000)
                    .add_asset_value(TradeAsset::from_bundle(asset, 50))
                    .build();

                serialize(tx)
            }

            OpState::Transfer => {
                let sender = self.pick_wallet();
                let receiver = self.pick_wallet();
                let asset = self.pick_asset();
                let coins = self.rng.gen_range(0, asset.amount() + 1);

                let tx = transaction::Builder::new()
                    .keypair(sender.0, sender.1)
                    .tx_transfer()
                    .recipient(receiver.0)
                    .amount(coins as u64)
                    .add_asset_value(asset)
                    .build();

                serialize(tx)
            }
        }
    }

    fn pick_asset(&mut self) -> AssetBundle {
        let asset = self.rng.choose(&self.assets).unwrap();
        AssetBundle::new(asset.id(), self.rng.gen_range(0, asset.amount() + 1))
    }

    fn split_asset(&mut self) -> AssetBundle {
        let asset_ref = self.rng.choose_mut(&mut self.assets).unwrap();
        let amount = self.rng.gen_range(0, asset_ref.amount() + 1);
        let asset = AssetBundle::new(asset_ref.id(), amount);
        let new_amount = match asset_ref.amount() - amount {
            0 => 1,
            amount => amount,
        };
        *asset_ref = AssetBundle::new(asset.id(), new_amount);
        asset
    }

    fn pick_wallet(&mut self) -> Wallet {
        self.rng.choose(&self.wallets).unwrap().clone()
    }
}

fn serialize<S: Serialize>(tx: S) -> String {
    serde_json::to_string(&tx).unwrap()
}

fn main() {
    let mut flooder = Flooder::new();
    let uri = "http://127.0.0.1:8000/api/services/cryptocurrency/v1/transactions";
    const TX_PACK_SIZE: usize = 64;

    let poll = Poll::new().unwrap();
    let mut httpc = Httpc::new(TX_PACK_SIZE);
    let mut events = Events::with_capacity(TX_PACK_SIZE);
    let mut calls = FnvHashMap::default();

    loop {
        for _ in 0..TX_PACK_SIZE {
            let tx = flooder.next_tx();
            let request = Request::builder()
                .method("POST")
                .uri(uri)
                .header("Content-Type", "application/json")
                .body(tx.into())
                .unwrap();

            let call = CallBuilder::new(request)
                .timeout_ms(1)
                .chunked_parse(false)
                .call(&mut httpc, &poll);

            match call {
                Ok(mut call) => {
                    httpc.call_send(&poll, &mut call, None);
                    calls.insert(call.get_ref(), call);
                }
                Err(mio_httpc::Error::Io(error)) => {
                    // Errno values for the 'too many open files' condition.
                    // TODO: these are values from linux, other platforms can
                    //       differ. Check this.
                    const ENFILE: i32 = 23;
                    const EMFILE: i32 = 24;

                    match error.raw_os_error() {
                        Some(EMFILE) | Some(ENFILE) => break,
                        _ => panic!("{}", error),
                    }
                }
                Err(error) => {
                    panic!("{}", error);
                }
            }
        }

        poll.poll(&mut events, Some(Duration::from_millis(1)))
            .unwrap();

        for event in events.iter() {
            let cref = match httpc.event(&event) {
                Some(cref) => cref,
                None => continue,
            };

            let call_done = {
                let call = calls.get_mut(&cref).unwrap();
                let status = httpc.call_recv(&poll, call, None);
                match status {
                    RecvState::Done => true,
                    RecvState::DoneWithBody(_) => true,
                    RecvState::Error(e) => {
                        println!("Error receiving: {}", e);
                        true
                    }
                    _ => false,
                }
            };

            if call_done {
                httpc.call_close(calls.remove(&cref).unwrap());
            }
        }

        for tout in httpc.timeout() {
            let call = calls.remove(&tout).unwrap();
            httpc.call_close(call);
        }

        flooder.op_state.advance();
    }
}
