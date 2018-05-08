use exonum::crypto::{PublicKey, Signature};

use assets::TradeAsset;
use service::SERVICE_ID;

/// Transaction ID.
pub const TRADE_ID: u16 = 501;

encoding_struct! {
    struct TradeOffer {
        buyer: &PublicKey,
        seller: &PublicKey,
        assets: Vec<TradeAsset>,

        fee_strategy: u8,
    }
}

message! {
    /// `trade` transaction.
    struct Trade {
        const TYPE = SERVICE_ID;
        const ID = TRADE_ID;

        offer:              TradeOffer,
        seed:               u64,
        seller_signature:   &Signature,
    }
}