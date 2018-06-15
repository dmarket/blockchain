use std::ptr;
use std::mem;

use libc::{c_char, size_t};
use exonum::messages::Message;

use capi::common::*;
use assets::AssetBundle;
use transactions::delete_assets::DeleteAssetsWrapper;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_tx_delete_assets_create(
        public_key: *const c_char, 
        seed: u64,
        error: *mut Error,
    ) -> *mut DeleteAssetsWrapper {
        let public_key = match parse_public_key(public_key) {
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

        Box::into_raw(Box::new(DeleteAssetsWrapper::new(&public_key, seed)))
    }
}

ffi_fn! {
    fn dmbc_tx_delete_assets_free(wrapper: *const DeleteAssetsWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut DeleteAssetsWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_tx_delete_assets_add_asset(
        wrapper: *mut DeleteAssetsWrapper,
        asset: *const AssetBundle,
        error: *mut Error,
    ) -> bool {
        let wrapper = match DeleteAssetsWrapper::from_ptr(wrapper) {
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
    fn dmbc_tx_delete_assets_into_bytes(
        wrapper: *mut DeleteAssetsWrapper,
        length: *mut size_t,
        error: *mut Error,
    ) -> *const u8 {
        let wrapper = match DeleteAssetsWrapper::from_ptr(wrapper) {
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