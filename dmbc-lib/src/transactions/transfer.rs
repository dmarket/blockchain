use exonum::crypto::PublicKey;

use assets::AssetBundle;
use service::SERVICE_ID;

/// Transaction ID.
pub const TRANSFER_ID: u16 = 200;

message! {
    /// `transfer` transaction.
    struct Transfer {
        const TYPE = SERVICE_ID;
        const ID = TRANSFER_ID;

        from:      &PublicKey,
        to:        &PublicKey,
        amount:    u64,
        assets:    Vec<AssetBundle>,
        seed:      u64,
        data_info: &str,
    }
}