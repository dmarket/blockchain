//! Types and operations on open-offers in the blockchain network.

mod schema;
mod offer;
mod offers;
mod open_offers;

pub use currency::offers::schema::Schema;
pub use currency::offers::open_offers::OpenOffers;
pub use currency::offers::offer::Offer;
pub use currency::offers::offers::{Offers, CloseOffer};


use exonum::crypto::PublicKey;
use exonum::storage::Fork;
use currency::transactions::components::ThirdPartyFees;
use currency::assets::{AssetBundle,TradeAsset};
use currency::wallet;
use currency::wallet::Wallet;
use std::collections::HashMap;


pub fn close_bids(
    view: &Fork,
    open_offers: &mut OpenOffers,
    asset: &TradeAsset,
    ask: &mut Offer,
    wallet: &mut Wallet,
) -> HashMap<PublicKey, Wallet>
{
    let mut wallets = HashMap::new();
    let closed_bids = open_offers.close_bid(asset.price(), asset.amount());
    if closed_bids.len() == 0 {
        return wallets;
    }

    let mut amount = 0;
    let mut coins_back = 0u64;

    for bid in &closed_bids {
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
    let mut wallets = HashMap::new();
    let closed_asks = open_offers.close_ask(asset.price(), asset.amount());
    if closed_asks.len() == 0 {
        return wallets;
    }

    let mut coins = 0u64;
    let mut amount = 0u64;
    for ask in &closed_asks {
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