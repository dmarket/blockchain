use exonum::crypto::PublicKey;

use capi::SERVICE_ID;
use assets::AssetBundle;

/// Transaction ID.
pub const DELETE_ASSETS_ID: u16 = 400;

message! {
    /// `delete_assets` transaction.
    struct DeleteAssets {
        const TYPE = SERVICE_ID;
        const ID = DELETE_ASSETS_ID;

        pub_key:     &PublicKey,
        assets:      Vec<AssetBundle>,
        seed:        u64,
    }
}