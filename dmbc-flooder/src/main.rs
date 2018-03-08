extern crate curl;
extern crate dmbc;
extern crate exonum;
extern crate fnv;
extern crate mio;
extern crate mio_httpc;
extern crate rand;
extern crate serde;
extern crate serde_json;

use dmbc::service::asset::{Asset, MetaAsset, AssetId};
use dmbc::service::builders::fee;
use dmbc::service::builders::transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;
use node_client::EvoClient;
use rand::Rng;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

mod node_client;

const SUCCESS: &str = "\"Success\"";
const FAIL: &str = "\"Fail\"";

const MAX_AMOUNT: u32 = 10_000;
const ASSET_NAME: &str = "RAXXLA";

#[derive(Clone, Copy)]
enum OpState {
    CreateWallet = 0,
    AddAssets = 1,
    DelAssets = 2,
//    Exchange = 3,
//    ExchangeWithIntermediary = 4,
//    Mining = 5,
//    TradeAssets = 6,
//    TradeAssetsWithIntermediary = 7,
//    Transfer = 8,
}

impl OpState {
    fn next(self) -> OpState {
        use OpState::*;
        match self {
            CreateWallet => AddAssets,
            AddAssets => DelAssets,
//            DelAssets => Exchange,
//            Exchange => ExchangeWithIntermediary,
//            ExchangeWithIntermediary => Mining,
//            Mining => TradeAssets,
//            TradeAssets => TradeAssetsWithIntermediary,
//            TradeAssetsWithIntermediary => Transfer,
//            Transfer => CreateWallet,
            DelAssets => CreateWallet,
        }
    }

    fn advance(&mut self) {
        *self = self.next();
    }
}
#[derive(Clone)]
struct Wallet {
    public: PublicKey,
    secret: SecretKey,
    assets: HashMap<AssetId, u32>,
    balance: u64,
}

impl Wallet {
    fn new(public: PublicKey, secret: SecretKey) -> Self {
        Wallet{
            public,
            secret,
            assets: vec![],
            balance: 0
        }
    }

    fn generate() -> Self {
        let (pk, sk) = crypto::gen_keypair();
        Wallet{
            public: pk,
            secret: sk,
            assets: vec![],
            balance: 0
        }
    }

    fn add_assets(&mut self, assets: Vec<Asset>) {

    }

    fn del_assets(&mut self, assets: Vec<Asset>) {

    }
}

struct Flooder {
    rng: rand::ThreadRng,
    wallets: HashMap<PublicKey, Wallet>,
    op_state: OpState,
}

impl Flooder {
    fn new() -> Self {
        let pk = PublicKey::from_hex("36a05e418393fb4b23819753f6e6dd51550ce030d53842c43dd1349857a96a61").unwrap();
        let sk = SecretKey::from_hex("d6935ba259dd54b18a2a40fd1c5f8f0544ae8472db48d7726a02aab582d35a6336a05e418393fb4b23819753f6e6dd51550ce030d53842c43dd1349857a96a61").unwrap();
        let mut f = Flooder {
            rng: rand::thread_rng(),
            wallets: HashMap::new(),
            op_state: OpState::CreateWallet,
        };
        f.insert(pk, Wallet::new(pk, sk));
        f
    }

