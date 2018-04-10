use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;

use dmbc::config;

pub fn pair(name: &str) -> io::Result<(PublicKey, SecretKey)> {
    let keys_path = config::config().api().keys_path();
    let public_path = keys_path.clone() + "/" + name + ".pub";
    let secret_path = keys_path.clone() + "/" + name;

    slurp(&public_path)
        .map(|key_string| PublicKey::from_hex(key_string).unwrap())
        .and_then(|public_key| {
            let key_string = slurp(&secret_path)?;
            let secret_key = SecretKey::from_hex(key_string).unwrap();
            Ok((public_key, secret_key))
        })
        .or_else(|e| {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(e);
            }
            let (public_key, secret_key) = crypto::gen_keypair();
            let mut public_file = File::create(public_path)?;
            let mut secret_file = File::create(secret_path)?;
            write!(public_file, "{}", public_key.to_hex())?;
            write!(secret_file, "{}", secret_key.to_hex())?;
            Ok((public_key, secret_key))
        })
}

fn slurp<P: AsRef<Path>>(filename: P) -> io::Result<String> {
    let mut out = String::new();
    File::open(filename)
        .and_then(|mut file| file.read_to_string(&mut out))
        .map(move |_| out)
}
