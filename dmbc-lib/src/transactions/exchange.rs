use exonum::crypto::{PublicKey, Signature};

use assets::AssetBundle;
use rudmbc::SERVICE_ID;

/// Transaction ID.
pub const EXCHANGE_ID: u16 = 601;

encoding_struct! {
    struct ExchangeOffer {
        sender:           &PublicKey,
        sender_assets:    Vec<AssetBundle>,
        sender_value:     u64,

        recipient:        &PublicKey,
        recipient_assets: Vec<AssetBundle>,

        fee_strategy:     u8,
    }
}

message! {
    /// `exchange` transaction.
    struct Exchange {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_ID;

        offer:             ExchangeOffer,
        seed:              u64,
        sender_signature:  &Signature,
        data_info:         &str,
    }
}