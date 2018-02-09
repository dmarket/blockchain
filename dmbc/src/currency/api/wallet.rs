extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate params;
extern crate router;
extern crate serde;
extern crate serde_json;
extern crate std;

use exonum::api::{Api, ApiError};
use exonum::blockchain::Blockchain;
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use router::Router;

use currency::ServiceApi;
use currency::asset::Asset;
use currency::schema::wallet::WalletSchema;
use currency::wallet::Wallet;

#[derive(Clone)]
pub struct WalletApi {
    pub blockchain: Blockchain,
}
/// Shortcut to get data on wallets.
impl WalletApi {
    fn get_wallet(&self, pub_key: &PublicKey) -> Wallet {
        let mut view = self.blockchain.fork();
        WalletSchema::map(&mut view, |mut schema| schema.wallet(pub_key))
    }

    fn get_wallets(&self) -> Option<Vec<Wallet>> {
        let mut view = self.blockchain.fork();
        WalletSchema::map(&mut view, |mut schema| {
            let idx = schema.wallets();
            let wallets: Vec<Wallet> = idx.values().collect();
            if wallets.is_empty() {
                None
            } else {
                Some(wallets)
            }
        })
    }

    fn get_assets(&self, pub_key: &PublicKey) -> Vec<Asset> {
        let wallet = self.get_wallet(pub_key);
        wallet.assets()
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
            let wallet = self_.get_wallet(&public_key);
            let res = self_.ok_response(&serde_json::to_value(wallet).unwrap());
            let mut res = res.unwrap();
            res.headers.set(AccessControlAllowOrigin::Any);
            Ok(res)
        };

        // Gets status of all wallets.
        let self_ = self.clone();
        let wallets_info = move |req: &mut Request| -> IronResult<Response> {
            if let Some(wallets) = self_.get_wallets() {
                // apply pagination parameters if they exist
                let wallets_to_send = ServiceApi::apply_pagination(req, &wallets);
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
                let res = self_
                    .not_found_response(&serde_json::to_value("Wallets database is empty").unwrap());
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
            let assets = self_.get_assets(&public_key);
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
            Ok(res)
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
