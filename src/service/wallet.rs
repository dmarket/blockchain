extern crate exonum;

use exonum::crypto::PublicKey;
use exonum::encoding::Field;

encoding_struct!{
    struct Asset {
        const SIZE = 12;

        field hash_id:    &str      [00 => 8]
        field amount:      u32      [8 => 12]
    }
}

encoding_struct! {
    struct Wallet {
        const SIZE = 48;

        field pub_key:            &PublicKey          [00 => 32]
        field balance:            u64                 [32 => 40]
        field assets:             Vec<Asset>          [40 => 48]
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
        for asset in asset_list {
            assets = assets.into_iter().map(|a| {
                if a.is_eq(&asset) {
                    Asset::new(a.hash_id(), a.amount() + asset.amount())
                } else {
                    a
                }
            }).collect::<Vec<_>>();
        }
        Field::write(&assets, &mut self.raw, 40, 48);
    }

    pub fn del_assets(&mut self, asset_list: Vec<Asset>) -> bool {
        let result = self.in_wallet_assets(asset_list.clone());
        let mut assets = self.assets();
        for asset in asset_list {
            assets = assets.into_iter().filter_map(|mut a| {
                if a.is_eq(&asset) && a.is_available_to_transfer(&asset) {
                    let amount = a.amount() - asset.amount();
                    if amount == 0 {
                        return None;
                    } else {
                        a = Asset::new(a.hash_id(), amount);
                    }
                }
                Some(a)
            }).collect::<Vec<_>>();
        }
        Field::write(&assets, &mut self.raw, 40, 48);
        result
    }

    fn allow_amount(&self, input_asset: &Asset) -> bool {
        self.assets().into_iter()
            .any(|asset| asset.is_eq(input_asset) && asset.is_available_to_transfer(input_asset))
    }

    pub fn in_wallet_assets(&self, asset_list: Vec<Asset>) -> bool {
        !asset_list.into_iter().any(|a| self.allow_amount(&a))
    }
}
