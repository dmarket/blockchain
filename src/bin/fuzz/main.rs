extern crate dmbc;

use std::io;
use std::io::Read;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::process;

fn fuzz<F>(f: F) where F: FnOnce() {
    panic::catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        process::abort();
    });
}

fn main() {
    fuzz(|| {
        let mut string = String::new();
        io::stdin().read_to_string(&mut string).unwrap();
        assert_eq!(string.chars().count(), "hello\n".len());
    });
}

