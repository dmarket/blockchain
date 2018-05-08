use exonum::crypto::{PublicKey, SecretKey};

pub struct Builder {
    public_key: Option<PublicKey>,
    secret_key: Option<SecretKey>,
    network_id: u8,
    protocol_version: u8,
    service_id: u16,
}