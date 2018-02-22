use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use serde_json;

use currency::SERVICE_ID;

pub const CREATE_WALLET_ID: u16 = 100;
const INITIAL_BALANCE: u64 = 1_00000000;

message! {
    struct CreateWallet {
        const TYPE = SERVICE_ID;
        const ID = CREATE_WALLET_ID;
        const SIZE = 32;

        field pub_key:     &PublicKey  [00 => 32]
    }
}

impl Transaction for CreateWallet {
    fn verify(&self) -> bool {
        unimplemented!()
    }

    fn execute(&self, view: &mut Fork) {
        let _ = view;
        unimplemented!()
    }

    fn info(&self) -> serde_json::Value {
        unimplemented!()
    }
}

