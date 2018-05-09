use exonum::crypto::{PublicKey, SecretKey};
use capi;

pub struct Builder {
    public_key: Option<PublicKey>,
    secret_key: Option<SecretKey>,
    network_id: u8,
    protocol_version: u8,
    service_id: u16,
    message_type: u16,
}

struct TransactionMetadata {
    public_key: PublicKey,
    secret_key: SecretKey,
    network_id: u8,
    protocol_version: u8,
    service_id: u16,
    message_type: u16,
}

impl From<Builder> for TransactionMetadata {
    fn from(b: Builder) -> Self {
        TransactionMetadata {
            public_key: b.public_key.unwrap(),
            secret_key: b.secret_key.unwrap(),
            network_id: b.network_id,
            protocol_version: b.protocol_version,
            service_id: b.service_id,
            message_type: b.message_type,
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            public_key: None,
            secret_key: None,
            network_id: 0,
            protocol_version: 0,
            service_id: capi::SERVICE_ID,
            message_type: 0,
        }
    }

    pub fn keypair(self, public_key: PublicKey, secret_key: SecretKey) -> Self {
        Builder {
            public_key: Some(public_key),
            secret_key: Some(secret_key),
            ..self
        }
    }

    pub fn network_id(self, network_id: u8) -> Self {
        Builder { network_id, ..self }
    }

    pub fn protocol_version(self, protocol_version: u8) -> Self {
        Builder {
            protocol_version,
            ..self
        }
    }

    pub fn service_id(self, service_id: u16) -> Self {
        Builder { service_id, ..self }
    }

    pub fn message_type(self, message_type: u16) -> Self {
        Builder { message_type, ..self }
    }

    fn validate(&self) -> Result<(), ()> {
        match (&self.public_key, &self.secret_key) {
            (&Some(_), &Some(_)) => Ok(()),
            _ => Err(()),
        }
    }
}