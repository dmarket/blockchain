extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use exonum::api::{Api, ApiError as ExonumApiError};
use exonum::blockchain::Blockchain;
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;
use hyper::header::ContentType;
use iron::headers::AccessControlAllowOrigin;
use iron::status;
use iron::prelude::*;
use router::Router;

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

pub type AssetResponse = Result<AssetInfo, ApiError>;

impl Api for AssetApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let get_owner_for_asset_id = move |req: &mut Request| -> IronResult<Response> {
            let path = req.url.path();
            let asset_id_str = path.last().unwrap();
            let a: AssetResponse = AssetId::from_hex(&asset_id_str)
                .map_err(|_| ApiError::AssetIdHashInvalid)
                .and_then(|asset_id| self_.get_asset_info(&asset_id).ok_or(ApiError::AssetIdNotFound));

            let mut res = Response::with((
                a.clone().err().map(|e| e.to_status()).unwrap_or(status::Ok),
                serde_json::to_string_pretty(&a).unwrap(),
            ));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            Ok(res)
        };

        let self_ = self.clone();
        let get_asset_id = move |req: &mut Request| -> IronResult<Response> {
            let public_key = {
                let wallet_key = req.extensions
                    .get::<Router>()
                    .unwrap()
                    .find("pub_key")
                    .unwrap();
                PublicKey::from_hex(wallet_key).map_err(ExonumApiError::FromHex)?
            };
            let meta_data = {
                req.extensions
                    .get::<Router>()
                    .unwrap()
                    .find("meta_data")
                    .unwrap()
            };
            let id = AssetId::from_data(meta_data, &public_key);

            let response_body = json!({
                "id": id.to_string(),
            });

            let res = self_.ok_response(&serde_json::to_value(response_body).unwrap());
            let mut res = res.unwrap();
            res.headers.set(AccessControlAllowOrigin::Any);
            Ok(res)
        };

        router.get(
            "/v1/assets/:asset_id",
            get_owner_for_asset_id,
            "get_owner_for_asset_id",
        );

        router.get(
            "/v1/assets/:pub_key/:meta_data",
            get_asset_id,
            "asset_id",
        );
    }
}
