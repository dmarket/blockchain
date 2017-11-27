extern crate exonum;

use exonum::crypto::PublicKey;
use exonum::encoding::Field;

encoding_struct! {
    struct Asset {
        const SIZE = 12;

        field hash_id: &str [0 =>  8]
        field amount:  u32 [8 => 12]
    }
}

encoding_struct! {
    struct AssetInfo {
        const SIZE = 36;

        field creator: &PublicKey [0  => 32]
        field amount:  u32        [32 => 36]
    }
}

encoding_struct! {
    struct Wallet {
        const SIZE = 48;

        field pub_key: &PublicKey [00 => 32]
        field balance: u64        [32 => 40]
        field assets:  Vec<Asset> [40 => 48]
    }
}

impl Asset {
    pub fn is_eq(&self, other: &Asset) -> bool {
        self.hash_id() == other.hash_id()
    }

    pub fn is_available_to_transfer(&self, other: &Asset) -> bool {
        self.amount() >= other.amount()
    }
}

impl Wallet {
    pub fn increase(&mut self, amount: u64) {
        let balance = self.balance() + amount;
        Field::write(&balance, &mut self.raw, 32, 40);
    }

    pub fn decrease(&mut self, amount: u64) {
        let balance = self.balance() - amount;
        Field::write(&balance, &mut self.raw, 32, 40);
    }

    pub fn add_assets(&mut self, asset_list: Vec<Asset>) {
        let mut assets = self.assets();
        let mut new_assets = asset_list.clone();
        for (i, asset) in asset_list.into_iter().enumerate() {
            assets = assets
                .into_iter()
                .map(|a| if a.is_eq(&asset) {
                    new_assets.remove(i);
                    Asset::new(a.hash_id(), a.amount() + asset.amount())
                } else {
                    a
                })
                .collect::<Vec<_>>();
        }
        assets.extend(new_assets);
        Field::write(&assets, &mut self.raw, 40, 48);
    }

    pub fn del_assets(&mut self, asset_list: &[Asset]) -> bool {
        if !self.in_wallet_assets(asset_list) {
            return false;
        }
        let mut assets = self.assets();
        for asset in asset_list {
            assets = assets
                .into_iter()
                .filter_map(|mut a| {
                    if a.is_eq(asset) && a.is_available_to_transfer(asset) {
                        let amount = a.amount() - asset.amount();
                        if amount == 0 {
                            return None;
                        } else {
                            a = Asset::new(a.hash_id(), amount);
                        }
                    }
                    Some(a)
                })
                .collect::<Vec<_>>();
        }
        Field::write(&assets, &mut self.raw, 40, 48);
        true
    }

    fn allow_amount(&self, input_asset: &Asset) -> bool {
        self.assets().into_iter().any(|asset| {
            asset.is_eq(input_asset) && asset.is_available_to_transfer(input_asset)
        })
    }

    pub fn in_wallet_assets(&self, asset_list: &[Asset]) -> bool {
        asset_list.into_iter().all(|a| self.allow_amount(&a))
    }
}

#[test]
fn in_wallet_assets_test() {
    let (pub_key, _) = ::exonum::crypto::gen_keypair();
    let wallet = Wallet::new(
        &pub_key,
        1000,
        vec![
            Asset::new("test_hash1", 30),
            Asset::new("test_hash2", 30),
            Asset::new("test_hash3", 30),
        ],
    );
    assert!(wallet.in_wallet_assets(&vec![Asset::new("test_hash2", 3)]));
    assert!(!wallet.in_wallet_assets(&vec![Asset::new("test_hash2", 33)]));
    assert!(!wallet.in_wallet_assets(&vec![Asset::new("test_hash4", 1)]));
    assert!(!wallet.in_wallet_assets(&vec![
        Asset::new("test_hash1", 1),
        Asset::new("test_hash4", 1),
    ]));
    assert!(!wallet.in_wallet_assets(&vec![
        Asset::new("test_hash1", 1),
        Asset::new("test_hash3", 31),
    ]));
}

#[test]
fn add_assets_test() {
    let (pub_key, _) = ::exonum::crypto::gen_keypair();
    let mut wallet = Wallet::new(
        &pub_key,
        1000,
        vec![
            Asset::new("test_hash1", 30),
            Asset::new("test_hash2", 30),
            Asset::new("test_hash3", 30),
        ],
    );

    wallet.add_assets(vec![Asset::new("test_hash2", 3)]);
    wallet.add_assets(vec![Asset::new("test_hash4", 3)]);
    assert!(wallet.in_wallet_assets(&vec![Asset::new("test_hash2", 33)]));
    assert!(wallet.in_wallet_assets(&vec![Asset::new("test_hash4", 3)]));
}

#[test]
fn del_assets_test() {
    let (pub_key, _) = ::exonum::crypto::gen_keypair();
    let mut wallet = Wallet::new(
        &pub_key,
        1000,
        vec![
            Asset::new("test_hash1", 30),
            Asset::new("test_hash2", 30),
            Asset::new("test_hash3", 30),
        ],
    );

    assert!(wallet.del_assets(&vec![Asset::new("test_hash2", 15)]));
    assert!(wallet.in_wallet_assets(&vec![Asset::new("test_hash2", 15)]));
    assert!(!wallet.del_assets(&vec![Asset::new("test_hash4", 3)]));
    assert!(!wallet.del_assets(&vec![Asset::new("test_hash3", 31)]));
    assert!(!wallet.del_assets(&vec![
        Asset::new("test_hash1", 10),
        Asset::new("test_hash3", 31),
    ]));
    assert!(wallet.in_wallet_assets(&vec![Asset::new("test_hash1", 30)]));
}

