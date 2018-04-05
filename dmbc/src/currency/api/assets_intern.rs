extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use exonum::api::Api;
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use iron::status;
use router::Router;
use hyper::header::ContentType;

use currency::api::error::ApiError;
use currency::assets::AssetId;

#[derive(Clone)]
pub struct AssetInternApi {
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssetIdRequest {
    pub assets: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AssetIdResponseBody {
    pub assets: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AssetIdBatchRequest {
    pub assets: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AssetIdBatchResponseBody {
    pub assets: HashMap<String, HashMap<String, String>>,
}

pub type AssetIdResponse = Result<AssetIdResponseBody, ApiError>;

pub type AssetIdBatchResponse = Result<AssetIdBatchResponseBody, ApiError>;

impl Api for AssetInternApi {
    fn wire(&self, router: &mut Router) {

        let get_asset_id = move |req: &mut Request| -> IronResult<Response> {
            let public_key_result = {
                let wallet_key = req.extensions
                    .get::<Router>()
                    .unwrap()
                    .find("pub_key")
                    .unwrap();
                PublicKey::from_hex(wallet_key)
            };
            let meta_data = {
                req.extensions
                    .get::<Router>()
                    .unwrap()
                    .find("meta_data")
                    .unwrap()
            };
            let result: AssetIdResponse = match public_key_result {
                Ok(public_key) => {
                    let id = AssetId::from_data(meta_data, &public_key);
                    let mut assets = HashMap::<String, String>::new();
                    assets.insert(meta_data.to_string(), id.to_string());
                    Ok(AssetIdResponseBody { assets })
                },
                Err(_) => Err(ApiError::WalletHexInvalid)
            };

            let mut res = Response::with((
                result.clone().err().map(|e| e.to_status()).unwrap_or(status::Ok),
                serde_json::to_string_pretty(&result).unwrap(),
            ));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            Ok(res)
        };

        let get_asset_ids_for_key = move |req: &mut Request| -> IronResult<Response> {
            let public_key_result = {
                let wallet_key = req.extensions
                    .get::<Router>()
                    .unwrap()
                    .find("pub_key")
                    .unwrap();
                PublicKey::from_hex(wallet_key)
            };
            let result: AssetIdResponse = match public_key_result {
                Ok(public_key) => {
                    match req.get::<bodyparser::Struct<AssetIdRequest>>() {
                        Ok(Some(request)) => {
                            let mut assets = HashMap::<String, String>::new();
                            for asset in request.assets {
                                let id = AssetId::from_data(&asset, &public_key);
                                assets.insert(asset, id.to_string());
                            }
                            Ok(AssetIdResponseBody { assets })
                        },
                        Ok(None) => Err(ApiError::EmptyRequestBody),
                        Err(_) => Err(ApiError::IncorrectRequest),
                    }
                },
                Err(_) => Err(ApiError::WalletHexInvalid)
            };

            let mut res = Response::with((
                result.clone().err().map(|e| e.to_status()).unwrap_or(status::Ok),
                serde_json::to_string_pretty(&result).unwrap(),
            ));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            Ok(res)
        };

        let get_asset_id_batch = move |req: &mut Request| -> IronResult<Response> {
            let result: AssetIdBatchResponse = match req.get::<bodyparser::Struct<AssetIdBatchRequest>>() {
                Ok(Some(request)) => {

                    fn create_batch(assets: &HashMap<String, Vec<String>>) -> Option<HashMap<String, HashMap<String, String>>>{
                        let mut assets_batch = HashMap::new();
                        for (key, assets_data) in assets {
                            let public_key = PublicKey::from_hex(key.clone());
                            if public_key.is_err() {
                                return None;
                            }

                            let mut assets = HashMap::<String, String>::new();
                            for asset in assets_data {
                                let id = AssetId::from_data(&asset, &public_key.unwrap());
                                assets.insert(asset.to_string(), id.to_string());
                            }
                            assets_batch.insert(key.to_string(), assets);
                        }
                        Some(assets_batch)
                    }

                    match create_batch(&request.assets) {
                        Some(assets) => Ok(AssetIdBatchResponseBody { assets }),
                        None => Err(ApiError::WalletHexInvalid)
                    }
                },
                Ok(None) => Err(ApiError::EmptyRequestBody),
                Err(_) => Err(ApiError::IncorrectRequest),
            };

            let mut res = Response::with((
                result.clone().err().map(|e| e.to_status()).unwrap_or(status::Ok),
                serde_json::to_string_pretty(&result).unwrap(),
            ));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            Ok(res)
        };

        router.get(
            "/v1/intern/assets/:pub_key/:meta_data",
            get_asset_id,
            "asset_id",
        );

        router.post(
            "/v1/intern/assets/:pub_key",
            get_asset_ids_for_key,
            "assets_ids_for_key",
        );

        router.post(
            "/v1/intern/assets",
            get_asset_id_batch,
            "assets_ids_batch"
        );
    }
}