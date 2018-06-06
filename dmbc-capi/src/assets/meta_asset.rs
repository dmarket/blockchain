use exonum::crypto::PublicKey; 

use assets::Fees;

encoding_struct! {
    /// Info for asset to be committed into the network in `add_assets` transaction.
    struct MetaAsset {
        receiver:  &PublicKey,
        data:      &str,
        amount:    u64,
        fees:      Fees,
    }
}
