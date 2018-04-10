extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate params;
extern crate router;
extern crate serde;
extern crate serde_json;
extern crate std;

use std::collections::HashMap;

use exonum::api::{Api, ApiError};
use exonum::blockchain::Blockchain;
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use router::Router;
use prometheus::Counter;

use currency::api::ServiceApi;
use currency::assets::AssetBundle;
use currency::wallet;
use currency::wallet::Wallet;

#[derive(Clone)]
pub struct WalletApi {
    pub blockchain: Blockchain,
}
#[derive(Serialize)]
struct WalletInfo {
    balance: u64,
    count_assets: u64,
}

impl WalletApi {
    fn wallet(&self, pub_key: &PublicKey) -> Wallet {
        let view = &mut self.blockchain.fork();
        wallet::Schema(view).fetch(pub_key)
    }

    fn wallets(&self) -> HashMap<PublicKey, WalletInfo> {
        let view = &mut self.blockchain.fork();
        let index = wallet::Schema(view).index();
        let mut result: HashMap<PublicKey, WalletInfo> = HashMap::new();
        for v in index.iter() {

            let wi = WalletInfo {
                balance: v.1.balance(),
                count_assets: v.1.assets().len() as u64,
            };
            result.insert(v.0, wi);
        }

        result
    }

    fn pagination_wallets(&self, offset: u64, limit: u64) -> (HashMap<PublicKey, WalletInfo>, u64, u64){
        let view = &mut self.blockchain.fork();
        let idx = wallet::Schema(view).index();
        let mut total:u64 = 0;
        let mut count:u64 = 0;
        let mut result: HashMap<PublicKey, WalletInfo> = HashMap::new();
        for v in idx.iter() {
            if total < offset || total >= offset + limit {
                total+=1;
                continue;
            }
            let wi = WalletInfo {
                balance: v.1.balance(),
                count_assets: v.1.assets().len() as u64,
            };
            result.insert(v.0, wi);
            count+=1;
            total+=1;
        }

        (result, total, count)
    }

    fn wallets_balance(&self) -> Vec<PublicKey> {
        let view = &mut self.blockchain.fork();
        let index = wallet::Schema(view).index();
        let wallets = index.into_iter().map(|v|{ v.0 }).collect();
        wallets
    }

    fn assets(&self, pub_key: &PublicKey) -> Vec<AssetBundle> {
        self.wallet(pub_key).assets()
    }
}

lazy_static! {
    static ref LIST_REQUESTS: Counter = register_counter!("dmbc_wallet_api_list_requests_total", "Wallet list requests.").unwrap();
    static ref LIST_RESPONSES: Counter = register_counter!("dmbc_wallet_api_list_responses_total", "Wallet list responses.").unwrap();
    static ref BALANCE_REQUESTS: Counter = register_counter!("dmbc_wallet_api_balance_requests_total", "Balance requests.").unwrap();
    static ref BALANCE_RESPONSES: Counter = register_counter!("dmbc_wallet_api_balance_responses_total", "Balance responses.").unwrap();
    static ref ASSETS_REQUESTS: Counter = register_counter!("dmbc_wallet_api_assets_requests_total", "Wallet asset list requests.").unwrap();
    static ref ASSETS_RESPONSES: Counter = register_counter!("dmbc_wallet_api_assets_responses_total", "Wallet asset list responses.").unwrap();
}

impl Api for WalletApi {
    fn wire(&self, router: &mut Router) {
        // Gets status of the wallet corresponding to the public key.
        let self_ = self.clone();
        let wallet_info = move |req: &mut Request| -> IronResult<Response> {
            BALANCE_REQUESTS.inc();

            let path = req.url.path();
            let wallet_key = path.last().unwrap();
            let public_key = PublicKey::from_hex(wallet_key).map_err(ApiError::FromHex)?;
            let wallet = self_.wallet(&public_key);
            let res = self_.ok_response(&serde_json::to_value(wallet).unwrap());
            let mut res = res.unwrap();
            res.headers.set(AccessControlAllowOrigin::Any);

            BALANCE_RESPONSES.inc();

            Ok(res)
        };

        // Gets status of all wallets.
        let self_ = self.clone();
        let wallets_info = move |req: &mut Request| -> IronResult<Response> {
            LIST_REQUESTS.inc();

            let (offset, limit) = ServiceApi::pagination_params(req);
            let (wallets, total, count) = self_.pagination_wallets(offset, limit);
            let response_body = json!({
                "total": total,
                "count": count,
                "wallets": wallets,
            });

            let res = self_.ok_response(&serde_json::to_value(response_body).unwrap());
            let mut res = res.unwrap();
            res.headers.set(AccessControlAllowOrigin::Any);

            LIST_RESPONSES.inc();

            Ok(res)
        };

        let self_ = self.clone();
        let wallet_assets_info = move |req: &mut Request| -> IronResult<Response> {
            ASSETS_REQUESTS.inc();

            let public_key = {
                let wallet_key = req.extensions
                    .get::<Router>()
                    .unwrap()
                    .find("pub_key")
                    .unwrap();
                PublicKey::from_hex(wallet_key).map_err(ApiError::FromHex)?
            };
            let assets = self_.assets(&public_key);
            // apply pagination parameters if they exist
            let assets_to_send = ServiceApi::apply_pagination(req, &assets);
            let assets_list = serde_json::to_value(&assets_to_send).unwrap();
            let response_body = json!({
                "total": assets.len(),
                "count": assets_to_send.len(),
                "assets": assets_list,
            });

            let res = self_.ok_response(&serde_json::to_value(response_body).unwrap());
            let mut res = res.unwrap();
            res.headers.set(AccessControlAllowOrigin::Any);

            ASSETS_RESPONSES.inc();

            Ok(res)
        };

        router.get("/v1/wallets", wallets_info, "wallets_info");
        router.get("/v1/wallets/:pub_key", wallet_info, "get_balance");
        router.get(
            "/v1/wallets/:pub_key/assets",
            wallet_assets_info,
            "assets_info",
        );
    }
}
