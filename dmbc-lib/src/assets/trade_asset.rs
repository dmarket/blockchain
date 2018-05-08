use assets::AssetId;

encoding_struct! {
    /// Asset representation to used in `trade` transactions.
    struct TradeAsset {
        id: AssetId,
        amount: u64,
        price:  u64,
    }
}