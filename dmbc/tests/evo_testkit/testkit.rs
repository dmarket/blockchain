
extern crate serde_json;

use mount::Mount;
use hyper::status::StatusCode;
use iron_test::{request, response};
use iron::headers::{ContentType, Headers};

use exonum::crypto::{self, PublicKey};
use exonum::storage::Error;
use exonum::messages::Message;
use exonum_testkit::{TestKit as ExonumTestKit, TestKitBuilder, TestKitApi as ExonumTestKitApi};
use exonum::encoding::serialize::reexport::{Serialize, Deserialize};

use dmbc::currency::configuration::Configuration;
use dmbc::currency::{SERVICE_NAME, Service};
use dmbc::currency::api::wallet::WalletResponse;
use dmbc::currency::wallet::{self, Wallet};
use dmbc::currency::assets::{self, AssetBundle, AssetInfo};
use dmbc::currency::api::transaction::{TxPostResponse, TransactionResponse};
use dmbc::currency::api::fees::FeesResponse;

pub trait EvoTestKit {
    fn create_wallet(&mut self, pub_key: &PublicKey, balance: u64) -> Result<Wallet, Error>;

    fn add_assets(&mut self, pub_key: &PublicKey, assets: Vec<AssetBundle>, infos: Vec<AssetInfo>) -> Result<(), Error>;

    fn set_configuration(&mut self, configuration: Configuration);
}

impl EvoTestKit for ExonumTestKit {
    fn create_wallet(&mut self, pub_key: &PublicKey, balance: u64) -> Result<Wallet, Error> {
        let blockchain = self.blockchain_mut();
        let mut fork = blockchain.fork();
        let wallet = Wallet::new(balance, vec![]);
        wallet::Schema(&mut fork).store(&pub_key, wallet.clone());

        blockchain.merge(fork.into_patch())?;

        Ok(wallet)
    }

    fn add_assets(&mut self, pub_key: &PublicKey, assets: Vec<AssetBundle>, infos: Vec<AssetInfo>) -> Result<(), Error> {
        let blockchain = self.blockchain_mut();
        let mut fork = blockchain.fork();
        let wallet = wallet::Schema(&fork).fetch(&pub_key);
        let wallet = Wallet::new(wallet.balance(), assets.clone());
        wallet::Schema(&mut fork).store(&pub_key, wallet);

        for (asset, info) in assets.into_iter().zip(infos.into_iter()) {
            assets::Schema(&mut fork).store(&asset.id(), info);
        }

        blockchain.merge(fork.into_patch())?;

        Ok(())
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
}


pub trait EvoTestKitApi {
    fn get_internal_with_status<D>(mount: &Mount, url: &str) -> (StatusCode, D) 
    where for <'de> D: Deserialize<'de>;

    fn get_with_status<D>(&self, endpoint: &str) -> (StatusCode, D)
    where for<'de> D: Deserialize<'de>;

    fn post_internal_with_status<T, D>(mount: &Mount, endpoint: &str, data: &T) -> (StatusCode, D) 
    where T: Serialize, for<'de> D: Deserialize<'de>;

    fn post_with_status<T, D>(&self, endpoint: &str, transaction: &T) -> (StatusCode, D)
    where T: Serialize, for<'de> D: Deserialize<'de>;

    fn wallet(&self, pub_key: &PublicKey) -> Wallet;

    fn post_tx<T>(&self, tx: &T) 
    where T: Message + Serialize; 

    fn post_fee<T>(&self, tx: &T) -> (StatusCode, FeesResponse)
    where T: Message + Serialize; 
}

impl EvoTestKitApi for ExonumTestKitApi {

    fn get_internal_with_status<D>(mount: &Mount, url: &str) -> (StatusCode, D) 
    where for <'de> D: Deserialize<'de>
    {
        let url = format!("http://localhost:3000/{}", url);
        let response = request::get(&url, Headers::new(), mount).unwrap();
        let status = response.status.unwrap();
        let body = response::extract_body_to_string(response);
        (status, serde_json::from_str(&body).unwrap())
    }

    fn get_with_status<D>(&self, endpoint: &str) -> (StatusCode, D)
    where for <'de> D: Deserialize<'de>
    {
        ExonumTestKitApi::get_internal_with_status(
            self.public_mount(),
            &format!("api/services/{}/{}", SERVICE_NAME, endpoint), 
        )
    } 

    fn post_internal_with_status<T, D>(mount: &Mount, endpoint: &str, data: &T) -> (StatusCode, D) 
    where T: Serialize, for<'de> D: Deserialize<'de>
    {
        let url = format!("http://localhost:3000/{}", endpoint);
        let response = request::post(
            &url,
            {
                let mut headers = Headers::new();
                headers.set(ContentType::json());
                headers
            },
            &serde_json::to_string(&data).expect("Cannot serialize data to JSON"),
            mount,
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
        ExonumTestKitApi::post_internal_with_status(
            &self.public_mount(),
            &format!("api/services/{}/{}", SERVICE_NAME, endpoint),
            transaction,
        )
    }

    fn wallet(&self, pub_key: &PublicKey) -> Wallet {
        let (status, response): (StatusCode, WalletResponse) = self.get_with_status(
            &format!("v1/wallets/{}", pub_key.to_string()),
        );

        assert_eq!(status, StatusCode::Ok);
        response.unwrap()
    }

    fn post_tx<T>(&self, tx: &T) 
    where T: Message + Serialize 
    {
        let (status, response): (StatusCode, TxPostResponse) = self.post_with_status(
            "v1/transactions", &tx
        );

        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response, Ok(Ok(TransactionResponse { tx_hash: tx.hash() })));
    }

    fn post_fee<T>(&self, tx: &T) -> (StatusCode, FeesResponse)
    where T: Message + Serialize
    {
        self.post_with_status("/v1/fees/transactions", &tx)
    }
}

#[test]
fn name() {
    let mut testkit = TestKitBuilder::validator()
        .with_validators(4)
        .with_service(Service::new())
        .create();
    let api = testkit.api();

    let (pub_key, _) = crypto::gen_keypair();

    let wallet = testkit.create_wallet(&pub_key, 100000).unwrap();

    let equivalent = api.wallet(&pub_key);
    assert_eq!(wallet, equivalent);
}