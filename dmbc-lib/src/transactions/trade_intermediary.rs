use exonum::crypto::{PublicKey, Signature};

use assets::TradeAsset;
use transactions::components::Intermediary;
use rudmbc::SERVICE_ID;

/// Transaction ID.
pub const TRADE_INTERMEDIARY_ID: u16 = 502;

encoding_struct! {
    struct TradeOfferIntermediary {
        intermediary: Intermediary,
        buyer:        &PublicKey,
        seller:       &PublicKey,
        assets:       Vec<TradeAsset>,

        fee_strategy: u8,
    }
}

message! {
    /// `trade_intermediary` transaction.
    struct TradeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = TRADE_INTERMEDIARY_ID;

        offer:                  TradeOfferIntermediary,
        seed:                   u64,
        seller_signature:       &Signature,
        intermediary_signature: &Signature,
        data_info:              &str,
    }
}