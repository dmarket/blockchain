use assets::AssetId;

encoding_struct! {
    /// A bundle of assets with the same id.
    struct AssetBundle {
        id:     AssetId,
        amount: u64,
    }
}