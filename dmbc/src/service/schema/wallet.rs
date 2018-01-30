extern crate exonum;

use exonum::crypto::PublicKey;
use exonum::storage::{Fork, MapIndex};

use service::SERVICE_NAME;
use service::wallet::Wallet;
use service::asset::Asset;

pub struct WalletSchema<'a>(&'a mut Fork);

impl<'a> WalletSchema<'a> {
    pub fn wallets(&mut self) -> MapIndex<&mut Fork, PublicKey, Wallet> {
        let key = SERVICE_NAME.to_string().replace("/", "_") + ".wallets";
        MapIndex::new(key, self.0)
    }

    // Utility method to quickly get a separate wallet from the storage.
    // If wallet doesn't exist, create new one
    pub fn wallet(&mut self, pub_key: &PublicKey) -> Wallet {
        match self.wallets().get(pub_key) {
            Some(wallet) => wallet,
            None => Wallet::default(),
        }
    }

    pub fn map<F, T>(view: &'a mut Fork, f: F) -> T
    where
        F: FnOnce(Self) -> T + 'a,
        T: 'a,
    {
        f(WalletSchema(view))
    }

    pub fn add_assets(view: &mut Fork, wallet: &mut Wallet, assets: &[Asset]) -> Result<(), ()> {
        wallet.add_assets(assets);
        Self::map(view, |mut schema| {
            schema.wallets().put(wallet.pub_key(), wallet.clone())
        });
        Ok(())
    }

    pub fn delete_assets(view: &mut Fork, wallet: &mut Wallet, assets: &[Asset]) -> Result<(), ()> {
        if !wallet.del_assets(assets) {
            return Err(());
        }

        // Remove wallet from db if it is empty completely, otherwise update db with changed wallet
        Self::map(view, |mut schema| match wallet.is_empty() {
            true => schema.wallets().remove(wallet.pub_key()),
            false => schema.wallets().put(wallet.pub_key(), wallet.clone()),
        });
        Ok(())
    }

    pub fn transfer_coins(
        view: &mut Fork,
        sender: &mut Wallet,
        receiver: &mut Wallet,
        coins: u64,
    ) -> Result<(), ()> {
        if sender.balance() < coins {
            return Err(());
        }

        sender.decrease(coins);
        receiver.increase(coins);

        // store changes
        Self::map(view, |mut schema| {
            schema.wallets().put(sender.pub_key(), sender.clone());
            schema.wallets().put(receiver.pub_key(), receiver.clone());
        });
        Ok(())
    }

    pub fn transfer_assets(
        view: &mut Fork,
        sender: &mut Wallet,
        receiver: &mut Wallet,
        assets: &[Asset],
    ) -> Result<(), ()> {
        if !sender.is_assets_in_wallet(&assets) {
            return Err(());
        }

        sender.del_assets(&assets);
        receiver.add_assets(&assets);

        // store changes
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(sender.pub_key(), sender.clone());
            schema.wallets().put(receiver.pub_key(), receiver.clone());
        });
        Ok(())
    }

    pub fn exchange_assets(
        view: &mut Fork,
        part_a: &mut Wallet,
        part_b: &mut Wallet,
        assets_a: &[Asset],
        assets_b: &[Asset],
    ) -> Result<(), ()> {
        if !part_a.is_assets_in_wallet(&assets_a) || !part_b.is_assets_in_wallet(&assets_b) {
            return Err(());
        }

        part_a.del_assets(&assets_a);
        part_b.add_assets(&assets_a);

        part_b.del_assets(&assets_b);
        part_a.add_assets(&assets_b);

        // store changes
        WalletSchema::map(view, |mut schema| {
            schema.wallets().put(part_a.pub_key(), part_a.clone());
            schema.wallets().put(part_b.pub_key(), part_b.clone());
        });
        Ok(())
    }

    pub fn get_wallet(view: &mut Fork, pub_key: &PublicKey) -> Wallet {
        Self::map(view, |mut schema| schema.wallet(pub_key))
    }
}

#[cfg(test)]
mod tests {
    use exonum::crypto;
    use exonum::storage::MemoryDB;
    use exonum::storage::Database;

    use service::asset::{Asset, AssetId};
    use service::builders::wallet;
    use service::schema::wallet::WalletSchema;

