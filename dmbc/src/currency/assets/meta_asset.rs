use exonum::crypto::PublicKey;

use currency::assets::{AssetBundle, AssetId, AssetInfo, Fees};

pub const ASSET_DATA_MAX_LENGTH: usize = 10 * 1024;

encoding_struct! {
    /// Info for asset to be committed into the network in `add_assets` transaction.
    struct MetaAsset {
        const SIZE = 56;

        field receiver:  &PublicKey [0  => 32]
        field data:      &str       [32 => 40]
        field amount:    u64        [40 => 48]
        field fees:      Fees       [48 => 56]
    }
}

impl MetaAsset {
    /// Verify valididty of the committed assets.
    pub fn verify(&self) -> bool {
        self.data().len() <= ASSET_DATA_MAX_LENGTH
    }

    /// Create an `AssetInfo` from this `MetaAsset`.
    pub fn to_info(&self, creator: &PublicKey) -> AssetInfo {
        AssetInfo::new(creator, self.amount(), self.fees())
    }

    /// Create an `AssetBundle` from this `MetaAsset`.
    pub fn to_bundle(&self, id: AssetId) -> AssetBundle {
        AssetBundle::new(id, self.amount())
    }
}
