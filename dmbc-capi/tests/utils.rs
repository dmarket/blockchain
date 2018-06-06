extern crate serde_json;

use std::process::Command;
use std::env;
use std::fs;
use std::io::BufReader;
use std::io::prelude::*;

use self::serde_json::{Value, Error};

pub fn run(tx_name: &str) -> String {
    let current_dir = env::current_dir().unwrap();
    let current_dir = current_dir.as_path();
    let input_file = current_dir.join("ctest").join("inputs").join(tx_name.to_owned() + ".json");
    let mut outpit_file = env::temp_dir();
    outpit_file.push(tx_name.to_owned() + ".txt");

    let output = Command::new("./compile_tests.sh")
        .current_dir(current_dir.join("ctest"))
        .output()
        .expect("failed to compile capi test executable.");
    assert!(output.status.success(), format!("compilation failed {:?}", output));

    let output = Command::new("./test")
        .current_dir(current_dir.join("ctest"))
        .arg(tx_name)
        .arg(input_file)
        .arg(outpit_file.clone())
        .output()
        .expect("failed to run test executable");
    assert!(output.status.success(), format!("running test failed {:?}", output));

    let file = fs::File::open(outpit_file.clone());
    assert!(file.is_ok());

    let mut buf_reader = BufReader::new(file.unwrap());
    let mut contents = String::new();
    let res = buf_reader.read_to_string(&mut contents);
    assert!(res.is_ok());

    assert!(fs::remove_file(outpit_file).is_ok());

    contents
}

pub fn read_inputs(tx_name: &str) -> Result<Value, Error> {
    let current_dir = env::current_dir().unwrap();
    let current_dir = current_dir.as_path();

    let file_path = current_dir.join("ctest").join("inputs").join(tx_name.to_owned() + ".json");
    let file = fs::File::open(file_path);
    assert!(file.is_ok());

    let mut buf_reader = BufReader::new(file.unwrap());
    let mut contents = String::new();
    let res = buf_reader.read_to_string(&mut contents);
    assert!(res.is_ok());

    let v: Value = serde_json::from_str(&contents)?;
    Ok(v)
}

pub fn hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    strs.join("")
}