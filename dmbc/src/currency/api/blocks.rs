extern crate serde_json;

use exonum::api::Api;
use exonum::blockchain::{Blockchain, Block};
use exonum::explorer::{BlockchainExplorer, BlockInfo};
use currency::api::error::ApiError;
use exonum::helpers::Height;
use hyper::header::ContentType;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use iron::status;
use prometheus::IntCounter;
use router::Router;
use currency::api::params::{Params, Value};

const MAX_BLOCKS_PER_REQUEST: u64 = 1000;

#[derive(Clone)]
pub struct BlocksApi {
    pub blockchain: Blockchain,
}

pub type BlocksResponse = Result<Vec<Block>, ApiError>;

impl BlocksApi {

    fn get_blocks(
        &self,
        count: u64,
        from: Option<u64>,
        skip_empty_blocks: bool,
    ) -> Result<Vec<Block>, ApiError> {
        if count > MAX_BLOCKS_PER_REQUEST {
            return Err(ApiError::IncorrectRequest);
        }
        let explorer = BlockchainExplorer::new(&self.blockchain);
        Ok(explorer.blocks_range(count, from, skip_empty_blocks))
    }

    fn get_block(&self, height: Height) -> Result<Option<BlockInfo>, ApiError> {
        let explorer = BlockchainExplorer::new(&self.blockchain);
        match explorer.block_info(height) {
            Some(b) => Ok(Some(b)),
            None => Err(ApiError::BlockNotFound)
        }
    }

}

lazy_static! {
    static ref LIST_REQUESTS: IntCounter = register_int_counter!(
        "dmbc_blocks_api_list_requests_total",
        "Block list requests."
    ).unwrap();
    static ref LIST_RESPONSES: IntCounter = register_int_counter!(
        "dmbc_blocks_api_list_responses_total",
        "Block list responses."
    ).unwrap();
    static ref INFO_REQUESTS: IntCounter = register_int_counter!(
        "dmbc_blocks_api_info_requests_total",
        "Block info requests."
    ).unwrap();
    static ref INFO_RESPONSES: IntCounter = register_int_counter!(
        "dmbc_blocks_api_info_responses_total",
        "Block info responses."
    ).unwrap();
}

impl Api for BlocksApi {
    fn wire(&self, router: &mut Router) {
        // Gets status of the wallet corresponding to the public key.
        let _self = self.clone();
        let blocks = move |req: &mut Request| -> IronResult<Response> {
            LIST_REQUESTS.inc();

            let map = req.get_ref::<Params>().unwrap();
            let mut err:Option<ApiError> = None;
            let count: u64 = match map.find(&["count"]) {
                Some(&Value::String(ref count_str)) => {
                    count_str
                        .parse()
                        .unwrap_or_else(|_| { err = Some(ApiError::IncorrectRequest); 0 })
                }
                _ => MAX_BLOCKS_PER_REQUEST
            };
            let latest: Option<u64> = match map.find(&["latest"]) {
                Some(&Value::String(ref from_str)) => {
                    from_str
                        .parse()
                        .map(|f|{ Some(f) })
                        .unwrap_or_else(|_| { err = Some(ApiError::IncorrectRequest); None })
                }
                _ => None
            };
            let skip_empty_blocks: bool = match map.find(&["skip_empty_blocks"]) {
                Some(&Value::String(ref skip_str)) => {
                    skip_str
                        .parse()
                        .unwrap_or_else(|_| { err = Some(ApiError::IncorrectRequest); false })
                }
                _ => false,
            };

            let result = match err {
                None => _self.get_blocks(count, latest, skip_empty_blocks),
                Some(e) => Err(e)
            };
            let status_code = result.clone()
                .map(|_| { status::Ok })
                .unwrap_or(status::BadRequest);

            let mut res =
                Response::with((status_code, serde_json::to_string_pretty(&result).unwrap()));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            LIST_RESPONSES.inc();
            Ok(res)
        };

        let _self = self.clone();
        let block = move |req: &mut Request| -> IronResult<Response> {
            INFO_REQUESTS.inc();

            let params = req.extensions.get::<Router>().unwrap();

            let result:Result<Option<BlockInfo>, ApiError> = match params.find("height") {
                Some(height_str) => {
                    height_str.parse()
                        .map_err(|_|ApiError::IncorrectRequest)
                        .and_then(|h| _self.get_block(Height(h)))
                }
                None => Err(ApiError::IncorrectRequest)
            };

            let status_code = match result {
                Ok(_) => status::Ok,
                Err(e) => e.to_status(),
            };
            let body = serde_json::to_string_pretty(&result).unwrap();
            let mut res =
                Response::with((status_code, body));
            res.headers.set(ContentType::json());
            res.headers.set(AccessControlAllowOrigin::Any);

            INFO_RESPONSES.inc();
            Ok(res)
        };

        router.get("/v1/blocks", blocks, "blocks_info");
        router.get("/v1/blocks/:height", block, "height");
    }
}
