use exonum::crypto::{PublicKey, Hash};

encoding_struct! {
    #[derive(Eq, PartialOrd, Ord)]
    struct Offer {
        wallet: &PublicKey,
        amount: u64,
        tx_hash: &Hash,
    }
}

impl Offer {
    pub fn remove_amount(&mut self, amount: u64) {
        *self = Offer::new(self.wallet(), self.amount() - amount, &self.tx_hash());
    }

    pub fn add_amount(&mut self, amount: u64) {
        *self = Offer::new(self.wallet(), self.amount() + amount, &self.tx_hash());
    }
}

