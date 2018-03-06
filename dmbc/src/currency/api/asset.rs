extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use exonum::api::Api;
use exonum::blockchain::Blockchain;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use router::Router;

use currency::assets;
use currency::assets::{AssetId, AssetInfo};

#[derive(Clone)]
pub struct AssetApi {
    pub blockchain: Blockchain,
}

/// Shortcut to get data on wallets.
impl AssetApi {
    fn get_owner_for_asset(&self, asset_id: &AssetId) -> Option<AssetInfo> {
        let view = self.blockchain.fork();
        assets::Schema(view).fetch(asset_id)
    }
}

impl Api for AssetApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let get_owner_for_asset_id = move |req: &mut Request| -> IronResult<Response> {
            let path = req.url.path();
            let asset_id_str = path.last().unwrap();
            let asset_id = AssetId::from_hex(&asset_id_str);
            if asset_id.is_err() {
                let res =
                    self_.not_found_response(&serde_json::to_value("Invalid Asset ID").unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                return Ok(res);
            }
            if let Some(owner) = self_.get_owner_for_asset(&asset_id.unwrap()) {
                let res = self_.ok_response(&serde_json::to_value(owner).unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            } else {
                let res =
                    self_.not_found_response(&serde_json::to_value("Asset not found").unwrap());
                let mut res = res.unwrap();
                res.headers.set(AccessControlAllowOrigin::Any);
                Ok(res)
            }
        };

        router.get(
            "/asset/:asset_id",
            get_owner_for_asset_id,
            "get_owner_for_asset_id",
        );
    }
}
