use exonum::crypto::PublicKey;

use currency::assets::Fees;
use currency::error::Error;

encoding_struct! {
    struct AssetInfo {
        const SIZE = 48;

        field creator: &PublicKey [0  => 32]
        field amount:  u64        [32 => 40]
        field fees:    Fees       [40 => 48]
    }
}

impl AssetInfo {
    pub fn merge(self, other: AssetInfo) -> Result<Self, Error> {
        let fees = self.fees();
        let creator = self.creator();

        if  fees != other.fees()
        ||  creator != other.creator()
        {
            return Err(Error::InvalidAssetInfo);
        }

        Ok(AssetInfo::new(creator, self.amount() + other.amount(), fees))
    }
}
