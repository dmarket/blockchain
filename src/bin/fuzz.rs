extern crate exonum;
extern crate dmbc;

use std::io;
use std::io::Read;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::process;

use exonum::messages::MessageWriter;
use exonum::messages::MessageBuffer;

fn main() {
    fuzz(|| {
        let mut data = Vec::new();
        io::stdin().read_to_end(&mut data).unwrap();
        MessageBuffer::from_vec(data);
        // make raw transaction from buffer
        // verify or exit early
        // launch testkit and pipe the transaction through API.
    });
}

fn fuzz<F>(f: F) where F: FnOnce() {
    panic::catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        process::abort();
    });
}

