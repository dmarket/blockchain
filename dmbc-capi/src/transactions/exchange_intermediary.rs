use exonum::crypto::{PublicKey, Signature};

use transactions::components::Intermediary;
use assets::AssetBundle;
use capi::SERVICE_ID;

/// Transaction ID.
pub const EXCHANGE_INTERMEDIARY_ID: u16 = 602;

encoding_struct! {
    struct ExchangeOfferIntermediary {
        intermediary:     Intermediary,

        sender:           &PublicKey,
        sender_assets:    Vec<AssetBundle>,
        sender_value:     u64,

        recipient:        &PublicKey,
        recipient_assets: Vec<AssetBundle>,

        fee_strategy:     u8,
    }
}

message! {
    /// `exchange_intermediary` transaction.
    struct ExchangeIntermediary {
        const TYPE = SERVICE_ID;
        const ID = EXCHANGE_INTERMEDIARY_ID;

        offer:                  ExchangeOfferIntermediary,
        seed:                   u64,
        sender_signature:       &Signature,
        intermediary_signature: &Signature,
        data_info:              &str,
    }
}