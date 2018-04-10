use exonum::crypto::PublicKey;
use exonum::blockchain::Transaction;
use exonum::storage::Fork;
use exonum::messages::Message;
use serde_json;
use prometheus::{Counter, Histogram};

use currency::SERVICE_ID;
use currency::error::Error;
use currency::wallet;
use currency::wallet::Wallet;
use currency::status;

/// Transaction ID.
pub const MINE_ID: u16 = 700;
const MINE_AMOUNT: u64 = 1_00000000;

message! {
    struct Mine {
        const TYPE = SERVICE_ID;
        const ID = MINE_ID;
        const SIZE = 40;

        field pub_key: &PublicKey [00 => 32]
        field seed:    u64        [32 => 40]
    }
}

impl Mine {
    fn process(&self, view: &mut Fork) -> Result<(), Error> {
        info!("Processing tx: {:?}", self);
        let wallet = wallet::Schema(&*view).fetch(self.pub_key());
        let wallet = Wallet::new(wallet.balance() + MINE_AMOUNT, wallet.assets());
        wallet::Schema(&mut *view).store(&self.pub_key(), wallet);
        Ok(())
    }
}

lazy_static! {
    static ref VERIFY_COUNT: Counter = register_counter!(
        "dmbc_transaction_mine_verify_count",
        "Times .verify() was called on a transaction."
    ).unwrap();
    static ref VERIFY_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_mine_verify_success_count",
        "Times verification was successfull on a transaction."
    ).unwrap();
    static ref EXECUTE_COUNT: Counter = register_counter!(
        "dmbc_transaction_mine_execute_count",
        "Transactions executed."
    ).unwrap();
    static ref EXECUTE_SUCCESS_COUNT: Counter = register_counter!(
        "dmbc_transaction_mine_execute_success_count",
        "Times transaction execution reported a success."
    ).unwrap();
    static ref EXECUTE_FINISH_COUNT: Counter = register_counter!(
        "dmbc_transaction_mine_execute_finish_count",
        "Times transaction has finished executing without panicking."
    ).unwrap();
    static ref EXECUTE_DURATION: Histogram = register_histogram!(
        "dmbc_transaction_mine_execute_duration_seconds",
        "Duration of transaction execution."
    ).unwrap();
}

impl Transaction for Mine {
    fn verify(&self) -> bool {
        VERIFY_COUNT.inc();

        if cfg!(fuzzing) {
            return true;
        }

        if self.verify_signature(self.pub_key()) {
            VERIFY_SUCCESS_COUNT.inc();
            true
        } else {
            false
        }
    }

    fn execute(&self, view: &mut Fork) {
        EXECUTE_COUNT.inc();
        let timer = EXECUTE_DURATION.start_timer();

        let result = self.process(view);

        if let &Ok(_) = &result {
            EXECUTE_SUCCESS_COUNT.inc();
        }

        status::Schema(view).store(self.hash(), result);

        timer.observe_duration();
        EXECUTE_FINISH_COUNT.inc();
    }

    fn info(&self) -> serde_json::Value {
        json!{[]}
    }
}
