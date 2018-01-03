use exonum::crypto::PublicKey;

#[derive(Serialize, Deserialize)]
pub struct FuzzData {
    pub genesis: PublicKey,
    pub alice: PublicKey,
    pub bob: PublicKey,
}
