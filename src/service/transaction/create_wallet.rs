extern crate exonum;

use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::crypto::PublicKey;
use exonum::messages::Message;

use super::{SERVICE_ID, TX_CREATE_WALLET_ID, INIT_BALANCE};
use service::wallet::{Wallet, Asset};
use service::schema::currency::CurrencySchema;

message! {
    struct TxCreateWallet {
        const TYPE = SERVICE_ID;
        const ID = TX_CREATE_WALLET_ID;
        const SIZE = 32;

        field pub_key:     &PublicKey  [00 => 32]
    }
}

impl Transaction for TxCreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) {
        let mut schema = CurrencySchema { view };
        if schema.wallet(self.pub_key()).is_none() {
            let asset = Asset::new("123123", 10);
            let wallet = Wallet::new(self.pub_key(), INIT_BALANCE, vec!(asset));
            println!("Create the wallet: {:?}", wallet);
            schema.wallets().put(self.pub_key(), wallet)
        }
    }
}
