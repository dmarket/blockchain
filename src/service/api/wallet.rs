extern crate serde;
extern crate serde_json;
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;
extern crate params;
extern crate std;

use exonum::blockchain::Blockchain;
use exonum::crypto::{PublicKey, HexValue};
use exonum::api::{Api, ApiError};
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use router::Router;

use service::schema::wallet::WalletSchema;
use service::wallet::{Wallet, Asset};

use self::params::{Params, FromValue};

const PARAMETER_OFFSET_KEY: &str = "offset";
const PARAMETER_LIMIT_KEY: &str = "limit";

#[derive(Clone)]
pub struct WalletApi {
    pub blockchain: Blockchain,
}
/// Shortcut to get data on wallets.
impl WalletApi {
    fn get_wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        let mut view = self.blockchain.fork();
        let mut schema = WalletSchema { view: &mut view };
        schema.wallet(pub_key)
    }

    fn get_wallets(&self) -> Option<Vec<Wallet>> {
        let mut view = self.blockchain.fork();
        let mut schema = WalletSchema { view: &mut view };
        let idx = schema.wallets();
        let wallets: Vec<Wallet> = idx.values().collect();
        if wallets.is_empty() {
            None
        } else {
            Some(wallets)
        }
    }

    fn get_assets(&self, pub_key: &PublicKey) -> Option<Vec<Asset>> {
        match self.get_wallet(pub_key) {
            Some(wallet) => Some(wallet.assets()),
            None => None,
        }
    }
}

impl Api for WalletApi {
    fn wire(&self, router: &mut Router) {

        fn apply_pagination<T: Clone>(req: &mut Request, elements: &Vec<T>) -> Vec<T> {
            let total_count = elements.len();
            // read url parameters
            let parameters = req.get_ref::<Params>().unwrap();
            let offset_parameter = parameters.get(PARAMETER_OFFSET_KEY);
            let limit_parameter = parameters.get(PARAMETER_LIMIT_KEY);

            // pagination parameters `offset` and `limit` should be considered together
            if offset_parameter.is_some() && limit_parameter.is_some() {
                let offset = FromValue::from_value(offset_parameter.unwrap()).unwrap_or(0);
                let limit = FromValue::from_value(limit_parameter.unwrap()).unwrap_or(total_count);

                // define wallets that need to be send in responce
                let from = std::cmp::min(offset, total_count);
                let to = std::cmp::min(from + limit, total_count);
                return elements[from..to].to_vec();
            }

            elements.clone()
        }

        // Gets status of the wallet corresponding to the public key.
        let self_ = self.clone();
        let wallet_info = move |req: &mut Request| -> IronResult<Response> {
            let path = req.url.path();
            let wallet_key = path.last().unwrap();
            let public_key = PublicKey::from_hex(wallet_key).map_err(ApiError::FromHex)?;
            if let Some(wallet) = self_.get_wallet(&public_key) {
                let res = self_.ok_response(&serde_json::to_value(wallet).unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res =
                    self_.not_found_response(&serde_json::to_value("Wallet not found").unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };

        // Gets status of all wallets.
        let self_ = self.clone();
        let wallets_info = move |req: &mut Request| -> IronResult<Response> {
            if let Some(wallets) = self_.get_wallets() {
                // apply pagination parameters if they exist
                let wallets_to_send = apply_pagination(req, &wallets);
                let wallet_list = serde_json::to_value(&wallets_to_send).unwrap();
                let response_body = json!({
                    "total": wallets.len(),
                    "count": wallets_to_send.len(),
                    "wallets": wallet_list,
                });

                let res = self_.ok_response(&serde_json::to_value(response_body).unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res = self_.not_found_response(
                    &serde_json::to_value("Wallets database is empty")
                        .unwrap(),
                );
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };

        let self_ = self.clone();
        let wallet_assets_info = move |req: &mut Request| -> IronResult<Response> {
            let public_key: PublicKey;
            {
                let wallet_key = req.extensions
                    .get::<Router>()
                    .unwrap()
                    .find("pub_key")
                    .unwrap();
                public_key = PublicKey::from_hex(wallet_key).map_err(ApiError::FromHex)?;
            }
            if let Some(assets) = self_.get_assets(&public_key) {
                // apply pagination parameters if they exist
                let assets_to_send = apply_pagination(req, &assets);
                let assets_list = serde_json::to_value(&assets_to_send).unwrap();
                let response_body = json!({
                    "total": assets.len(),
                    "count": assets_to_send.len(),
                    "assets": assets_list,
                });

                let res = self_.ok_response(&serde_json::to_value(response_body).unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res =
                    self_.not_found_response(&serde_json::to_value("Wallet not found").unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };

        router.get("/wallets", wallets_info, "wallets_info");
        router.get("/wallets/:pub_key", wallet_info, "get_balance");
        router.get(
            "/wallets/:pub_key/assets",
            wallet_assets_info,
            "assets_info",
        );
    }
}
