use exonum::crypto::PublicKey;

use currency::assets::AssetId;
use currency::assets::TradeAsset;

encoding_struct! {
    /// A bundle of assets with the same id.
    struct AssetBundle {
        const SIZE = 24;

        field id:     AssetId [0  => 16]
        field amount: u64     [16 => 24]
    }
}

impl AssetBundle {
    /// Create new `AssetBundle` from data string and public key.
    pub fn from_data(data: &str, amount: u64, pub_key: &PublicKey) -> AssetBundle {
        let id = AssetId::from_data(data, pub_key);
        AssetBundle::new(id, amount)
    }
}

impl From<TradeAsset> for AssetBundle {
    fn from(ta: TradeAsset) -> Self {
        AssetBundle::new(ta.id(), ta.amount())
    }
}
