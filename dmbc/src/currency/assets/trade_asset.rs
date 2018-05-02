use currency::assets::{AssetBundle, AssetId};

encoding_struct! {
    /// Asset representation to used in `trade` transactions.
    struct TradeAsset {
        id: AssetId,
        amount: u64,
        price:  u64,
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

    pub fn to_bundle(&self) -> AssetBundle {
        AssetBundle::new(self.id(), self.amount())
    }
}
