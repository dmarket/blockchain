use std::collections::HashMap;
use std::error::Error;
use std::thread;

use curl::easy::Easy;
use exonum::crypto::PublicKey;
use serde_json;

use dmbc::config;

// TODO: NodeKeys and ValidatorInfo duplicate structures
//       in the dmbc-discovery crate.
//       Put them into common module.

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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ValidatorInfo(pub NodeKeys, pub NodeInfo);

pub fn connect(
    info: &ValidatorInfo,
    is_validator: bool,
) -> Result<HashMap<NodeKeys, NodeInfo>, Box<Error>> {
    let discovery = config::config().service_discovery().address();

    let mut nodes = receive_nodes(&discovery)?;
    if !is_validator {
        return Ok(nodes);
    }

    nodes.remove(&info.0);

    send_node(&discovery, info)?;

    Ok(nodes)
}

fn receive_nodes(discovery: &str) -> Result<HashMap<NodeKeys, NodeInfo>, Box<Error>> {
    let mut nodes_get = Vec::new();

    let mut handle = Easy::new();
    handle.url(discovery).map_err(Box::new)?;
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|data| {
                nodes_get.extend_from_slice(data);
                Ok(data.len())
            })
            .map_err(Box::new)?;
        transfer.perform().map_err(Box::new)?;
    }

    serde_json::from_slice(&nodes_get)
        .map(|c: Vec<(NodeKeys, NodeInfo)>| c.into_iter().collect())
        .map_err(|e| e.into())
}

fn send_node(discovery: &str, info: &ValidatorInfo) -> Result<(), Box<Error>> {
    let node_post = serde_json::to_string(info).map_err(Box::new)?;

    let mut handle = Easy::new();
    handle.url(discovery).map_err(Box::new)?;
    handle.post(true).map_err(Box::new)?;
    handle
        .post_fields_copy(node_post.as_bytes())
        .map_err(Box::new)?;
    thread::spawn(move || match handle.perform() {
        Err(e) => eprintln!("Error in send_node(): {}", e),
        _ => (),
    });

    Ok(())
}
