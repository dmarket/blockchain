extern crate exonum;
extern crate rand;

use exonum::crypto::PublicKey;
use dmbc::currency::assets::MetaAsset;
use dmbc::currency::transactions::builders::fee;
use rand::Rng;

const MAX_AMOUNT: u64 = 10_000;
const MAX_GEN_ASSETS: u8 = 5;

pub fn generate_meta_assets(pk:PublicKey) -> Vec<MetaAsset> {
    let mut rng = rand::thread_rng();
    let mut assets: Vec<MetaAsset> = Vec::new();
    let get_count_assets = rng.gen_range(0, MAX_GEN_ASSETS);
    for _i in 0..get_count_assets {
        let amount = rng.gen_range(0, MAX_AMOUNT);
        let fees = fee::Builder::new()
            .trade(10, 10)
            .exchange(10, 10)
            .transfer(10, 10)
            .build();
        let rnd_name = rng
            .gen_ascii_chars()
            .take(10)
            .collect::<String>();
        assets.push(MetaAsset::new(&pk, &rnd_name, amount, fees));
    }

    assets
}
