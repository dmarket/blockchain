extern crate curl;
extern crate dmbc;
extern crate exonum;
extern crate fnv;
extern crate mio;
extern crate mio_httpc;
extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate env_logger;

use dmbc::service::asset::{AssetId, Asset};
use dmbc::service::wallet::Wallet as EvoWallet;
//use dmbc::service::builders::fee;
//use dmbc::service::builders::transaction;
use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;
use node_client::EvoClient;
use serde_json::Value;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;


mod node_client;
mod transaction;
mod asset;

use asset::meta_asset::generate_meta_assets;
use transaction::add_asset::create_add_asset_tx;

const NODE_URL: &str = "http://88.99.64.219:8000";

#[derive(Clone, Debug)]
pub struct Wallet {
    public: PublicKey,
    secret: SecretKey,
    assets: HashMap<AssetId, u32>,
    balance: u64,
}

impl Wallet {
    fn new(public: PublicKey, secret: SecretKey) -> Self {
        let empty_assets: HashMap<AssetId, u32> = HashMap::new();
        Wallet {
            public,
            secret,
            assets: empty_assets,
            balance: 0
        }
    }

    fn generate() -> Self {
        let (pk, sk) = crypto::gen_keypair();
        let empty_assets: HashMap<AssetId, u32> = HashMap::new();
        Wallet {
            public: pk,
            secret: sk,
            assets: empty_assets,
            balance: 0
        }
    }
}
fn data_from_str(s: String) -> (u64, HashMap<AssetId, u32>) {
    let evo_wallet: EvoWallet = serde_json::from_str(&s).unwrap();
    let mut result: HashMap<AssetId, u32> = HashMap::new();
    for asset in evo_wallet.assets() {
        result.insert(asset.id(), asset.amount());
    }
    (evo_wallet.balance(), result)
}

pub struct Bot {
    ec: EvoClient,
    wallets: HashMap<PublicKey, Wallet>,
    pending: HashMap<String, Vec<Wallet>>,
}

impl Bot {
    pub fn new(ec: EvoClient) -> Self {
        Bot{
            ec,
            wallets: HashMap::new(),
            pending: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
//            let wallets = self.ec.wallets();
//            println!("{}", wallets);
//            sleep(Duration::new(1, 0));
//
            let _pk = PublicKey::from_hex("d6ac63f875899a6972bff988da73282ba887fdc59605f126882b9284ec1977c3").unwrap();
//            let wallets = self.ec.wallet(pk);
//            println!("{}", wallets);
//            sleep(Duration::new(1, 0));

            self.send_transaction();

            sleep(Duration::new(2, 0));

            self.check_pending();

            println!("{:?}", self.wallets);
            break;

        }
    }
    pub fn send_transaction(&mut self) {
        let wallet = Wallet::generate();
        let m_assets = generate_meta_assets(wallet.public);
        println!("1{:?}", m_assets);
        let w1 = wallet.clone();
        let tx_string = create_add_asset_tx(wallet, m_assets);
        let tx_hash = self.ec.tx_send(tx_string.as_bytes());
        println!("2 {}", tx_hash);
        let wallets = vec![w1];
        self.pending.insert(tx_hash, wallets);
    }

    pub fn check_pending(&mut self) {
//        let mut tt = .clone();
        for (tx_hash, wallets) in &self.pending {
            match self.ec.tx_status(tx_hash).as_str() {
                "Success"|"Fail" => {
                    for wallet in wallets {
                        let wallet_info = self.ec.wallet(wallet.public);
                        let (balance, assets) = data_from_str(wallet_info);
                        let new_wallet = Wallet {
                            public: wallet.public.clone(),
                            secret: wallet.secret.clone(),
                            assets,
                            balance,
                        };
                        self.wallets.remove(&new_wallet.public);
                        self.wallets.insert(new_wallet.public, new_wallet);
                    }
                },
                _ => {
                    continue
                },
            };

        }
    }
    
}

fn main() {
    env_logger::init();

    let ec = EvoClient::new(NODE_URL.to_string());
    let mut bot = Bot::new(ec);

    bot.run();
}