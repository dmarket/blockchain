use currency::asset::{AssetBundle, AssetId};

encoding_struct! {
    struct TradeAsset {
        const SIZE = 32;

        field id: AssetId [0 => 16]
        field amount: u64 [16 => 24]
        field price:  u64 [24 => 32]
    }
}

impl TradeAsset {
    pub fn from_bundle(bundle: AssetBundle, price: u64) -> Self {
        TradeAsset::new(bundle.id(), bundle.amount(), price)
    }

    pub fn total_price(&self) -> u64 {
        self.amount() as u64 * self.price()
    }
}
