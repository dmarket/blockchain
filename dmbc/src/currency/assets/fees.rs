encoding_struct! {
    /// Fee data for specific kind of operations.
    struct Fee {
        tax:   u64,
        ratio: u64,
    }
}

encoding_struct! {
    /// Third party fee data, part of `AssetInfo`.
    struct Fees {
        trade:    Fee,
        exchange: Fee,
        transfer: Fee,
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
