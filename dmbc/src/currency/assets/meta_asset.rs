use exonum::crypto::{Hash, PublicKey};

use currency::assets::{AssetBundle, AssetId, AssetInfo, Fees};

pub const ASSET_DATA_MAX_LENGTH: usize = 10 * 1024;

encoding_struct! {
    /// Info for asset to be committed into the network in `add_assets` transaction.
    struct MetaAsset {
        receiver:  &PublicKey,
        data:      &str,
        amount:    u64,
        fees:      Fees,
    }
}

impl MetaAsset {
    /// Verify valididty of the committed assets.
    pub fn verify(&self) -> bool {
        self.data().len() <= ASSET_DATA_MAX_LENGTH
    }

    /// Create an `AssetInfo` from this `MetaAsset`.
    pub fn to_info(&self, creator: &PublicKey, origin: &Hash) -> AssetInfo {
        AssetInfo::new(creator, origin, self.amount(), self.fees(), self.data())
    }

    /// Create an `AssetBundle` from this `MetaAsset`.
    pub fn to_bundle(&self, id: AssetId) -> AssetBundle {
        AssetBundle::new(id, self.amount())
    }
}
