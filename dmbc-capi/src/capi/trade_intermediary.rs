use std::ptr;
use std::mem;

use libc::{c_char, size_t};
use exonum::storage::StorageValue;
use exonum::messages::Message;

use assets::TradeAsset;
use transactions::components::Intermediary;
use transactions::trade_intermediary::{TradeOfferIntermediaryWrapper, TradeIntermediaryWrapper};
use capi::common::*;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_trade_offer_intermediary_create(
        intermediary: *mut Intermediary,
        seller_public_key: *const c_char,
        buyer_public_key: *const c_char,
        fee_strategy: u8,
        error: *mut Error,
    ) -> *mut TradeOfferIntermediaryWrapper {
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

        if intermediary.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Intermediary isn't initialized.".to_string()));
                }
                return ptr::null_mut();
            }
        }
        let intermediary = unsafe { &*intermediary };
        let wrapper = TradeOfferIntermediaryWrapper::new(intermediary.clone(), &seller_public_key, &buyer_public_key, fee_strategy);
        Box::into_raw(Box::new(wrapper))
    }
}

ffi_fn! {
    fn dmbc_trade_offer_intermediary_free(wrapper: *const TradeOfferIntermediaryWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut TradeOfferIntermediaryWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_trade_offer_intermediary_add_asset(
        wrapper: *mut TradeOfferIntermediaryWrapper,
        asset: *mut TradeAsset,
        error: *mut Error,
    ) -> bool {
        let wrapper = match TradeOfferIntermediaryWrapper::from_ptr(wrapper) {
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
    fn dmbc_trade_offer_intermediary_into_bytes(
        wrapper: *mut TradeOfferIntermediaryWrapper, 
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let wrapper = match TradeOfferIntermediaryWrapper::from_ptr(wrapper) {
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
    fn dmbc_tx_trade_intermediary_create(
        wrapper: *mut TradeOfferIntermediaryWrapper, 
        seller_signature: *const c_char,
        intermediary_signature: *const c_char,
        seed: u64,
        memo: *const c_char,
        error: *mut Error,
    ) -> *mut TradeIntermediaryWrapper {
        let wrapper = match TradeOfferIntermediaryWrapper::from_ptr(wrapper) {
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
        let seller_signature = match parse_signature(seller_signature) {
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
        let intermediary_signature = match parse_signature(intermediary_signature) {
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
        let tx = TradeIntermediaryWrapper::new(wrapper, seed, &seller_signature, &intermediary_signature, memo);
        Box::into_raw(Box::new(tx))
    }
}

ffi_fn! {
    fn dmbc_tx_trade_intermediary_free(wrapper: *const TradeIntermediaryWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut TradeIntermediaryWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_tx_trade_intermediary_into_bytes(
        wrapper: *mut TradeIntermediaryWrapper,
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let wrapper = match TradeIntermediaryWrapper::from_ptr(wrapper) {
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