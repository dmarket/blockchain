extern crate serde_json;

use currency::api::ServiceApi;
use currency::api::error::ApiError;
use currency::offers::HistoryOffers;
use currency::offers::history;

use exonum::api::Api;
use exonum::blockchain::Blockchain;
use exonum::crypto::Hash;
use exonum::encoding::serialize::FromHex;
use hyper::header::ContentType;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use iron::status;
use prometheus::IntCounter;
use router::Router;
use std::collections::HashMap;

const MAX_BLOCKS_PER_REQUEST: u64 = 1000;

#[derive(Clone)]
pub struct HistoryOffersApi {
    pub blockchain: Blockchain,
}
pub type HistoryOfferResult = Result<Option<HistoryOffers>, ApiError>;

pub type HistoryOffersResponse = Result<HistoryOffersInfo, ApiError>;
#[derive(Clone, Serialize, Deserialize)]
pub struct HistoryOffersInfo {
    pub total: u64,
    pub count: u64,
    pub offer_info: HashMap<Hash, HistoryOfferInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HistoryOfferInfo {
    pub tx_amount: u64,
}

impl HistoryOfferInfo {
    pub fn from(open_offers: &HistoryOffers) -> Self {
        HistoryOfferInfo {
            tx_amount: open_offers.history().len() as u64,
        }
    }
}

impl HistoryOffersApi {
    fn pagination_history_offers(
        &self,
        offset: u64,
        limit: u64,
    ) -> (HashMap<Hash, HistoryOfferInfo>, u64, u64) {
        let view = &mut self.blockchain.fork();
        let idx = history::Schema(view).index();
        let mut total: u64 = 0;
        let mut count: u64 = 0;
        let mut result = HashMap::new();
        for v in idx.iter() {
            if total < offset || total >= offset + limit {
                total += 1;
                continue;
            }
            let hoi = HistoryOfferInfo::from(&v.1);
            result.insert(v.0, hoi);
            count += 1;
            total += 1;
        }

        (result, total, count)
    }

}

lazy_static! {
    static ref LIST_REQUESTS: IntCounter = register_int_counter!(
        "dmbc_history_offers_api_list_requests_total",
        "OpenOffer list requests."
    ).unwrap();
    static ref LIST_RESPONSES: IntCounter = register_int_counter!(
        "dmbc_history_offers_api_list_responses_total",
        "OpenOffer list responses."
    ).unwrap();
    static ref INFO_REQUESTS: IntCounter = register_int_counter!(
        "dmbc_history_offers_api_info_requests_total",
        "OpenOffer info requests."
    ).unwrap();
    static ref INFO_RESPONSES: IntCounter = register_int_counter!(
        "dmbc_history_offers_api_info_responses_total",
        "OpenOffer info responses."
    ).unwrap();
}

impl Api for HistoryOffersApi {
    fn wire(&self, router: &mut Router) {
        // Gets status of the wallet corresponding to the public key.
        let _self = self.clone();
        let history_offers_info = move |req: &mut Request| -> IronResult<Response> {
            LIST_REQUESTS.inc();

            let (offset, limit) = ServiceApi::pagination_params(req);
            let (offer_info, total, count) = _self.pagination_history_offers(offset, limit);

            let result: HistoryOffersResponse = Ok(HistoryOffersInfo {
                total,
                count,
                offer_info,
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

            let result: HistoryOfferResult = match params.find("tx_hash") {
                Some(tx_hash_str) =>
                    match Hash::from_hex(tx_hash_str) {
                        Ok(tx_hash) => {
                            let view = &mut _self.blockchain.fork();
                            Ok(Some(history::Schema(view).fetch(&tx_hash)))
                        },
                        Err(_) => Err(ApiError::TransactionHashInvalid)
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

        router.get("/v1/history/offers", history_offers_info, "history_offers_info");
        router.get("/v1/history/offers/:tx_hash", bids_asks, "offer_history");
    }
}
