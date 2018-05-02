extern crate serde_json;

use hyper::status::StatusCode;
use iron_test::{request, response};
use iron::Handler;
use iron::headers::{ContentType, Headers};

use exonum::crypto::{self, PublicKey};
use exonum::messages::Message;
use exonum_testkit::{TestKit as ExonumTestKit, TestKitBuilder, TestKitApi as ExonumTestKitApi};
use exonum::encoding::serialize::FromHex;
use exonum::encoding::serialize::reexport::{Serialize, Deserialize};

use dmbc::currency::configuration::Configuration;
use dmbc::currency::{SERVICE_NAME, Service};
use dmbc::currency::wallet::{self, Wallet};
use dmbc::currency::assets::{self, AssetBundle, AssetInfo, Fees, MetaAsset, AssetId};
use dmbc::currency::transactions::builders::fee;
use dmbc::currency::api::transaction::{TxPostResponse, StatusResponse};
use dmbc::currency::api::fees::FeesResponse;
use dmbc::currency::api::wallet as wallet_api;
use dmbc::currency::api::wallet::{WalletResponse, WalletAssetsResponse,
                                    WalletInfo, ExtendedAsset};
use dmbc::currency::configuration::GENESIS_WALLET_PUB_KEY;

pub trait DmbcTestKit {
    fn default() -> Self;

    fn add_assets(&mut self, pub_key: &PublicKey, assets: Vec<(AssetBundle, AssetInfo)>);

    fn set_configuration(&mut self, configuration: Configuration);

    fn fetch_wallet(&mut self, pub_key: &PublicKey) -> Wallet;

    fn store_wallet(&mut self, pub_key: &PublicKey, wallet: Wallet);

    fn fetch_asset_info(&mut self, id: &AssetId) -> Option<AssetInfo>;

    fn store_asset_info(&mut self, id: &AssetId, info: AssetInfo);
}

impl DmbcTestKit for ExonumTestKit {
    fn default() -> Self {
        TestKitBuilder::validator()
            .with_validators(4)
            .with_service(Service::new())
            .create()
    }

    fn add_assets(&mut self, pub_key: &PublicKey, assets: Vec<(AssetBundle, AssetInfo)>) {
        let blockchain = self.blockchain_mut();
        let mut fork = blockchain.fork();
        let mut wallet = wallet::Schema(&fork).fetch(&pub_key);

        for (asset, info) in assets {
            wallet.add_assets(vec![asset.clone()]);
            assets::Schema(&mut fork).store(&asset.id(), info);
        }
        wallet::Schema(&mut fork).store(&pub_key, Wallet::new(wallet.balance(), wallet.assets()));

        assert!(blockchain.merge(fork.into_patch()).is_ok());
    }

    fn set_configuration(&mut self, configuration: Configuration) {
        let cfg_change_height = self.height().next();
        let proposal = {
            let mut cfg = self.configuration_change_proposal();
            cfg.set_service_config(&SERVICE_NAME, configuration);
            cfg.set_actual_from(cfg_change_height);
            cfg
        };
        self.commit_configuration_change(proposal);
        self.create_block();
    }

    fn fetch_wallet(&mut self, pub_key: &PublicKey) -> Wallet {
        let blockchain = self.blockchain_mut();
        let fork = blockchain.fork();
        wallet::Schema(&fork).fetch(&pub_key)
    }

    fn store_wallet(&mut self, pub_key: &PublicKey, wallet: Wallet) {
        let blockchain = self.blockchain_mut();
        let mut fork = blockchain.fork();

        let mut existing_wallet = wallet::Schema(&fork).fetch(&pub_key);
        existing_wallet.add_assets(wallet.assets());
        let updated_balance = existing_wallet.balance() + wallet.balance();
        wallet::Schema(&mut fork).store(&pub_key, Wallet::new(updated_balance, existing_wallet.assets()));

        assert!(blockchain.merge(fork.into_patch()).is_ok());
    }

    fn fetch_asset_info(&mut self, id: &AssetId) -> Option<AssetInfo> {
        let blockchain = self.blockchain_mut();
        let fork = blockchain.fork();
        assets::Schema(&fork).fetch(&id)
    }

