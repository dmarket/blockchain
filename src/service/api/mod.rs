pub mod transaction;
pub mod asset;
pub mod wallet;
pub mod hash;

extern crate params;

use exonum::blockchain::Blockchain;
use exonum::node::ApiSender;
use exonum::api::Api;
use router::Router;
use iron::prelude::*;
use std;

use self::transaction::TransactionApi;
use self::asset::AssetApi;
use self::wallet::WalletApi;
use self::hash::HashApi;
use self::params::{Params, FromValue};

const PARAMETER_OFFSET_KEY: &str = "offset";
const PARAMETER_LIMIT_KEY: &str = "limit";

#[derive(Clone)]
pub struct ServiceApi {
    pub channel: ApiSender,
    pub blockchain: Blockchain,
}

impl ServiceApi {

    /// returns a slice `&[T]` if `request` has pagination parameters.
    /// `offset` and `limit`, otherwise returns existing slice
    ///
    /// # URL request
    ///
    /// ` https://blockchain.com/api/services/cryptocurrency/v1/wallets?offset=4&limit=10 `
    pub fn apply_pagination<'a, T>(req: &mut Request, elements: &'a [T]) -> &'a [T] {
        let total_count = elements.len();
        // read url parameters
        let parameters = req.get_ref::<Params>().unwrap();
        let offset_parameter = parameters.get(PARAMETER_OFFSET_KEY);
        let limit_parameter = parameters.get(PARAMETER_LIMIT_KEY);

        // pagination parameters `offset` and `limit` should be considered together
        if offset_parameter.is_some() && limit_parameter.is_some() {
            let offset = FromValue::from_value(offset_parameter.unwrap()).unwrap_or(0);
            let limit = FromValue::from_value(limit_parameter.unwrap()).unwrap_or(total_count);

            // validate parameters for pagination 
            let from = std::cmp::min(offset, total_count);
            let to = std::cmp::min(from + limit, total_count);
            return &elements[from..to]
        }

        elements
    }
}

impl Api for ServiceApi {
    fn wire(&self, router: &mut Router) {
        let self_ = self.clone();
        let api = TransactionApi {
            channel: self_.channel,
            blockchain: self_.blockchain,
        };
        api.wire(router);

        let api = AssetApi { blockchain: self.clone().blockchain };
        api.wire(router);

        let api = WalletApi { blockchain: self.clone().blockchain };
        api.wire(router);

        let api = HashApi {};
        api.wire(router);
    }
}
