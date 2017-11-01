extern crate serde;
extern crate serde_json;
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;

use exonum::blockchain::Blockchain;
use exonum::crypto::{PublicKey, HexValue};
use exonum::api::{Api, ApiError};
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use router::Router;

use service::schema::wallet::WalletSchema;
use service::wallet::Wallet;

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
        let wallets_info = move |_: &mut Request| -> IronResult<Response> {
            if let Some(wallets) = self_.get_wallets() {
                let res = self_.ok_response(&serde_json::to_value(wallets).unwrap());
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
