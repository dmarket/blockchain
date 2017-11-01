pub mod transaction;
pub mod asset;
pub mod wallet;
pub mod hash;

use exonum::blockchain::Blockchain;
use exonum::node::{ApiSender, NodeChannel};
use exonum::api::Api;
use router::Router;

use self::transaction::TransactionApi;
use self::asset::AssetApi;
use self::wallet::WalletApi;
use self::hash::HashApi;

#[derive(Clone)]
pub struct ServiceApi {
    pub channel: ApiSender<NodeChannel>,
    pub blockchain: Blockchain,
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
