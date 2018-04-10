use exonum::crypto::{PublicKey, Hash};

use currency::assets::Fees;
use currency::error::Error;

encoding_struct! {
    /// Information about an asset in the network.
    struct AssetInfo {
        const SIZE = 80;

        field creator: &PublicKey [0  => 32]
        field origin:  &Hash      [32 => 64]
        field amount:  u64        [64 => 72]
        field fees:    Fees       [72 => 80]
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
            self.fees()
        ))
    }
}
