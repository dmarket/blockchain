use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey, Hash};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};
use exonum::encoding::serialize::reexport::Serialize;
use exonum::messages::Message;

use dmbc::currency::Service;
use dmbc::currency::SERVICE_NAME;
use dmbc::currency::assets::AssetId;
use dmbc::currency::wallet::Wallet;
use dmbc::currency::api::transaction::{TransactionResponse, StatusResponse, TxPostResponse};
use dmbc::currency::api::asset::AssetResponse;
use dmbc::currency::api::wallet::WalletResponse;
use dmbc::currency::configuration::{Configuration, TransactionFees};
use dmbc::currency::transactions::builders::transaction;
use dmbc::currency::assets::{Fees, MetaAsset};

pub mod mine;
pub mod add_assets;
pub mod delete_assets;
pub mod transfer;
pub mod exchange;
pub mod exchange_intermediary;
pub mod trade;
pub mod trade_intermediary;

pub const DMC_1:u64 = 1_00_000_000;

pub fn init_testkit() -> TestKit {
    TestKitBuilder::validator()
        .with_validators(4)
        .with_service(Service::new())
        .create()
}

pub fn get_wallet(api: &TestKitApi, pub_key: &PublicKey) -> Wallet {
    let response: WalletResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/wallets/{}", pub_key.to_string()),
    );

    response.unwrap()
}

pub fn get_status(api: &TestKitApi, tx_hash: &Hash) -> StatusResponse {
    api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/transactions/{}", tx_hash.to_string()),
    )
}

pub fn get_asset_info(api: &TestKitApi, asset_id: &AssetId) -> AssetResponse {
    api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("/v1/assets/{}", asset_id.to_string()),
    )
}

pub fn post_tx<T>(api: &TestKitApi, tx: &T)
    where T:Message + Serialize
{
    let tx_response:TxPostResponse = api.post(
        ApiKind::Service(SERVICE_NAME),
        "v1/transactions",
        &tx
    );

    assert_eq!(tx_response, Ok(Ok(TransactionResponse{tx_hash:tx.hash()})));
}

pub fn set_configuration(testkit: &mut TestKit, fees: TransactionFees) {
    let configuration = Configuration::new(fees);
    let cfg_change_height = testkit.height().next();
    let proposal = {
        let mut cfg = testkit.configuration_change_proposal();
        cfg.set_service_config(&SERVICE_NAME, configuration.clone());
        cfg.set_actual_from(cfg_change_height);
        cfg
    };
    testkit.commit_configuration_change(proposal);
    testkit.create_block();
}

fn mine_wallet(testkit: &mut TestKit,) -> (PublicKey, SecretKey) {
    let (pk, sk) = crypto::gen_keypair();

    let mine_1_dmc = transaction::Builder::new()
        .keypair(pk, sk.clone())
        .tx_mine()
        .build();

    post_tx(&testkit.api(), &mine_1_dmc);
    testkit.create_block();

    (pk, sk)
}

pub struct WalletMiner {
    public_key: PublicKey,
    secret_key: SecretKey,
    assets: Vec<MetaAsset>,
}

impl WalletMiner {
    fn new() -> Self {
        let (public_key, secret_key) = crypto::gen_keypair();
        WalletMiner {
            public_key,
            secret_key,
            assets: Vec::new(),
        }
    }

    pub fn add_asset(self, name: &str, count: u64, fees: Fees) -> Self {
        let asset = MetaAsset::new(&self.public_key, name, count, fees);
        self.add_asset_value(asset)
    }

    pub fn add_asset_value(mut self, asset: MetaAsset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn mine_empty(self) -> (PublicKey, SecretKey) {
        (self.public_key, self.secret_key)
    }

    pub fn mine(self, testkit: &mut TestKit) -> (PublicKey, SecretKey) {
        let mine_1_dmc = transaction::Builder::new()
            .keypair(self.public_key, self.secret_key.clone())
            .tx_mine()
            .build();

        post_tx(&testkit.api(), &mine_1_dmc);
        testkit.create_block();

        if !self.assets.is_empty() {
            let mut tx_add_assets_builder = transaction::Builder::new()
            .keypair(self.public_key, self.secret_key.clone())
            .tx_add_assets()
            .seed(85);

            for asset in self.assets {
                tx_add_assets_builder = tx_add_assets_builder.add_asset_value(asset);
            }

            let tx_add_assets = tx_add_assets_builder.build();

            post_tx(&testkit.api(), &tx_add_assets);
            testkit.create_block();
        }

        (self.public_key, self.secret_key)
    }

    
}