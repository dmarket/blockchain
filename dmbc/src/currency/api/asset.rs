extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use exonum::api::Api;
use exonum::blockchain::Blockchain;
use hyper::header::ContentType;
use iron::headers::AccessControlAllowOrigin;
use iron::status;
use iron::prelude::*;
use router::Router;
use prometheus::Counter;
use lazy_static;

use currency::api::error::ApiError;
use currency::assets;
use currency::assets::{AssetId, AssetInfo};

#[derive(Clone)]
pub struct AssetApi {
    pub blockchain: Blockchain,
}

/// Shortcut to get data on wallets.
impl AssetApi {
    fn get_asset_info(&self, asset_id: &AssetId) -> Option<AssetInfo> {
        let view = self.blockchain.fork();
        assets::Schema(view).fetch(asset_id)
    }
}

pub type AssetResponse = Result<Option<AssetInfo>, ApiError>;

lazy_static! {
    static ref INFO_REQUESTS: Counter = register_counter!("dmbc_asset_api_info_requests_total", "AssetInfo requests.").unwrap();
    static ref INFO_RESPONSES: Counter = register_counter!("dmbc_asset_api_info_responses_total", "AssetInfo response.").unwrap();
}

impl Api for AssetApi {
    fn wire(&self, router: &mut Router) {
        lazy_static::initialize(&INFO_REQUESTS);
        lazy_static::initialize(&INFO_RESPONSES);

        let self_ = self.clone();
        let get_owner_for_asset_id = move |req: &mut Request| -> IronResult<Response> {
            INFO_REQUESTS.inc();

            let path = req.url.path();
            let asset_id_str = path.last().unwrap();
            let a: AssetResponse = AssetId::from_hex(&asset_id_str)
                .map_err(|_| ApiError::AssetIdHashInvalid)
//                .and_then(|asset_id| self_.get_asset_info(&asset_id).ok_or(ApiError::AssetIdNotFound));
                .map(|asset_id| self_.get_asset_info(&asset_id));

            let mut res = Response::with((
                a.clone().err().map(|e| e.to_status()).unwrap_or(status::Ok),
                serde_json::to_string_pretty(&a).unwrap(),
            ));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            INFO_RESPONSES.inc();

            Ok(res)
        };

        router.get(
            "/v1/assets/:asset_id",
            get_owner_for_asset_id,
            "get_owner_for_asset_id",
        );
    }
}
