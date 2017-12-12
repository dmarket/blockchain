extern crate exonum;

use exonum::crypto::PublicKey;
use exonum::encoding::Field;
use service::asset::Asset;

encoding_struct! {
    struct Wallet {
        const SIZE = 48;

        field pub_key: &PublicKey [00 => 32]
        field balance: u64        [32 => 40]
        field assets:  Vec<Asset> [40 => 48]
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

#[cfg(test)]
mod tests {
    use super::Wallet;
    use service::asset::{Asset, AssetID};

    #[test]
    fn test_in_wallet_assets() {
        let (pub_key, _) = ::exonum::crypto::gen_keypair();

        let assetid1 = AssetID::from_str("67e5504410b1426f9247bb680e5fe0c8").unwrap();
        let assetid2 = AssetID::from_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap();
        let assetid3 = AssetID::from_str("8d7d6d5d4d3d2d1d2c1c2b1b4a3a2a1a").unwrap();
        let assetid4 = AssetID::from_str("8c0ef5e086bb7429f6241b0144055e76").unwrap();

        let wallet = Wallet::new(
            &pub_key,
            1000,
            vec![
                Asset::new(assetid1, 30),
                Asset::new(assetid2, 30),
                Asset::new(assetid3, 30),
            ],
        );
        assert!(wallet.in_wallet_assets(&vec![Asset::new(assetid2, 3)]));
        assert!(!wallet.in_wallet_assets(&vec![Asset::new(assetid2, 33)]));
        assert!(!wallet.in_wallet_assets(&vec![Asset::new(assetid4, 1)]));
        assert!(!wallet.in_wallet_assets(&vec![
            Asset::new(assetid1, 1),
            Asset::new(assetid4, 1),
        ]));
        assert!(!wallet.in_wallet_assets(&vec![
            Asset::new(assetid1, 1),
            Asset::new(assetid3, 31),
        ]));
    }

    #[test]
    fn test_add_assets() {
        let (pub_key, _) = ::exonum::crypto::gen_keypair();

        let assetid1 = AssetID::from_str("67e5504410b1426f9247bb680e5fe0c8").unwrap();
        let assetid2 = AssetID::from_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap();
        let assetid3 = AssetID::from_str("8d7d6d5d4d3d2d1d2c1c2b1b4a3a2a1a").unwrap();
        let assetid4 = AssetID::from_str("8c0ef5e086bb7429f6241b0144055e76").unwrap();

        let mut wallet = Wallet::new(
            &pub_key,
            1000,
            vec![
                Asset::new(assetid1, 30),
                Asset::new(assetid2, 30),
                Asset::new(assetid3, 30),
            ],
        );

        wallet.add_assets(vec![Asset::new(assetid2, 3)]);
        wallet.add_assets(vec![Asset::new(assetid4, 3)]);
        assert!(wallet.in_wallet_assets(&vec![Asset::new(assetid2, 33)]));
        assert!(wallet.in_wallet_assets(&vec![Asset::new(assetid4, 3)]));
    }

    #[test]
    fn test_del_assets() {
        let (pub_key, _) = ::exonum::crypto::gen_keypair();

        let assetid1 = AssetID::from_str("67e5504410b1426f9247bb680e5fe0c8").unwrap();
        let assetid2 = AssetID::from_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap();
        let assetid3 = AssetID::from_str("8d7d6d5d4d3d2d1d2c1c2b1b4a3a2a1a").unwrap();
        let assetid4 = AssetID::from_str("8c0ef5e086bb7429f6241b0144055e76").unwrap();

        let mut wallet = Wallet::new(
            &pub_key,
            1000,
            vec![
                Asset::new(assetid1, 30),
                Asset::new(assetid2, 30),
                Asset::new(assetid3, 30),
            ],
        );

        assert!(wallet.del_assets(&vec![Asset::new(assetid2, 15)]));
        assert!(wallet.in_wallet_assets(&vec![Asset::new(assetid2, 15)]));
        assert!(!wallet.del_assets(&vec![Asset::new(assetid4, 3)]));
        assert!(!wallet.del_assets(&vec![Asset::new(assetid3, 31)]));
        assert!(!wallet.del_assets(&vec![
            Asset::new(assetid1, 10),
            Asset::new(assetid3, 31),
        ]));
        assert!(wallet.in_wallet_assets(&vec![Asset::new(assetid1, 30)]));
    }
}
