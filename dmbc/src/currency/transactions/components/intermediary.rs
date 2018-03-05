use exonum::crypto::PublicKey;

encoding_struct! {
    struct Intermediary {
        const SIZE = 40;

        field wallet:    &PublicKey [0  => 32]
        field commission: u64       [32 => 40]
    }
}

