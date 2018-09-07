use std::mem;
use std::ptr;

use messages::Message;
use libc::{c_char, size_t};

use assets::TradeAsset;
use capi::common::*;
use transactions::ask_offer::AskOfferWrapper;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_tx_ask_offer_create(
        public_key: *const c_char,
        asset: *mut TradeAsset,
        seed: u64,
        memo: *const c_char,
        error: *mut Error,
    ) -> *mut AskOfferWrapper {
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

        if asset.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Invalid asset pointer.".to_string()));
                }
                return ptr::null_mut();
            }
        }

        let asset = TradeAsset::from_ptr(asset);

        let memo = match parse_str(memo) {
            Ok(info) => info,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null_mut();
                }
            }
        };

        let wrapper = AskOfferWrapper::new(&public_key, asset.clone(), seed, memo);
        Box::into_raw(Box::new(wrapper))
    }
}

ffi_fn! {
    fn dmbc_tx_ask_offer_free(wrapper: *const AskOfferWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut AskOfferWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_tx_ask_offer_into_bytes(
        wrapper: *mut AskOfferWrapper,
        length: *mut size_t,
        error: *mut Error 
    ) -> *const u8 {
        let wrapper = match AskOfferWrapper::from_ptr(wrapper) {
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