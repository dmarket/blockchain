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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeState {
    pub height: u64,
    pub is_validator: bool,
}

impl Default for NodeState {
    fn default() -> Self {
        NodeState {
            height: 0,
            is_validator: false,
        }
    }
}

lazy_static! {
    static ref INFOS: RwLock<HashMap<NodeKeys, NodeInfo>> = RwLock::new(HashMap::with_capacity(4));
    static ref STATES: RwLock<HashMap<NodeKeys, NodeState>> =
        RwLock::new(HashMap::with_capacity(4));
}

const UNABLE_TO_LOCK_ERROR: &'static str = "unable to lock";

pub fn update(keys: NodeKeys, info: NodeInfo) -> bool {
    STATES
        .write()
        .expect(UNABLE_TO_LOCK_ERROR)
        .entry(keys)
        .or_default();

    INFOS
        .write()
        .expect(UNABLE_TO_LOCK_ERROR)
        .insert(keys, info)
        .is_some()
}

pub fn state(keys: NodeKeys) -> Option<NodeState> {
    STATES
        .read()
        .expect(UNABLE_TO_LOCK_ERROR)
        .get(&keys)
        .cloned()
}

pub fn list() -> Vec<(NodeKeys, NodeInfo)> {
    INFOS
        .read()
        .expect(UNABLE_TO_LOCK_ERROR)
        .iter()
        .map(|(k, v)| (*k, v.clone()))
        .collect()
}
