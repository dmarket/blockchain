use exonum::encoding::Field;

use currency::assets::AssetBundle;
use currency::error::Error;

encoding_struct! {
    /// Wallet data.
    #[derive(Eq, PartialOrd, Ord)]
    struct Wallet {
        const SIZE = 16;

        field balance: u64              [0 => 8]
        field assets:  Vec<AssetBundle> [8 => 16]
    }
}

impl Wallet {
    /// Create new wallet with zero balance and no assets.
    pub fn new_empty() -> Self {
        Wallet::new(0, Vec::new())
    }

    /// Push assets into the wallet.
    pub fn push_assets<I>(&mut self, new_assets: I)
    where
        I: IntoIterator<Item = AssetBundle>,
    {
        let mut assets = self.assets();
        for new in new_assets {
            if let Some(index) = assets.iter_mut().position(|a| a.id() == new.id()) {
                let asset = &mut assets[index];
                let new_amount = asset.amount() + new.amount();
                *asset = AssetBundle::new(asset.id(), new_amount);
            } else {
                assets.push(new);
            }
        }
        *self = Wallet::new(self.balance(), assets);
    }
}

/// Move funds between wallets.
///
/// # Errors
/// 
/// Returns `InsufficientFunds` if the `from` wallet balance is less than `amount`.
pub fn move_coins(from: &mut Wallet, to: &mut Wallet, amount: u64) -> Result<(), Error> {
    if from.balance() < amount {
        return Err(Error::InsufficientFunds);
    }

    let from_balance = from.balance() - amount;
    let to_balance = to.balance() + amount;

    Field::write(&from_balance, &mut from.raw, 0, 8);
    Field::write(&to_balance, &mut to.raw, 0, 8);

    Ok(())
}

/// Move assets between wallets.
///
/// # Errors
///
/// Returns `InsufficientFunds` if the wallets in `move_specs` are not present
/// in the `from` wallet in the specified quantity.
pub fn move_assets(
    from: &mut Wallet,
    to: &mut Wallet,
    move_specs: &[AssetBundle],
) -> Result<(), Error> {
    let mut from_assets = from.assets();
    let mut to_assets = to.assets();

    for spec in move_specs {
        let from_asset = match from_assets.iter_mut().find(|a| a.id() == spec.id()) {
            Some(asset) => {
                if asset.amount() < spec.amount() {
                    return Err(Error::InsufficientAssets);
                }
                asset
            }
            None => return Err(Error::InsufficientAssets),
        };

        let to_asset = match to_assets.iter_mut().position(|a| a.id() == spec.id()) {
            Some(index) => &mut to_assets[index],
            None => {
                to_assets.push(AssetBundle::new(spec.id(), 0));
                to_assets.last_mut().unwrap()
            }
        };

        *from_asset = AssetBundle::new(spec.id(), from_asset.amount() - spec.amount());
        *to_asset = AssetBundle::new(spec.id(), to_asset.amount() + spec.amount());
    }

    return Ok(());
}
