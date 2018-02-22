use exonum::crypto::PublicKey;

use currency::asset::{AssetId, Fees, AssetInfo, AssetBundle};

pub const ASSET_DATA_MAX_LENGTH: usize = 10 * 1024;

encoding_struct! {
    struct MetaAsset {
        const SIZE = 52;

        field receiver:  &PublicKey [0  => 32]
        field data:      &str       [32 => 40]
        field amount:    u64        [40 => 44]
        field fees:      Fees       [44 => 52]
    }
}

impl MetaAsset {
    pub fn verify(&self) -> bool {
        let trade_ok    = self.fees().trade().ratio() != 0;
        let exchange_ok = self.fees().exchange().ratio() != 0;
        let transfer_ok = self.fees().transfer().ratio() != 0;
        let data_ok     = self.data().len() <= ASSET_DATA_MAX_LENGTH;

        trade_ok && exchange_ok && transfer_ok && data_ok
    }
    
    pub fn to_info(&self, creator: &PublicKey) -> AssetInfo {
        AssetInfo::new(creator, self.amount(), self.fees())
    }

    pub fn to_bundle(&self, id: AssetId) -> AssetBundle {
        AssetBundle::new(id, self.amount())
    }
}

