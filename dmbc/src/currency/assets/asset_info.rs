use exonum::crypto::{Hash, PublicKey};

use currency::assets::Fees;
use currency::error::Error;

encoding_struct! {
    /// Information about an asset in the network.
    struct AssetInfo {
        creator: &PublicKey,
        origin:  &Hash,
        amount:  u64,
        fees:    Fees,
    }
}

impl AssetInfo {
    /// Merge two `AssetInfo`s.
    ///
    /// # Errors
    /// Returns an `InvalidAssetInfo` error if the structs either have
    /// different creators or fee information.
    pub fn merge(self, other: AssetInfo) -> Result<Self, Error> {
        let fees = self.fees();
        let creator = self.creator();
        let origin = self.origin();

        if fees != other.fees() || creator != other.creator() {
            return Err(Error::InvalidAssetInfo);
        }

        Ok(AssetInfo::new(
            creator,
            origin,
            self.amount() + other.amount(),
            fees,
        ))
    }

    /// Decreases amount of assets
    ///
    /// # Errors
    /// Returns an `InsufficientAssets` error if requested amount is bigger
    /// than `AssetInfo` contains.
    pub fn decrease(self, amount: u64) -> Result<Self, Error> {
        if self.amount() < amount {
            return Err(Error::InsufficientAssets);
        }

        Ok(AssetInfo::new(
            self.creator(),
            self.origin(),
            self.amount() - amount,
            self.fees(),
        ))
    }
}