    fn store_asset_info(&mut self, id: &AssetId, info: AssetInfo) {
        let blockchain = self.blockchain_mut();
        let mut fork = blockchain.fork();

        assets::Schema(&mut fork).store(id, info);

        assert!(blockchain.merge(fork.into_patch()).is_ok());
    }
}


pub trait DmbcTestKitApi {
    fn get_internal_with_status<H, D>(handler: &H, endpoint: &str) -> (StatusCode, D) 
    where H: Handler, for <'de> D: Deserialize<'de>;

    fn get_with_status<D>(&self, endpoint: &str) -> (StatusCode, D)
    where for<'de> D: Deserialize<'de>;

    fn post_with_status<T, D>(&self, endpoint: &str, transaction: &T) -> (StatusCode, D)
    where T: Serialize, for<'de> D: Deserialize<'de>;

    fn post_raw_with_status<D>(&self, endpoint: &str, body: &str) -> (StatusCode, D)
    where for <'de> D: Deserialize<'de>;

    fn post_raw_with_status2<D>(&self, endpoint: &str, headers: Headers, body: &str) -> (StatusCode, D)
    where for <'de> D: Deserialize<'de>;

    fn post_tx<T>(&self, tx: &T) -> (StatusCode, TxPostResponse)
    where T: Message + Serialize; 

    fn get_tx_status<T>(&self, transaction: &T) -> (StatusCode, StatusResponse)
    where T: Message + Serialize;

    fn post_fee<T>(&self, tx: &T) -> (StatusCode, FeesResponse)
    where T: Message + Serialize; 

    fn get_wallet(&self, public_key: &PublicKey) -> WalletInfo;

    fn get_wallet_assets(&self, public_key: &PublicKey) -> Vec<ExtendedAsset>;
}

impl DmbcTestKitApi for ExonumTestKitApi {

    fn get_internal_with_status<H, D>(handler: &H, endpoint: &str) -> (StatusCode, D) 
    where H: Handler, for <'de> D: Deserialize<'de>
    {
        let url = format!("http://localhost:3000/api/services/{}/{}", SERVICE_NAME, endpoint);
        let response = request::get(&url, Headers::new(), handler).unwrap();
        let status = response.status.unwrap();
        let body = response::extract_body_to_string(response);
        (status, serde_json::from_str(&body).unwrap())
    }

    fn get_with_status<D>(&self, endpoint: &str) -> (StatusCode, D)
    where for <'de> D: Deserialize<'de>
    {
        ExonumTestKitApi::get_internal_with_status(
            self.public_handler(),
            endpoint, 
        )
    }

    fn post_raw_with_status<D>(&self, endpoint: &str, body: &str) -> (StatusCode, D)
    where for <'de> D: Deserialize<'de> 
    {
        let url = format!("http://localhost:3000/api/services/{}/{}", SERVICE_NAME, endpoint);
        let response = request::post(
            &url,
            {
                let mut headers = Headers::new();
                headers.set(ContentType::json());
                headers
            },
            &body,
            self.public_handler(),
        ).expect("Cannot send data");
        let status = response.status.unwrap();
        let body = response::extract_body_to_string(response);
        (status, serde_json::from_str(&body).unwrap())
    }

    fn post_raw_with_status2<D>(&self, endpoint: &str, headers: Headers, body: &str) -> (StatusCode, D)
    where for <'de> D: Deserialize<'de>
    {
        let url = format!("http://localhost:3000/api/services/{}/{}", SERVICE_NAME, endpoint);
        let response = request::post(
            &url,
            headers,
            &body,
            self.public_handler(),
        ).expect("Cannot send data");
        let status = response.status.unwrap();
        let body = response::extract_body_to_string(response);
        (status, serde_json::from_str(&body).unwrap())
    }

    fn post_with_status<T, D>(&self, endpoint: &str, transaction: &T) -> (StatusCode, D)
    where
        T: Serialize,
        for<'de> D: Deserialize<'de>,
    {
        self.post_raw_with_status(
            endpoint,
            &serde_json::to_string(&transaction).expect("Cannot serialize data to JSON"),
        )
    }

    fn post_tx<T>(&self, tx: &T) -> (StatusCode, TxPostResponse)
    where T: Message + Serialize 
    {
        self.post_with_status("v1/transactions", &tx)
    }

    fn get_tx_status<T>(&self, transaction: &T) -> (StatusCode, StatusResponse)
    where T: Message + Serialize 
    {   
        let endpoint = &format!("/v1/transactions/{}", transaction.hash().to_string());
        self.get_with_status(endpoint)
    }

