extern crate bodyparser;
extern crate exonum;
extern crate iron;
extern crate router;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use exonum::api::Api;
use exonum::blockchain::Blockchain;
use exonum::crypto::PublicKey;
// use exonum::encoding::serialize::FromHex;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use iron::status;
use router::Router;
use hyper::header::ContentType;

use currency::api::error::ApiError;
use currency::error::Error;
use currency::transactions::components::FeesCalculator;
use currency::transactions::{AddAssets, DeleteAssets, Exchange,
                             ExchangeIntermediary, Mine, Trade,
                             TradeIntermediary, Transfer};

#[derive(Clone)]
pub struct FeesApi {
    pub blockchain: Blockchain,
}

#[serde(untagged)]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum FeesRequest {
    Transfer(Transfer),
    AddAssets(AddAssets),
    DeleteAssets(DeleteAssets),
    Trade(Trade),
    TradeIntermediary(TradeIntermediary),
    Exchange(Exchange),
    // ExchangeIntermediary(ExchangeIntermediary),
    // Mine(Mine),
}

impl Into<Box<FeesCalculator>> for FeesRequest {
    fn into(self) -> Box<FeesCalculator> {
        match self {
            FeesRequest::Transfer(trans) => Box::new(trans),
            FeesRequest::AddAssets(trans) => Box::new(trans),
            FeesRequest::DeleteAssets(trans) => Box::new(trans),
            FeesRequest::Trade(trans) => Box::new(trans),
            FeesRequest::TradeIntermediary(trans) => Box::new(trans),
            FeesRequest::Exchange(trans) => Box::new(trans),
            // FeesRequest::ExchangeIntermediary(trans) => Box::new(trans),
            // FeesRequest::Mine(trans) => Box::new(trans),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FeesResponseBody {
    pub fees: HashMap<PublicKey, u64>,
}

pub type FeesResponse = Result<Result<FeesResponseBody, Error>, ApiError>;

impl Api for FeesApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let fees = move |req: &mut Request| -> IronResult<Response> {
            let result: FeesResponse = match req.get::<bodyparser::Struct<FeesRequest>>() {
                Ok(Some(request)) => {
                    let calculator: Box<FeesCalculator> = request.into();
                    let view = &mut self_.blockchain.fork();
                    match calculator.get_fees(view) {
                        Ok(fees) => Ok(Ok(FeesResponseBody{ fees })),
                        Err(e) => Ok(Err(e)),
                    }
                },
                Ok(None) => Err(ApiError::EmptyRequestBody),
                Err(_) => Err(ApiError::IncorrectRequest),
            };

            let status_code =
                result.clone()
                    .ok()
                    .map(|r|
                        r.err().map(|_| status::BadRequest).unwrap_or(status::Created))
                    .unwrap_or(status::BadRequest);

            let mut res = Response::with((
                status_code,
                serde_json::to_string_pretty(&result).unwrap(),
            ));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            Ok(res)
        };

        router.post("/v1/fees/transactions", fees, "transaction_fee");
    }
}