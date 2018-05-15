use std::ffi::CStr;

use libc::c_char;
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;

use error::{Error, ErrorKind};

pub fn hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    strs.join("")
}

pub fn parse_str<'a>(string: *const c_char) -> Result<&'a str, Error> {
    match unsafe { CStr::from_ptr(string).to_str() } {
        Ok(string_str) => Ok(string_str),
        Err(err) => Err(Error::new(ErrorKind::Utf8(err))),
    }
}

pub fn parse_public_key(public_key: *const c_char) -> Result<PublicKey, Error> {
    match parse_str(public_key) {
        Ok(pk_str) => {
            match PublicKey::from_hex(pk_str) {
                Ok(pk) => Ok(pk),
                Err(err) => Err(
                    Error::new(ErrorKind::Hex(err))
                )
            }
        },
        Err(err) => Err(err)
    }
}