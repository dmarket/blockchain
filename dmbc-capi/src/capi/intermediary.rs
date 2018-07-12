use std::ptr;

use capi::common::parse_public_key;
use libc::c_char;
use transactions::components::Intermediary;

use error::Error;

ffi_fn! {
    fn dmbc_intermediary_create(public_key: *const c_char, commision: u64, error: *mut Error) -> *mut Intermediary {
        let public_key = match parse_public_key(public_key) {
            Ok(public_key) => public_key,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null_mut();
                }
            }
        };

        Box::into_raw(Box::new(Intermediary::new(&public_key, commision)))
    }
}

ffi_fn! {
    fn dmbc_intermediary_free(intermediary: *const Intermediary) {
         if !intermediary.is_null() {
            unsafe { Box::from_raw(intermediary as *mut Intermediary); }
        }
    }
}
