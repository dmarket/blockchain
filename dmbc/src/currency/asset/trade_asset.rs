use currency::asset::AssetId;

encoding_struct! {
    struct TradeAsset {
        const SIZE = 28;

        field id: AssetId [0 => 16]
        field amount: u64 [16 => 20]
        field price: u64  [20 => 28]
    }
}

impl TradeAsset {
    pub fn total_price(&self) -> u64 {
        self.amount() as u64 * self.price()
    }
}
