encoding_struct! {
    /// Fee data for specific kind of operations.
    struct Fee {
        const SIZE = 16;

        field tax:   u64  [0 => 8]
        field ratio: u64  [8 => 16]
    }
}

encoding_struct! {
    /// Third party fee data, part of `AssetInfo`.
    struct Fees {
        const SIZE = 24;

        field trade:    Fee [ 0 => 8]
        field exchange: Fee [ 8 => 16]
        field transfer: Fee [16 => 24]
    }
}

impl Fee {
    /// Calculate fee value for specific price.
    pub fn for_price(&self, price: u64) -> u64 {
        let price_ratio = if self.ratio() > 0 {
            price / self.ratio()
        } else {
            0
        };
        self.tax() + price_ratio
    }
}
