use exonum::encoding::Field;

use currency::assets::AssetBundle;
use currency::error::Error;

encoding_struct! {
    #[derive(Eq, PartialOrd, Ord)]
    struct Wallet {
        const SIZE = 48;

        field balance: u64              [32 => 40]
        field assets:  Vec<AssetBundle> [40 => 48]
    }
}

impl Wallet {
    pub fn new_empty() -> Self {
        Wallet::new(0, Vec::new())
    }

    pub fn push_assets<I>(&mut self, new_assets: I)
    where
        I: IntoIterator<Item=AssetBundle>
    {
        let mut assets = self.assets();
        assets.extend(new_assets);
        *self = Wallet::new(
            self.balance(),
            assets
        );
    }
}

pub fn move_coins(from: &mut Wallet, to: &mut Wallet, amount: u64) -> Result<(), Error> {
    if from.balance() < amount {
        return Err(Error::InsufficientFunds)
    }

    let from_balance = from.balance() - amount;
    let to_balance = to.balance() + amount;

    Field::write(&from_balance, &mut from.raw, 32, 40);
    Field::write(&to_balance, &mut to.raw, 32, 40);

    Ok(())
}

