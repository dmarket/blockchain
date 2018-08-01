use exonum::storage::{Fork, MapIndex, Snapshot};
use exonum::crypto::PublicKey;

use currency::assets::{AssetId, TradeAsset, AssetBundle};
use currency::offers::{OpenOffers, Offer};
use currency::SERVICE_NAME;
use currency::wallet;
use currency::wallet::Wallet;
use currency::transactions::components::ThirdPartyFees;
use std::collections::HashMap;

/// The schema for accessing wallets data.
pub struct Schema<S>(pub S)
    where
        S: AsRef<Snapshot>;

impl<S> Schema<S>
    where
        S: AsRef<Snapshot>,
{
    /// Internal `MapIndex` with immutable access.
    pub fn index(self) -> MapIndex<S, AssetId, OpenOffers> {
        let key = SERVICE_NAME.to_string() + ".open_offers";
        MapIndex::new(key, self.0)
    }

    /// Fetch state for the specified wallet from the snapshot.
    pub fn fetch(self, asset_id: &AssetId) -> OpenOffers {
        self.index()
            .get(asset_id)
            .unwrap_or_else(|| OpenOffers::new_open_offers() )
    }
}

impl<'a> Schema<&'a mut Fork> {
    /// Internal `MapIndex` with mutable access.
    pub fn index_mut(&mut self) -> MapIndex<&mut Fork, AssetId, OpenOffers> {
        let key = SERVICE_NAME.to_string() + ".open_offers";
        MapIndex::new(key, &mut *self.0)
    }

    /// Store the new state for a wallet in the database.
    pub fn store(&mut self, asset_id: &AssetId, open_offers: OpenOffers) {
        match (open_offers.bids().len(), open_offers.asks().len()) {
            (0, 0) => self.remove(asset_id),
            (_, _) => self.index_mut().put(asset_id, open_offers),
        };
    }

    /// Remove wallet state from the database.
    pub fn remove(&mut self, asset_id: &AssetId) {
        self.index_mut().remove(asset_id);
    }
}

pub fn close_bids(
    view: &Fork,
    open_offers: &mut OpenOffers,
    asset: &TradeAsset,
    ask: &mut Offer,
    wallet: &mut Wallet,
) -> HashMap<PublicKey, Wallet>
{
    let mut wallets: HashMap<PublicKey, Wallet> = HashMap::new();
    let closed_bids = open_offers.close_bid(asset.price(), asset.amount());
    if closed_bids.len() == 0 {
        return wallets;
    }

    let mut amount = 0;
    let mut coins_back = 0u64;

    for bid in closed_bids.iter() {
        amount += bid.amount;
        coins_back += bid.amount * (asset.price() - bid.price);

        let wallet_want_fees = ThirdPartyFees::new_trade(view, &vec![TradeAsset::new(asset.id(), bid.amount, bid.price)]).unwrap().0;
        let mut sum_fee_coins = 0u64;
        for (pk, fee_coins) in wallet_want_fees {
            if pk == bid.wallet {
                continue
            }
            sum_fee_coins += fee_coins;
            let wallet = wallets.entry(pk).or_insert(wallet::Schema(view).fetch(&pk));
            *wallet = Wallet::new(wallet.balance() + fee_coins, wallet.assets());
        }


        let wallet = wallets.entry(bid.wallet).or_insert(wallet::Schema(view).fetch(&bid.wallet));
        *wallet = Wallet::new(wallet.balance() + bid.price * bid.amount - sum_fee_coins, wallet.assets());
    }

    ask.remove_amount(amount);

    let add_asset = AssetBundle::new(asset.id(), amount);
    wallet.add_assets(vec![add_asset]);

    *wallet = Wallet::new(wallet.balance() + coins_back, wallet.assets());

    wallets

}

pub fn close_asks(
    view: &Fork,
    open_offers: &mut OpenOffers,
    asset: &TradeAsset,
    bid: &mut Offer,
    buyer: &mut Wallet,
) -> HashMap<PublicKey, Wallet>
{
    let mut wallets: HashMap<PublicKey, Wallet> = HashMap::new();
    let closed_asks = open_offers.close_ask(asset.price(), asset.amount());
    if closed_asks.len() == 0 {
        return wallets;
    }

    let mut coins = 0u64;
    let mut amount = 0u64;
    for ask in closed_asks.iter() {
        coins += ask.amount * asset.price();
        amount += ask.amount;
        let wallet = wallets.entry(ask.wallet).or_insert(wallet::Schema(view).fetch(&ask.wallet));
        wallet.add_assets(vec![AssetBundle::new(asset.id(), ask.amount)]);
        *wallet = Wallet::new(wallet.balance() + (ask.price - asset.price()) * ask.amount, wallet.assets());

    }

    bid.remove_amount(amount);

    // fees
    let wallet_want_fees = ThirdPartyFees::new_trade(view, &vec![TradeAsset::new(asset.id(), amount, asset.price())]).unwrap().0;

    let mut sum_fee_coins = 0u64;
    for (pk, fee_coins) in wallet_want_fees {
        sum_fee_coins += fee_coins;
        let wallet = wallets.entry(pk).or_insert(wallet::Schema(view).fetch(&pk));
        *wallet = Wallet::new(wallet.balance() + fee_coins, wallet.assets());
    }

    *buyer = Wallet::new(buyer.balance() + coins - sum_fee_coins, buyer.assets());

    wallets
}