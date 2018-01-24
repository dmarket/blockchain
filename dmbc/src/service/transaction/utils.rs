extern crate exonum;
use exonum::crypto::PublicKey;
use exonum::storage::Fork;

use service::asset::Asset;
use service::wallet::Wallet;
use service::schema::wallet::WalletSchema;

encoding_struct! {
    struct Intermediary {
        const SIZE = 40;

        field wallet:       &PublicKey [0 => 32]
        field commision:    u64        [32 => 40]
    }
}

pub fn get_wallet<'a>(view: &mut Fork, pub_key: &PublicKey) -> Wallet {
    WalletSchema::map(view, |mut schema| schema.wallet(pub_key))
}

pub fn transfer_coins(
    view: &mut Fork,
    sender: &mut Wallet,
    receiver: &mut Wallet,
    coins: u64,
) -> bool {
    if !sender.is_sufficient_funds(coins) {
        return false;
    }

    sender.decrease(coins);
    receiver.increase(coins);

    // store changes
    WalletSchema::map(view, |mut schema| {
        schema.wallets().put(sender.pub_key(), sender.clone());
        schema.wallets().put(receiver.pub_key(), receiver.clone());
    });
    true
}

pub fn split_coins(coins: u64) -> (u64, u64) {
    let first_half = (coins as f64 / 2.0).ceil() as u64;
    let second_half = coins - first_half;
    (first_half, second_half)
}

pub fn transfer_assets(
    view: &mut Fork,
    sender: &mut Wallet,
    receiver: &mut Wallet,
    assets: &[Asset],
) -> bool {
    if !sender.is_assets_in_wallet(&assets) {
        return false;
    }

    sender.del_assets(&assets);
    receiver.add_assets(&assets);

    // store changes
    WalletSchema::map(view, |mut schema| {
        schema.wallets().put(sender.pub_key(), sender.clone());
        schema.wallets().put(receiver.pub_key(), receiver.clone());
    });
    true
}

pub fn exchange_assets(
    view: &mut Fork,
    part_a: &mut Wallet,
    part_b: &mut Wallet,
    assets_a: &[Asset],
    assets_b: &[Asset],
) -> bool {
    if !part_a.is_assets_in_wallet(&assets_a) || !part_b.is_assets_in_wallet(&assets_b) {
        return false;
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
    true
}

#[cfg(test)]
mod tests {
    use exonum::crypto;
    use exonum::storage::MemoryDB;
    use exonum::storage::Database;

    use service::asset::{Asset, AssetId};
    use service::builders::wallet;
    use service::schema::wallet::WalletSchema;
    use super::transfer_coins;
    use super::transfer_assets;
    use super::exchange_assets;
    use super::split_coins;

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

        assert!(!transfer_coins(fork, &mut sender, &mut recipient, 200));
        assert!(transfer_coins(fork, &mut sender, &mut recipient, 100));

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

        assert!(!transfer_assets(
            fork,
            &mut sender,
            &mut recipient,
            &[absent_asset]
        ));

        assert!(transfer_assets(
            fork,
            &mut sender,
            &mut recipient,
            &[sender_asset.clone()]
        ));

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

        assert!(exchange_assets(
            fork,
            &mut sender,
            &mut recipient,
            &[sender_asset.clone()],
            &[recipient_asset.clone()]
        ));

        let (sender, recipient) = WalletSchema::map(fork, |mut s| {
            (s.wallet(&sender_public), s.wallet(&recipient_public))
        });

        assert!(sender.asset(sender_id).is_none());
        assert!(recipient.asset(recipient_id).is_none());
        assert_eq!(sender.asset(recipient_id).unwrap(), recipient_asset);
        assert_eq!(recipient.asset(sender_id).unwrap(), sender_asset);
    }

    #[test]
    fn test_split_coins() {
        let (a, b) = split_coins(10);
        assert!(a == b);

        let (a, b) = split_coins(11);
        assert!(a == 6);
        assert!(b == 5);
    }
}
