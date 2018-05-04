use decimal;

encoding_struct! {
    /// Fee data for specific kind of operations.
    struct Fee {
        fixed:      u64,
        fraction: decimal::UFract64,
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
        self.fixed() + self.fraction() * price
    }
}
