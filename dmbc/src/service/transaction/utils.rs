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
