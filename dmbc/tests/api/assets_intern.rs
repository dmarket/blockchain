use api::*;
use std::collections::HashMap;
use iron::headers::Headers;
use iron_test::{request, response};

use exonum_testkit::ApiKind;

use dmbc::currency::Service;
use dmbc::currency::SERVICE_NAME;
use dmbc::currency::api::wallet;
use dmbc::currency::api::assets_intern::{AssetIdResponse, AssetIdResponseBody};
use dmbc::currency::assets::AssetId;
use dmbc::currency::api::error::ApiError;

#[test]
fn asset_id_from_meta() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let meta_data = "asset";

    let (pub_key, _) = WalletMiner::new().mine(&mut testkit);

    let response: AssetIdResponse = api.get(
        ApiKind::Service(SERVICE_NAME),
        &format!("/v1/assets/intern/{}/{}", pub_key.to_string(), meta_data),
    );

    let id = AssetId::from_data(meta_data, &pub_key);
    let mut assets = HashMap::<String, String>::new();
    assets.insert(meta_data.to_string(), id.to_string());

    assert_eq!(response, Ok(AssetIdResponseBody{ assets }));
}

#[test]
fn assets_ids_from_meta() {

}