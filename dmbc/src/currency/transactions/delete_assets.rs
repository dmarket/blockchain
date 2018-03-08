use std::collections::HashMap;

use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{Service, SERVICE_ID};
use currency::assets;
use currency::assets::{AssetBundle, AssetId, AssetInfo};
use currency::wallet;
use currency::error::Error;
use currency::status;
use currency::transactions::components::Fees;

/// Transaction ID.
pub const DELETE_ASSETS_ID: u16 = 400;

message! {
    /// `delete_assets` transaction.
    struct DeleteAssets {
        const TYPE = SERVICE_ID;
        const ID = DELETE_ASSETS_ID;
        const SIZE = 48;

        field pub_key:     &PublicKey       [00 => 32]
        field assets:      Vec<AssetBundle> [32 => 40]
        field seed:        u64              [40 => 48]
    }
}

impl DeleteAssets {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);

        let genesis_pub = Service::genesis_wallet();
        let creator_pub = self.pub_key();

        let mut genesis = wallet::Schema(&*view).fetch(&genesis_pub);
        let mut creator = wallet::Schema(&*view).fetch(&creator_pub);

        let fees = Fees::new_del_assets(&view, self.assets()).unwrap();

        fees.collect_to_genesis(&mut creator, &mut genesis)?;

        wallet::Schema(&mut *view).store(&genesis_pub, genesis);
        wallet::Schema(&mut *view).store(&creator_pub, creator.clone());

        creator.remove_assets(self.assets())?;

        let mut updated_infos: HashMap<AssetId, AssetInfo> = HashMap::new();
        for asset in self.assets() {
            let id = asset.id();
            let state = assets::Schema(&mut *view).fetch(&id);
            
            let info = state.map_or_else(
                || Err(Error::AssetNotFound),
                |info| info.decrease(&creator_pub, asset.amount())
            )?;

            // we should update new AssetInfo candidate not override if it 
            // already exists from previous iterations.
            updated_infos.entry(id).and_modify(|prev_info| {
                *prev_info = AssetInfo::new(
                    prev_info.creator(), 
                    prev_info.amount() - asset.amount(), 
                    prev_info.fees()
                )
            }).or_insert(info);
        }

        wallet::Schema(&mut *view).store(&creator_pub, creator);
        for (id, info) in updated_infos {
            match info.amount() {
                0 => assets::Schema(&mut *view).remove(&id),
                _ => assets::Schema(&mut *view).store(&id, info),
            }
        }

        Ok(())
    }
}

impl Transaction for DeleteAssets {
    fn verify(&self) -> bool {
        // TODO
        if cfg!(fuzzing) {
            return true;
        }

        // TODO: check assets for copies. Its important.

        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let result = self.process(view);
        status::Schema(view).store(self.hash(), result);
    }

    fn info(&self) -> serde_json::Value {
        json!({})
    }
}
