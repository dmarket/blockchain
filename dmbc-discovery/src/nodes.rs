use std::collections::HashMap;
use std::sync::RwLock;

use exonum::crypto::PublicKey;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeKeys {
    pub consensus: PublicKey,
    pub service: PublicKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeInfo {
    pub public: String,
    pub private: String,
    pub peer: String,
}

lazy_static! {
    static ref NODES: RwLock<HashMap<NodeKeys, NodeInfo>> = RwLock::new(HashMap::with_capacity(4));
}

const UNABLE_TO_LOCK_ERROR: &'static str = "unable to lock";

pub fn put(keys: NodeKeys, info: NodeInfo) -> bool {
    NODES
        .write()
        .expect(UNABLE_TO_LOCK_ERROR)
        .insert(keys, info)
        .is_some()
}

pub fn get(keys: NodeKeys) -> Option<NodeInfo> {
    NODES
        .read()
        .expect(UNABLE_TO_LOCK_ERROR)
        .get(&keys)
        .cloned()
}

pub fn has(keys: NodeKeys) -> bool {
    NODES
        .read()
        .expect(UNABLE_TO_LOCK_ERROR)
        .get(&keys)
        .is_some()
}

pub fn list() -> HashMap<NodeKeys, NodeInfo> {
    NODES.read().expect(UNABLE_TO_LOCK_ERROR).clone()
}
