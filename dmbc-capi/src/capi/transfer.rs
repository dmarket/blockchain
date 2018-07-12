use std::mem;
use std::ptr;

use exonum::messages::Message;
use libc::{c_char, size_t};

use assets::AssetBundle;
use capi::common::*;
use transactions::transfer::TransferWrapper;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_tx_transfer_create(
        from: *const c_char,
        to: *const c_char,
        amout: u64,
        seed: u64,
        data_info: *const c_char,
        error: *mut Error,
    ) -> *mut TransferWrapper {
        let from = match parse_public_key(from) {
            Ok(pk) => pk,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null_mut();
                }
            }
        };

        let to = match parse_public_key(to) {
            Ok(pk) => pk,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null_mut();
                }
            }
        };

        let data_info = match parse_str(data_info) {
            Ok(pk) => pk,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null_mut();
                }
            }
        };

        Box::into_raw(
            Box::new(
                TransferWrapper::new(&from, &to, amout, seed, data_info)
            )
        )
    }
}

ffi_fn! {
    fn dmbc_tx_transfer_free(wrapper: *const TransferWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut TransferWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_tx_transfer_add_asset(
        wrapper: *mut TransferWrapper,
        asset: *const AssetBundle,
        error: *mut Error,
    ) -> bool {
        let wrapper = match TransferWrapper::from_ptr(wrapper) {
            Ok(wrapper) => wrapper,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false
                }
            }
        };

        if asset.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Invalid asset pointer.".to_string()));
                }
                return false;
            }
        }

        let asset = AssetBundle::from_ptr(asset);
        wrapper.add_asset(asset.clone());

        true
    }
}

ffi_fn! {
    fn dmbc_tx_transfer_into_bytes(
        wrapper: *mut TransferWrapper,
        length: *mut size_t,
        error: *mut Error,
    ) -> *const u8 {
        let wrapper = match TransferWrapper::from_ptr(wrapper) {
            Ok(wrapper) => wrapper,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null();
                }
            }
        };

        let bytes = wrapper.unwrap().raw().body().to_vec();
        assert!(bytes.len() == bytes.capacity());
        let length = unsafe { &mut *length };
        let len = bytes.len() as size_t;
        *length = len;

        let ptr = bytes.as_ptr();
        mem::forget(bytes);

        ptr
    }
}
