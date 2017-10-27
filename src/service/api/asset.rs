extern crate serde;
extern crate serde_json;
extern crate exonum;
extern crate router;
extern crate bodyparser;
extern crate iron;

use exonum::blockchain::Blockchain;
use exonum::crypto::PublicKey;
use exonum::api::Api;
use iron::headers::{AccessControlAllowOrigin};
use iron::prelude::*;
use router::Router;

use service::schema::asset::AssetSchema;

#[derive(Clone)]
pub struct AssetApi {
    pub blockchain: Blockchain,
}

/// Shortcut to get data on wallets.
impl AssetApi {
    fn get_owner_for_asset(&self, asset_id: &str) -> Option<PublicKey> {
        let mut view = self.blockchain.fork();
        let mut schema = AssetSchema { view: &mut view };
        schema.creator(asset_id.to_string())
    }
}

impl Api for AssetApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let get_owner_for_asset_id = move |req: &mut Request| ->IronResult<Response> {
            let path = req.url.path();
            let asset_id = path.last().unwrap();
            if let Some(owner) = self_.get_owner_for_asset(*asset_id) {
                let res= self_.ok_response(&serde_json::to_value(owner).unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res = self_.not_found_response(&serde_json::to_value("Asset not found").unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };

        router.get("/asset/:asset_id", get_owner_for_asset_id, "get_owner_for_asset_id");
    }
}
