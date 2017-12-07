use exonum::crypto::PublicKey;
use service::assetid::AssetID;

pub const ASSET_HASH_ID_MAX_LENGTH: usize = 10 * 1024; // 10 KBytes

encoding_struct! {
    struct MetaAsset {
        const SIZE = 12;

        field meta_data: &str   [0 => 8]
        field amount: u32       [8 => 12]
    }
}

impl MetaAsset {
    pub fn count(assets: &[MetaAsset]) -> u64 {
        assets.iter().fold(
            0,
            |acc, asset| acc + asset.amount() as u64,
        )
    }

    pub fn is_valid(&self) -> bool {
        self.meta_data().len() <= ASSET_HASH_ID_MAX_LENGTH
    }
}

encoding_struct! {
    struct AssetInfo {
        const SIZE = 36;

        field creator: &PublicKey [0  => 32]
        field amount:  u32        [32 => 36]
    }
}

encoding_struct! {
    struct Asset {
        const SIZE = 20;

        field hash_id: AssetID [0 =>  16]
        field amount:  u32 [16 => 20]
    }
}

impl Asset {
    pub fn is_eq(&self, other: &Asset) -> bool {
        self.hash_id() == other.hash_id()
    }

    pub fn is_available_to_transfer(&self, other: &Asset) -> bool {
        self.amount() >= other.amount()
    }

    pub fn count(assets: &[Asset]) -> u64 {
        assets.iter().fold(
            0,
            |acc, asset| acc + asset.amount() as u64,
        )
    }
}