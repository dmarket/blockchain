
use config;
use exonum::crypto::{PublicKey, SecretKey};
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPair {
    pub public: PublicKey,
    pub secret: SecretKey,
}

pub trait Disc {
    fn save(&self, filename: &str);
    fn read(filename: &str) -> Self;
}

impl Disc for KeyPair {
    fn save(&self, filename: &str) {
        let path = config::config().api().keys_path();
        let path = Path::new(&path).join(filename);
        let display = path.display();

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why.description()),
            Ok(file) => file,
        };
        let s = json!(self);

        match file.write_all(s.to_string().as_bytes()) {
            Err(why) => panic!("couldn't write to {}: {}", display, why.description()),
            Ok(_) => println!("successfully wrote to {}", display),
        };
    }


    fn read(filename: &str) -> KeyPair {

        let path = config::config().api().keys_path();
        let path = Path::new(&path).join(filename);
        let display = path.display();

        // Open a file in write-only mode, returns `io::Result<File>`
        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };

        let mut reader = BufReader::new(&file);

        // read_line takes reads a line and writes to a string, so we give it one.
        let buffer_string = &mut String::new();
        let _res = reader.read_line(buffer_string);
        let s = buffer_string.to_string();
        let keys: KeyPair = serde_json::from_str(&s).unwrap();

        keys
    }
}

#[cfg(test)]
use exonum::crypto;
#[test]
fn test_create_keys() {
    let (p, s) = crypto::gen_keypair();
    let key = KeyPair {
        public: p,
        secret: s,
    };
    key.save("test.json");
    let new_key = KeyPair::read("test.json");

    assert_eq!(key.public, new_key.public);
    assert_eq!(key.secret, new_key.secret);
}
