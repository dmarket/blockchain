use currency::assets::{AssetBundle, AssetId};

encoding_struct! {
    /// Asset representation to used in `trade` transactions.
    struct TradeAsset {
        const SIZE = 32;

        field id: AssetId [0 => 16]
        field amount: u64 [16 => 24]
        field price:  u64 [24 => 32]
    }
}

impl TradeAsset {
    /// Create a `TradeAsset` from a bundle and per item price.
    pub fn from_bundle(bundle: AssetBundle, price: u64) -> Self {
        TradeAsset::new(bundle.id(), bundle.amount(), price)
    }

    /// Get total value of the `TradeAsset`.
    pub fn total_price(&self) -> u64 {
        self.amount() * self.price()
    }
}
