use std::collections::HashMap;

use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;

use currency::{Service, SERVICE_ID};
use currency::assets;
use currency::assets::AssetBundle;
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

        let fees = Fees::new_delete_assets(&view, self.assets()).unwrap();

        fees.collect_to_genesis(&mut creator, &mut genesis)?;

        wallet::Schema(&mut*view).store(&genesis_pub, genesis);
        wallet::Schema(&mut*view).store(&creator_pub, creator.clone());

        let mut infos = HashMap::new();

        for asset in self.assets() {
            let info = match assets::Schema(&*view).fetch(&asset.id()) {
                Some(info) => info,
                None => return Err(Error::AssetNotFound),
            };
            if info.creator() != creator_pub {
                return Err(Error::InvalidTransaction);
            }
            let mut entry = infos.remove(&asset.id()).unwrap_or(info);
            entry.decrease(asset.amount())?;
        }

        creator.remove_assets(self.assets())?;

        wallet::Schema(&mut*view).store(creator_pub, creator);

        for (id, info) in infos {
            assets::Schema(&mut*view).store(&id, info);
        }

        Ok(())
    }
}

impl Transaction for DeleteAssets {
    fn verify(&self) -> bool {
        if cfg!(fuzzing) {
            return true;
        }

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
