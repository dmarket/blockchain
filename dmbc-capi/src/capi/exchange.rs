use std::ptr;
use std::mem;

use libc::{c_char, size_t};
use exonum::storage::StorageValue;

use assets::AssetBundle;
use transactions::exchange::{ExchangeOfferWrapper, ExchangeWrapper};
use capi::common::*;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_exchange_offer_create(
        sender_public_key: *const c_char,
        sender_value: u64,
        recipient_public_key: *const c_char,
        fee_strategy: u8,
        error: *mut Error,
    ) -> *mut ExchangeOfferWrapper {
        let sender_key = match parse_public_key(sender_public_key) {
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
        let recipient_key = match parse_public_key(recipient_public_key) {
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


        let wrapper = ExchangeOfferWrapper::new(&sender_key, sender_value, &recipient_key, fee_strategy);
        Box::into_raw(Box::new(wrapper))
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_free(wrapper: *const ExchangeOfferWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut ExchangeOfferWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_recipient_add_asset(
        wrapper: *mut ExchangeOfferWrapper,
        asset: *mut AssetBundle,
        error: *mut Error,
    ) -> bool {
        let wrapper = match ExchangeOfferWrapper::from_ptr(wrapper) {
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

        if asset.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Invalid asset pointer.".to_string()));
                }
                return false;
            }
        } 

        let asset = AssetBundle::from_ptr(asset);
        wrapper.add_recipient_asset(asset.clone());
        true
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_sender_add_asset(
        wrapper: *mut ExchangeOfferWrapper,
        asset: *mut AssetBundle,
        error: *mut Error,
    ) -> bool {
        let wrapper = match ExchangeOfferWrapper::from_ptr(wrapper) {
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

        if asset.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Invalid asset pointer.".to_string()));
                }
                return false;
            }
        } 

        let asset = AssetBundle::from_ptr(asset);
        wrapper.add_sender_asset(asset.clone());
        true
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_into_bytes(
        wrapper: *mut ExchangeOfferWrapper, 
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let wrapper = match ExchangeOfferWrapper::from_ptr(wrapper) {
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

        let bytes = wrapper.unwrap().clone().into_bytes();
        assert!(bytes.len() == bytes.capacity());
        let length = unsafe { &mut *length };
        let len = bytes.len() as size_t;
        *length = len;

        let ptr = bytes.as_ptr();
        mem::forget(bytes);

        ptr
    }
}

ffi_fn! {
    fn dmbc_tx_exchange_create(
        wrapper: *mut ExchangeOfferWrapper, 
        signature: *const c_char,
        seed: u64,
        memo: *const c_char,
        error: *mut Error,
    ) -> *mut ExchangeWrapper {
        let wrapper = match ExchangeOfferWrapper::from_ptr(wrapper) {
            Ok(wrapper) => wrapper,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null_mut();
                }
            }
        };
        let signature = match parse_signature(signature) {
            Ok(sig) => sig,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null_mut();
                }
            }
        };
        let memo = match parse_str(memo) {
            Ok(memo) => memo,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null_mut();
                }
            }
        };

        let wrapper = wrapper.unwrap().clone();
        let tx = ExchangeWrapper::new(wrapper, seed, &signature, memo);
        Box::into_raw(Box::new(tx))
    }
}

ffi_fn! {
    fn dmbc_tx_exchange_free(wrapper: *const ExchangeWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut ExchangeWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_tx_exchange_into_bytes(
        wrapper: *mut ExchangeWrapper,
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let wrapper = match ExchangeWrapper::from_ptr(wrapper) {
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

        let bytes = wrapper.unwrap().clone().into_bytes();
        assert!(bytes.len() == bytes.capacity());
        let length = unsafe { &mut *length };
        let len = bytes.len() as size_t;
        *length = len;

        let ptr = bytes.as_ptr();
        mem::forget(bytes);

        ptr
    }
}