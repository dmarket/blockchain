extern crate serde_json;

use currency::api::ServiceApi;
use currency::api::error::ApiError;
use currency::assets::{AssetId};
use currency::offers;
use currency::offers::OpenOffers;


use exonum::api::Api;
use exonum::blockchain::Blockchain;
use hyper::header::ContentType;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use iron::status;
use prometheus::IntCounter;
use router::Router;
use std::collections::HashMap;

const MAX_BLOCKS_PER_REQUEST: u64 = 1000;

#[derive(Clone)]
pub struct OfferApi {
    pub blockchain: Blockchain,
}
pub type OpenOffersResult = Result<Option<OpenOffers>, ApiError>;

pub type OpenOffersResponse = Result<OpenOffersInfo, ApiError>;
#[derive(Clone, Serialize, Deserialize)]
pub struct OpenOffersInfo {
    pub total: u64,
    pub count: u64,
    pub offers_info: HashMap<AssetId, OpenOfferInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenOfferInfo {
    pub bids: u64,
    pub asks: u64,
}

impl OpenOfferInfo {
    pub fn from(open_offers: &OpenOffers) -> Self {
        OpenOfferInfo {
            bids: open_offers.bids().into_iter().map(|bid| bid.offers().len() as u64).sum(),
            asks: open_offers.asks().into_iter().map(|ask| ask.offers().len() as u64).sum(),
        }
    }
}

impl OfferApi {
    fn pagination_offers(
        &self,
        offset: u64,
        limit: u64,
    ) -> (HashMap<AssetId, OpenOfferInfo>, u64, u64) {
        let view = &mut self.blockchain.fork();
        let idx = offers::Schema(view).index();
        let mut total: u64 = 0;
        let mut count: u64 = 0;
        let mut result: HashMap<AssetId, OpenOfferInfo> = HashMap::new();
        for v in idx.iter() {
            if total < offset || total >= offset + limit {
                total += 1;
                continue;
            }
            let wi = OpenOfferInfo::from(&v.1);
            result.insert(v.0, wi);
            count += 1;
            total += 1;
        }

        (result, total, count)
    }

}

lazy_static! {
    static ref LIST_REQUESTS: IntCounter = register_int_counter!(
        "dmbc_offers_api_list_requests_total",
        "OpenOffer list requests."
    ).unwrap();
    static ref LIST_RESPONSES: IntCounter = register_int_counter!(
        "dmbc_offers_api_list_responses_total",
        "OpenOffer list responses."
    ).unwrap();
    static ref INFO_REQUESTS: IntCounter = register_int_counter!(
        "dmbc_offers_api_info_requests_total",
        "OpenOffer info requests."
    ).unwrap();
    static ref INFO_RESPONSES: IntCounter = register_int_counter!(
        "dmbc_offers_api_info_responses_total",
        "OpenOffer info responses."
    ).unwrap();
}

impl Api for OfferApi {
    fn wire(&self, router: &mut Router) {
        // Gets status of the wallet corresponding to the public key.
        let _self = self.clone();
        let offers_info = move |req: &mut Request| -> IronResult<Response> {
            LIST_REQUESTS.inc();

            let (offset, limit) = ServiceApi::pagination_params(req);
            let (offers_info, total, count) = _self.pagination_offers(offset, limit);

            let result: OpenOffersResponse = Ok(OpenOffersInfo {
                total,
                count,
                offers_info,
            });

            let mut res =
                Response::with((status::Ok, serde_json::to_string_pretty(&result).unwrap()));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            LIST_RESPONSES.inc();

            Ok(res)
        };

        let _self = self.clone();
        let bids_asks = move |req: &mut Request| -> IronResult<Response> {
            INFO_REQUESTS.inc();

            let params = req.extensions.get::<Router>().unwrap();

            let result: OpenOffersResult = match params.find("asset_id") {
                Some(asset_id_str) =>
                    match AssetId::from_hex(asset_id_str) {
                        Ok(asset_id) => {
                            let view = &mut _self.blockchain.fork();
                            Ok(Some(offers::Schema(view).fetch(&asset_id)))
                        },
                        Err(_) => Err(ApiError::AssetIdInvalid)
                    },
                None => Err(ApiError::IncorrectRequest),
            };

            let status_code = match result {
                Ok(_) => status::Ok,
                Err(e) => e.to_status(),
            };
            let body = serde_json::to_string_pretty(&result).unwrap();
            let mut res = Response::with((status_code, body));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            INFO_RESPONSES.inc();
            Ok(res)

        };

        router.get("/v1/offers/", offers_info, "open_offers");
        router.get("/v1/offers/:asset_id", bids_asks, "bids_asks");
    }
}
