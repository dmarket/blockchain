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
use service::wallet::Wallet;

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
}

impl Api for WalletApi {
    fn wire(&self, router: &mut Router) {

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
            use self::params::{Params, FromValue};

            if let Some(wallets) = self_.get_wallets() {
                // define default values for request parameters
                let total_wallet_count = wallets.len();
                let mut wallets_to_send = wallets.clone();

                // read url parameters
                let parameters = req.get_ref::<Params>().unwrap();
                let offset_parameter = parameters.get(PARAMETER_OFFSET_KEY);
                let limit_parameter = parameters.get(PARAMETER_LIMIT_KEY);

                // pagination parameters `offset` and `limit` should be considered together
                if offset_parameter.is_some() && limit_parameter.is_some() {
                    let offset = FromValue::from_value(offset_parameter.unwrap()).unwrap_or(0);
                    let limit = FromValue::from_value(limit_parameter.unwrap()).unwrap_or(
                        total_wallet_count,
                    );

                    // define wallets that need to be send in responce
                    let from = std::cmp::min(offset, total_wallet_count);
                    let to = std::cmp::min(from + limit, total_wallet_count);
                    wallets_to_send = wallets_to_send[from..to].to_vec();
                }

                let wallet_list = serde_json::to_value(&wallets_to_send).unwrap();
                let response_body = json!({
                    "total": total_wallet_count,
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

        router.get("/wallets", wallets_info, "wallets_info");
        router.get("/wallet/:pub_key", wallet_info, "get_balance");
    }
}