    #[test]
    fn test_transfer_coins() {
        let (sender_public, _) = crypto::gen_keypair();
        let (recipient_public, _) = crypto::gen_keypair();

        let mut sender = wallet::Builder::new()
            .key(sender_public)
            .balance(100)
            .build();

        let mut recipient = wallet::Builder::new()
            .key(recipient_public)
            .balance(100)
            .build();

        let db = Box::new(MemoryDB::new());
        let fork = &mut db.fork();

        WalletSchema::map(fork, |mut s| {
            s.wallets().put(&sender.pub_key(), sender.clone());
            s.wallets().put(&recipient.pub_key(), recipient.clone());
        });

        assert!(WalletSchema::transfer_coins(fork, &mut sender, &mut recipient, 200).is_err());
        assert!(WalletSchema::transfer_coins(fork, &mut sender, &mut recipient, 100).is_ok());

        let (sender, recipient) = WalletSchema::map(fork, |mut s| {
            (s.wallet(&sender_public), s.wallet(&recipient_public))
        });

        assert_eq!(sender.balance(), 0);
        assert_eq!(recipient.balance(), 200);
    }

    #[test]
    fn test_transfer_assets() {
        let (sender_public, _) = crypto::gen_keypair();
        let (recipient_public, _) = crypto::gen_keypair();

        let sender_data = "sender asset";
        let sender_id = AssetId::new(sender_data, &sender_public).unwrap();
        let sender_asset = Asset::new(sender_id, 100);

        let absent_data = "absent data";
        let absent_id = AssetId::new(absent_data, &sender_public).unwrap();
        let absent_asset = Asset::new(absent_id, 100);

        let mut sender = wallet::Builder::new()
            .key(sender_public)
            .add_asset_value(sender_asset.clone())
            .build();

        let mut recipient = wallet::Builder::new().key(recipient_public).build();

        let db = Box::new(MemoryDB::new());
        let fork = &mut db.fork();

        WalletSchema::map(fork, |mut s| {
            s.wallets().put(&sender.pub_key(), sender.clone());
            s.wallets().put(&recipient.pub_key(), recipient.clone());
        });

        assert!(
            WalletSchema::transfer_assets(fork, &mut sender, &mut recipient, &[absent_asset])
                .is_err()
        );

        assert!(
            WalletSchema::transfer_assets(
                fork,
                &mut sender,
                &mut recipient,
                &[sender_asset.clone()]
            ).is_ok()
        );

        let (sender, recipient) = WalletSchema::map(fork, |mut s| {
            (s.wallet(&sender_public), s.wallet(&recipient_public))
        });

        assert!(sender.asset(sender_id).is_none());
        assert_eq!(recipient.asset(sender_id).unwrap(), sender_asset);
    }

    #[test]
    fn test_exchange_assets() {
        let (sender_public, _) = crypto::gen_keypair();
        let (recipient_public, _) = crypto::gen_keypair();

        let sender_data = "sender asset";
        let sender_id = AssetId::new(sender_data, &sender_public).unwrap();
        let sender_asset = Asset::new(sender_id, 100);

        let recipient_data = "recipient data";
        let recipient_id = AssetId::new(recipient_data, &sender_public).unwrap();
        let recipient_asset = Asset::new(recipient_id, 100);

        let mut sender = wallet::Builder::new()
            .key(sender_public)
            .add_asset_value(sender_asset.clone())
            .build();

        let mut recipient = wallet::Builder::new()
            .key(recipient_public)
            .add_asset_value(recipient_asset.clone())
            .build();

        let db = Box::new(MemoryDB::new());
        let fork = &mut db.fork();

        WalletSchema::map(fork, |mut s| {
            s.wallets().put(&sender.pub_key(), sender.clone());
            s.wallets().put(&recipient.pub_key(), recipient.clone());
        });

        assert!(
            WalletSchema::exchange_assets(
                fork,
                &mut sender,
                &mut recipient,
                &[sender_asset.clone()],
                &[recipient_asset.clone()]
            ).is_ok()
        );

        let (sender, recipient) = WalletSchema::map(fork, |mut s| {
            (s.wallet(&sender_public), s.wallet(&recipient_public))
        });

        assert!(sender.asset(sender_id).is_none());
        assert!(recipient.asset(recipient_id).is_none());
        assert_eq!(sender.asset(recipient_id).unwrap(), recipient_asset);
        assert_eq!(recipient.asset(sender_id).unwrap(), sender_asset);
    }
}
