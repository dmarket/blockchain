extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

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

#[derive(Clone, Serialize, Deserialize, Debug)]
struct AssetIdRequest {
    pub assets: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct AssetIdResponse {
    pub assets: HashMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct AssetIdBatchRequest {
    pub assets: HashMap<String, Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct AssetIdBatchResponse {
    pub assets: HashMap<String, HashMap<String, String>>,
}

pub type AssetResponse = Result<Option<AssetInfo>, ApiError>;

impl Api for AssetApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let get_owner_for_asset_id = move |req: &mut Request| -> IronResult<Response> {
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

        let self_ = self.clone();
        let get_assets_ids = move |req: &mut Request| -> IronResult<Response> {
            let public_key = {
                let wallet_key = req.extensions
                    .get::<Router>()
                    .unwrap()
                    .find("pub_key")
                    .unwrap();
                PublicKey::from_hex(wallet_key).map_err(ExonumApiError::FromHex)?
            };
            match req.get::<bodyparser::Struct<AssetIdRequest>>() {
                Ok(Some(request)) => {
                    let mut assets = HashMap::<String, String>::new();
                    for asset in request.assets {
                        let id = AssetId::from_data(&asset, &public_key);
                        assets.insert(asset, id.to_string());
                    }
                    let response_data = json!(AssetIdResponse { assets });
                    let ok_res = self_.ok_response(&response_data);
                    let mut res = ok_res.unwrap();
                    res.headers.set(AccessControlAllowOrigin::Any);
                    Ok(res)
                },
                Ok(None) => Err(ExonumApiError::IncorrectRequest("Empty request body".into()))?,
                Err(e) => Err(ExonumApiError::IncorrectRequest(Box::new(e)))?,
            }
        };

        let self_ = self.clone();
        let get_asset_id_batch = move |req: &mut Request| -> IronResult<Response> {
            match req.get::<bodyparser::Struct<AssetIdBatchRequest>>() {
                Ok(Some(request)) => {
                    let mut assets_batch = HashMap::<String, HashMap<String, String>>::new();
                    for (key, assets_data) in request.assets {
                        let public_key = PublicKey::from_hex(key.clone()).map_err(ExonumApiError::FromHex)?;
                        let mut assets = HashMap::<String, String>::new();
                        for asset in assets_data {
                            let id = AssetId::from_data(&asset, &public_key);
                            assets.insert(asset, id.to_string());
                        }
                        assets_batch.insert(key, assets);
                    }

                    let response_data = json!(AssetIdBatchResponse { assets: assets_batch });
                    let ok_res = self_.ok_response(&response_data);
                    let mut res = ok_res.unwrap();
                    res.headers.set(AccessControlAllowOrigin::Any);
                    Ok(res)
                },
                Ok(None) => Err(ExonumApiError::IncorrectRequest("Empty request body".into()))?,
                Err(e) => Err(ExonumApiError::IncorrectRequest(Box::new(e)))?,
            }
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

        router.post(
            "/v1/assets/:pub_key",
            get_assets_ids,
            "assets_ids",
        );

        router.post(
            "/v1/assets/batch",
            get_asset_id_batch,
            "assets_ids_batch"
        );
    }
}