    fn next_tx(&mut self) -> String {
        match self.op_state {
            OpState::CreateWallet => {
                let (pk, sk) = crypto::gen_keypair();
                Wallet::new(pk, sk.clone());
                let tx = transaction::Builder::new()
                    .keypair(pk,  sk)
                    .tx_create_wallet()
                    .build();

                serialize(tx)
            }

            OpState::AddAssets => {
                let amount = self.rng.gen_range(0, MAX_AMOUNT);
                let wallet = self.pick_wallet();
                let fees = fee::Builder::new()
                    .trade(10, 10)
                    .exchange(10, 10)
                    .transfer(10, 10)
                    .build();
                let asset = MetaAsset::new(&wallet.public, ASSET_NAME, amount, fees);
//                self.assets.push(Asset::from_meta_asset(&asset, wallet.public));
                let tx = transaction::Builder::new()
                    .keypair(wallet.public, wallet.secret.clone())
                    .tx_add_assets()
                    .add_asset_value(asset)
                    .build();

                serialize(tx)
            }

            OpState::DelAssets => {
                let wallet = Flooder::pick_wallet(&mut self);
                let asset = self.split_asset();
                let tx = transaction::Builder::new()
                    .keypair(wallet.0, wallet.1)
                    .tx_del_assets()
                    .add_asset_value(asset)
                    .build();

                serialize(tx)
            }
//
//            OpState::Exchange => {
//                let sender = self.pick_wallet();
//                let receiver = self.pick_wallet();
//
//                let s_asset = self.pick_asset();
//                let r_asset = self.pick_asset();
//
//                let tx = transaction::Builder::new()
//                    .keypair(sender.0, sender.1)
//                    .tx_exchange()
//                    .sender_add_asset_value(s_asset)
//                    .sender_value(9)
//                    .recipient(receiver.0)
//                    .recipient_add_asset_value(r_asset)
//                    .fee_strategy(1)
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::ExchangeWithIntermediary => {
//                let sender = self.pick_wallet();
//                let receiver = self.pick_wallet();
//                let intermediary = self.pick_wallet();
//
//                let s_asset = self.pick_asset();
//                let r_asset = self.pick_asset();
//
//                let tx = transaction::Builder::new()
//                    .keypair(sender.0, sender.1)
//                    .tx_exchange_with_intermediary()
//                    .intermediary_key_pair(intermediary.0, intermediary.1)
//                    .commision(10)
//                    .sender_add_asset_value(s_asset)
//                    .sender_value(9)
//                    .recipient(receiver.0)
//                    .recipient_add_asset_value(r_asset)
//                    .fee_strategy(1)
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::Mining => {
//                let wallet = self.pick_wallet();
//                let tx = transaction::Builder::new()
//                    .keypair(wallet.0, wallet.1)
//                    .tx_mining()
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::TradeAssets => {
//                let seller = self.pick_wallet();
//                let buyer = self.pick_wallet();
//                let asset = self.pick_asset();
//
//                let tx = transaction::Builder::new()
//                    .keypair(seller.0, seller.1)
//                    .tx_trade_assets()
//                    .buyer(buyer.0)
//                    .add_asset_value(asset.into_trade_asset(50))
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::TradeAssetsWithIntermediary => {
//                let seller = self.pick_wallet();
//                let buyer = self.pick_wallet();
//                let intermediary = self.pick_wallet();
//                let asset = self.pick_asset();
//
//                let tx = transaction::Builder::new()
//                    .keypair(seller.0, seller.1)
//                    .tx_trade_assets_with_intermediary()
//                    .buyer(buyer.0)
//                    .intermediary_key_pair(intermediary.0, intermediary.1)
//                    .commision(1_0000_0000)
//                    .add_asset_value(asset.into_trade_asset(50))
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::Transfer => {
//                let sender = self.pick_wallet();
//                let receiver = self.pick_wallet();
//                let asset = self.pick_asset();
//                let coins = self.rng.gen_range(0, asset.amount() + 1);
//
//                let tx = transaction::Builder::new()
//                    .keypair(sender.0, sender.1)
//                    .tx_transfer()
//                    .recipient(receiver.0)
//                    .amount(coins as u64)
//                    .add_asset_value(asset)
//                    .build();
//
//                serialize(tx)
//            }
        }
    }

//    fn pick_asset(&mut self) -> Asset {
//        let asset = self.rng.choose(&self.assets).unwrap();
//        Asset::new(asset.id(), self.rng.gen_range(0, asset.amount() + 1))
//    }

    fn split_asset(&mut self, &) -> Asset {
        let asset_ref = self.rng.choose_mut(&mut self.assets).unwrap();
        let amount = self.rng.gen_range(0, asset_ref.amount() + 1);
        let asset = Asset::new(asset_ref.id(), amount);
        let new_amount = match asset_ref.amount() - amount {
            0 => 1,
            amount => amount,
        };
        *asset_ref = Asset::new(asset.id(), new_amount);
        asset
    }

    fn pick_wallet(&mut self) -> &mut Wallet {
        if self.rng.gen() { // random bool
            let wallet = Wallet::generate();
            self.wallets.push(wallet.clone());
            self.wallets.last_mut().unwrap()
        } else {
            self.rng.choose_mut(&mut self.wallets).unwrap()
        }
    }
}

fn serialize<S: Serialize>(tx: S) -> String {
    serde_json::to_string(&tx).unwrap()
}

fn main() {

    let mut flooder = Flooder::new();
    let ec = EvoClient::new("http://88.99.64.219:8000".to_string());
    let mut txs: HashMap<String,u8> = HashMap::new();


    let wallets = ec.wallets();
    println!("{}", wallets);

    let tx = flooder.next_tx();
    let create_wallet_tx = tx.as_bytes();
    let tx_response = ec.tx_send(create_wallet_tx);
    println!("{}", tx_response);
    txs.insert(tx_response, 0);

    flooder.op_state.advance();
    sleep(Duration::new(1, 0));

    let tx = flooder.next_tx();
    let add_asset_tx = tx.as_bytes();
    let tx_response = ec.tx_send(add_asset_tx);
    println!("{}", tx_response);
    txs.insert(tx_response, 0);

    flooder.op_state.advance();
    sleep(Duration::new(1, 0));

    for (tx_response, status) in &mut txs {
        if *status != 0 {
            continue
        }
        println!("Calling {}: {}", tx_response, status);
        let v: Value = serde_json::from_str(&tx_response).unwrap();
        let hash = v["tx_hash"].as_str().unwrap();
        let hash = hash.trim_matches('"').to_string();
        println!("{:?}", hash);


        let response = ec.tx_status(&hash);
        let v: Value = serde_json::from_str(&response).unwrap();
        println!("{:?}",v);

        match v["tx_status"].to_string().as_ref() {
            SUCCESS => *status = 1,
            FAIL => *status = 0,
            _ => panic!("Fack"),
        }
    }

    println!("{:?}", txs);

}

