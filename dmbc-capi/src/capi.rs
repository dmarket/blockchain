use std::ptr; 
use std::ffi::CStr;
use std::error::Error;

use libc::c_char;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::encoding::serialize::FromHex;

use transactions::builders::transaction::Builder;
use error::{Error as LibError, ErrorKind};

/// Service identifier.
pub const SERVICE_ID: u16 = 2;

ffi_fn! {
    fn dmbc_builder_create(
        public_key: *const c_char,
        secret_key: *const c_char,
        network_id: u8,
        protocol_version: u8,
        message_type: u16,
        error: *mut LibError,
    ) -> *const Builder {
        let public_key_result = unsafe { CStr::from_ptr(public_key).to_str() };
        let secret_key_result = unsafe { CStr::from_ptr(secret_key).to_str() };

        let public_key = match public_key_result {
            Ok(pk_str) => {
                match PublicKey::from_hex(pk_str) {
                    Ok(pk) => pk,
                    Err(err) => unsafe {
                        if !error.is_null() {
                            *error = LibError::new(ErrorKind::Hex(err));
                        }
                        return ptr::null();
                    },
                }
            },
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = LibError::new(ErrorKind::Str(err));
                    }
                    return ptr::null();
                }
            },
        };

        let secret_key = match secret_key_result {
            Ok(pk_str) => {
                match SecretKey::from_hex(pk_str) {
                    Ok(pk) => pk,
                    Err(err) => unsafe {
                        if !error.is_null() {
                            *error = LibError::new(ErrorKind::Hex(err));
                        }
                        return ptr::null();
                    },
                }
            },
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = LibError::new(ErrorKind::Str(err));
                    }
                    return ptr::null();
                }
            },
        };

        return ptr::null();
    }
}

ffi_fn! {
    fn dmbc_builder_free(builder: *const Builder) {
        unsafe { Box::from_raw(builder as *mut Builder); }
    }
}