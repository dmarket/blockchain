pub mod add_asset;

use serde_json;
use serde::Serialize;

pub fn serialize<S: Serialize>(tx: S) -> String {
    serde_json::to_string(&tx).unwrap()
}