    fn post_fee<T>(&self, tx: &T) -> (StatusCode, FeesResponse)
    where T: Message + Serialize
    {
        self.post_with_status("/v1/fees/transactions", &tx)
    }

    fn get_wallet(&self, public_key: &PublicKey) -> WalletInfo {
        let (status, response): (StatusCode, WalletResponse) = self.get_with_status(
            &format!("/v1/wallets/{}", public_key.to_string())
        );
        
        assert_eq!(status, StatusCode::Ok);
        assert!(response.is_ok());
        response.unwrap()
    }

    fn get_wallet_assets(&self, public_key: &PublicKey) -> Vec<ExtendedAsset> {
        let (status, response): (StatusCode, WalletAssetsResponse) = self.get_with_status(
            &format!(
                "/v1/wallets/{}/assets?{}=true",
                public_key.to_string(),
                wallet_api::PARAMETER_META_DATA_KEY,
            )
        );

        assert_eq!(status, StatusCode::Ok);
        assert!(response.is_ok());
        response.unwrap().assets
    }
}

pub fn asset_fees(t: u64, r: u64) -> Fees {
    fee::Builder::new()
        .trade(t, r)
        .exchange(t, r)
        .transfer(t, r)
        .build()
}

pub fn create_asset(
    meta_data: &str, 
    units: u64, 
    fees: Fees, 
    creator: &PublicKey
) -> (AssetBundle, AssetInfo) 
{
    let (receiver, _) = crypto::gen_keypair();
    let meta_asset = MetaAsset::new(&receiver, meta_data, units, fees);
    let id = AssetId::from_data(meta_data, creator);
    let asset = meta_asset.to_bundle(id);
    let info = meta_asset.to_info(creator, &crypto::hash(meta_data.as_bytes()));

    (asset, info)
}

pub fn create_asset2(
    meta_data: &str, 
    units: u64, 
    fees: Fees, 
    creator: &PublicKey,
    origin: &crypto::Hash
) -> (AssetBundle, AssetInfo) 
{
    let (receiver, _) = crypto::gen_keypair();
    let meta_asset = MetaAsset::new(&receiver, meta_data, units, fees);
    let id = AssetId::from_data(meta_data, creator);
    let asset = meta_asset.to_bundle(id);
    let info = meta_asset.to_info(creator, origin);

    (asset, info)
}


pub fn default_genesis_key() -> PublicKey {
    PublicKey::from_hex(GENESIS_WALLET_PUB_KEY).unwrap()
}

pub struct DmbcTestApiBuilder {
    configuration: Option<Configuration>,
    wallets: Vec<(PublicKey, Wallet)>,
    assets: Vec<(PublicKey, (AssetBundle, AssetInfo))>,
    infos: Vec<(AssetId, AssetInfo)>,
}

impl DmbcTestApiBuilder {
    pub fn new() -> Self {
        DmbcTestApiBuilder {
            configuration: None,
            wallets: Vec::new(),
            assets: Vec::new(),
            infos: Vec::new(),
        }
    }

    pub fn with_configuration(self, configuration: Configuration) -> Self {
        DmbcTestApiBuilder {
            configuration: Some(configuration),
            ..self
        }
    }

    pub fn add_asset_to_wallet(mut self, public_key: &PublicKey, asset: (AssetBundle, AssetInfo)) -> Self {
        self.assets.push((*public_key, asset));
        self
    }

    pub fn add_asset_info(mut self, id: &AssetId, info: AssetInfo) -> Self {
        self.infos.push((*id, info));
        self
    }

    pub fn add_wallet_value(mut self, public_key: &PublicKey, wallet: Wallet) -> Self {
        self.wallets.push((*public_key, wallet));
        self
    }

    pub fn create(self) -> ExonumTestKit {
        let mut testkit = ExonumTestKit::default();
        if let Some(configuration) = self.configuration {
            testkit.set_configuration(configuration);
        }

        for (key, wallet) in self.wallets {
            testkit.store_wallet(&key, wallet);
        }

        for (id, info) in self.infos {
            testkit.store_asset_info(&id, info);
        }
        
        for (key, asset) in self.assets {
            testkit.add_assets(&key, vec![asset])
        }

        testkit
    }
}