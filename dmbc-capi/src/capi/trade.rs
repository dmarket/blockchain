use std::mem;
use std::ptr;

use exonum::messages::Message;
use exonum::storage::StorageValue;
use libc::{c_char, size_t};

use assets::TradeAsset;
use capi::common::*;
use transactions::trade::{TradeOfferWrapper, TradeWrapper};

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_trade_offer_create(
        seller_public_key: *const c_char,
        buyer_public_key: *const c_char,
        fee_strategy: u8,
        seed: u64,
        data_info: *const c_char,
        error: *mut Error,
    ) -> *mut TradeOfferWrapper {
        let seller_public_key = match parse_public_key(seller_public_key) {
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
        let buyer_public_key = match parse_public_key(buyer_public_key) {
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
        let data_info = match parse_str(data_info) {
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


        let wrapper = TradeOfferWrapper::new(&seller_public_key, &buyer_public_key, fee_strategy, seed, data_info);
        Box::into_raw(Box::new(wrapper))
    }
}

ffi_fn! {
    fn dmbc_trade_offer_free(wrapper: *const TradeOfferWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut TradeOfferWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_trade_offer_add_asset(
        wrapper: *mut TradeOfferWrapper,
        asset: *mut TradeAsset,
        error: *mut Error,
    ) -> bool {
        let wrapper = match TradeOfferWrapper::from_ptr(wrapper) {
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

        let asset = TradeAsset::from_ptr(asset);
        wrapper.add_asset(asset.clone());
        true
    }
}

ffi_fn! {
    fn dmbc_trade_offer_into_bytes(
        wrapper: *mut TradeOfferWrapper,
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let wrapper = match TradeOfferWrapper::from_ptr(wrapper) {
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
    fn dmbc_tx_trade_create(
        wrapper: *mut TradeOfferWrapper,
        signature: *const c_char,
        error: *mut Error,
    ) -> *mut TradeWrapper {
        let wrapper = match TradeOfferWrapper::from_ptr(wrapper) {
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

        let wrapper = wrapper.unwrap().clone();
        let tx = TradeWrapper::new(wrapper, &signature);
        Box::into_raw(Box::new(tx))
    }
}

ffi_fn! {
    fn dmbc_tx_trade_free(wrapper: *const TradeWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut TradeWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_tx_trade_into_bytes(
        wrapper: *mut TradeWrapper,
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let wrapper = match TradeWrapper::from_ptr(wrapper) {
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
