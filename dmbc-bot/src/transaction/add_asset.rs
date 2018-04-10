extern crate exonum;

use dmbc::currency::assets::MetaAsset;
use dmbc::currency::transactions::builders::transaction;
use Wallet;

use transaction::serialize;

pub fn create_add_asset_tx(wallet: Wallet, assets: Vec<MetaAsset>) -> String {
    let tx = transaction::Builder::new()
        .keypair(wallet.public, wallet.secret.clone())
        .tx_add_assets()
        .add_assets_value(assets)
        .build();

    serialize(tx)
}
