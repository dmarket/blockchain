use std::net::SocketAddr;
use std::borrow::Cow;

use crypto;
use crypto::Hash;
use storage::StorageValue;
use messages::Connect;
use encoding::Field;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub addr: SocketAddr,
    pub connect: Connect,
}

impl PeerInfo {
    pub fn new(addr: SocketAddr, connect: Connect) -> Self {
        let target_port = connect.addr().port();
        let target_ip = addr.ip();
        let addr = SocketAddr::new(target_ip, target_port);
        PeerInfo { addr, connect }
    }
}

impl StorageValue for PeerInfo {
    fn hash(&self) -> Hash {
        crypto::hash(&self.clone().into_bytes())
    }

    fn into_bytes(self) -> Vec<u8> {
        let total_len = SocketAddr::field_size() + Connect::field_size();
        let mut bytes = Vec::with_capacity(total_len as usize);
        bytes.resize(total_len as usize, 0u8);

        self.addr.write(&mut bytes, 0, SocketAddr::field_size());
        self.connect.write(&mut bytes, SocketAddr::field_size(), total_len);

        bytes
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        let total_len = SocketAddr::field_size() + Connect::field_size();

        debug_assert_eq!(total_len as usize, value.len());

        unsafe {
            let addr = SocketAddr::read(&value, 0, SocketAddr::field_size());
            let connect = Connect::read(&value, SocketAddr::field_size(), total_len);
            PeerInfo { addr, connect }
        }
    }
}

