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
            let mut is_add = false;

            for i in 0..assets.len() {
                if assets[i].hash_id() == asset.hash_id() {
                    let amount = asset.amount() + assets[i].amount();
                    assets[i] = Asset::new(asset.hash_id(), amount);
                    is_add = true;
                    break;
                }
            }

            if !is_add {
                assets.push(asset);
            }
        }
        Field::write(&assets, &mut self.raw, 40, 48);
    }

    pub fn del_assets(&mut self, asset_list: Vec<Asset>) -> bool {
        let mut assets = self.assets();
        for asset in asset_list {
            let mut is_del = false;
            //for (i, a) in assets.iter_mut().enumerate() {
            for i in 0..assets.len() {
                if assets[i].hash_id() == asset.hash_id() && assets[i].amount() >= asset.amount() {
                    let amount = assets[i].amount() - asset.amount();
                    if amount == 0 {
                        assets.remove(i);
                    } else {
                        assets[i] = Asset::new(asset.hash_id(), amount);
                    }
                    is_del = true;
                    break;
                }
            }
            if !is_del {
                return false;
            }
        }
        Field::write(&assets, &mut self.raw, 40, 48);
        true
    }

    fn allow_amount(&self, input_asset: &Asset) -> bool {
        self.assets().into_iter()
            .filter(|asset| asset.hash_id() == input_asset.hash_id() && asset.amount() >= input_asset.amount())
            .collect::<Vec<_>>()
            .is_empty()
    }

    pub fn in_wallet_assets(&self, asset_list: Vec<Asset>) -> bool {
        !asset_list.into_iter().filter(|a| self.allow_amount(a)).collect::<Vec<_>>().is_empty()
    }
}
