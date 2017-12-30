use exonum::crypto::PublicKey;
use exonum::storage::{Fork, MapIndex};

use service::SERVICE_NAME;
use service::asset::{Asset, AssetId, AssetInfo, Fees};

pub struct AssetSchema<'a>(&'a mut Fork);

impl<'a> AssetSchema<'a> {
    pub fn assets(&mut self) -> MapIndex<&mut Fork, AssetId, AssetInfo> {
        let name = SERVICE_NAME.to_string().replace("/", "_") + ".assets";
        MapIndex::new(name, self.0)
    }

    pub fn info(&mut self, asset_id: &AssetId) -> Option<AssetInfo> {
        self.assets().get(&asset_id)
    }

    pub fn add_asset(
        &mut self,
        asset_id: &AssetId,
        creator: &PublicKey,
        amount: u32,
        fees: Fees,
    ) -> bool {
        match self.info(&asset_id) {
            None => {
                let info = AssetInfo::new(creator, amount, fees);
                self.assets().put(&asset_id, info);
                println!("Add asset {:?} for wallet: {:?}", asset_id, creator);
                true
            }
            Some(info) => {
                if info.creator() != creator { return false }
                let info = AssetInfo::new(creator, info.amount() + amount, fees);
                self.assets().put(&asset_id, info);
                true
            }
        }
    }

    pub fn add_assets(&mut self, assets: &Vec<Asset>, fees_list: &Vec<Fees>, pub_key: &PublicKey) -> bool {
        let assets_and_fees = assets.iter().zip(fees_list);
        for (asset, fees) in assets_and_fees {
            if !self.add_asset(&asset.id(), pub_key, asset.amount(), fees.clone()) {
                return false
            }
        }

        true
    }

    pub fn del_assets(&mut self, deleted: &[Asset]) {
        let mut infos = self.assets();
        for asset in deleted {
            let info = match infos.get(&asset.id()) {
                Some(info) => info,
                _ => continue,
            };
            let amount = info.amount() - asset.amount();
            let info = AssetInfo::new(info.creator(), amount, info.fees());
            match info.amount() {
                0 => infos.remove(&asset.id()),
                _ => infos.put(&asset.id(), info),
            }
        }
    }

    pub fn map<F, T>(view: &'a mut Fork, f: F) -> T
        where F: FnOnce(Self) -> T + 'a, T: 'a
    {
        f(AssetSchema(view))
    }
}
