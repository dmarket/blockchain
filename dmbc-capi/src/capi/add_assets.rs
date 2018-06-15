use std::ptr;
use std::mem;

use libc::{c_char, size_t};
use exonum::messages::Message;

use capi::common::*;
use assets::{Fees, MetaAsset};
use transactions::add_assets::AddAssetWrapper;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_tx_add_assets_create(
        public_key: *const c_char,
        seed: u64,
        error: *mut Error,
    ) -> *mut AddAssetWrapper {
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

        Box::into_raw(Box::new(AddAssetWrapper::new(&public_key, seed)))
    }
}

ffi_fn! {
    fn dmbc_tx_add_asset_free(wrapper: *const AddAssetWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut AddAssetWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_tx_add_assets_add_asset(
        wrapper: *mut AddAssetWrapper,
        name: *const c_char,
        count: u64,
        fees: *const Fees,
        receiver_key: *const c_char,
        error: *mut Error,
    ) -> bool {
        let wrapper = match AddAssetWrapper::from_ptr(wrapper) {
            Ok(wrapper) => wrapper,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false;
                }
            }
        };

        if fees.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Invalid fees pointer.".to_string()));
                }
                return false;
            }
        } 
        let fees = Fees::from_ptr(fees);

        let receiver_key = match parse_public_key(receiver_key) {
            Ok(pk) => pk,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false;
                }
            }
        };

        let name = match parse_str(name) {
            Ok(name) => name,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false;
                }
            }
        };


        let meta = MetaAsset::new(&receiver_key, name, count, fees.clone());
        wrapper.add_asset(meta);

        true
    }
}

ffi_fn! {
    fn dmbc_tx_add_assets_into_bytes(
        wrapper: *mut AddAssetWrapper,
        length: *mut size_t,
        error: *mut Error,
    ) -> *const u8 {
        let wrapper = match AddAssetWrapper::from_ptr(wrapper) {
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