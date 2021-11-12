use exonum::encoding::Field;

use currency::assets::AssetBundle;
use currency::error::Error;

encoding_struct! {
    /// Wallet data.
    #[derive(Eq, PartialOrd, Ord)]
    struct Wallet {
        balance: u64,
        #[deprecated]
        assets: Vec<AssetBundle>,
    }
}

impl Wallet {
    /// Create new wallet with zero balance and no assets.
    pub fn new_empty() -> Self {
        Wallet::new(0, vec![])
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

