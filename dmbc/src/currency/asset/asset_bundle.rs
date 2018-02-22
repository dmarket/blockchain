use exonum::crypto::PublicKey;

use currency::asset::AssetId;
use currency::asset::TradeAsset;

encoding_struct! {
    struct AssetBundle {
        const SIZE = 24;

        field id:     AssetId [0  => 16]
        field amount: u64     [16 => 24]
    }
}

impl AssetBundle {
    pub fn from_data_and_amount(data: &str, amount: u64, pub_key: &PublicKey) -> AssetBundle {
        let id = AssetId::from_data(data, pub_key);
        AssetBundle::new(id, amount)
    }
}

impl From<TradeAsset> for AssetBundle {
    fn from(ta: TradeAsset) -> Self {
        AssetBundle::new(ta.id(), ta.amount())
    }
}

