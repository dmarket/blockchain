use exonum::crypto::PublicKey;

use assets::MetaAsset;
use service::SERVICE_ID;

pub const ADD_ASSETS_ID: u16 = 300;

message!{
    /// `add_assets` transaction.
    struct AddAssets {
        const TYPE = SERVICE_ID;
        const ID = ADD_ASSETS_ID;

        pub_key:     &PublicKey,
        meta_assets: Vec<MetaAsset>,
        seed:        u64,
    }
}