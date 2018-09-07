use exonum::crypto::PublicKey;

evo_encoding_struct! {
    /// Intermediary specification for `_intermediary` transactions.
    struct Intermediary {
        wallet:    &PublicKey,
        commission: u64,
    }
}
