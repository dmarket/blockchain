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