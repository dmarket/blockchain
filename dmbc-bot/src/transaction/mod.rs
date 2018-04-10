pub mod add_asset;

use serde::Serialize;
use serde_json;

pub fn serialize<S: Serialize>(tx: S) -> String {
    serde_json::to_string(&tx).unwrap()
}
