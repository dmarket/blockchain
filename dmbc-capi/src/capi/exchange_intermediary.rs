use std::mem;
use std::ptr;

use exonum::messages::Message;
use exonum::storage::StorageValue;
use libc::{c_char, size_t};

use assets::AssetBundle;
use capi::common::*;
use transactions::components::Intermediary;
use transactions::exchange_intermediary::{
    ExchangeIntermediaryWrapper, ExchangeOfferIntermediaryWrapper,
};

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_exchange_offer_intermediary_create(
        intermediary: *mut Intermediary,
        sender_public_key: *const c_char,
        sender_value: u64,
        recipient_public_key: *const c_char,
        fee_strategy: u8,
        seed: u64,
        memo: *const c_char,
        error: *mut Error,
    ) -> *mut ExchangeOfferIntermediaryWrapper {
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
        if intermediary.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Intermediary isn't initialized.".to_string()));
                }
                return ptr::null_mut();
            }
        }
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
        let intermediary = unsafe { &*intermediary };
        let wrapper = ExchangeOfferIntermediaryWrapper::new(intermediary.clone(), &sender_key, sender_value, &recipient_key, fee_strategy, seed, memo);
        Box::into_raw(Box::new(wrapper))
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_intermediary_free(wrapper: *const ExchangeOfferIntermediaryWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut ExchangeOfferIntermediaryWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_intermediary_recipient_add_asset(
        wrapper: *mut ExchangeOfferIntermediaryWrapper,
        asset: *mut AssetBundle,
        error: *mut Error,
    ) -> bool {
        let wrapper = match ExchangeOfferIntermediaryWrapper::from_ptr(wrapper) {
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
    fn dmbc_exchange_offer_intermediary_sender_add_asset(
        wrapper: *mut ExchangeOfferIntermediaryWrapper,
        asset: *mut AssetBundle,
        error: *mut Error,
    ) -> bool {
        let wrapper = match ExchangeOfferIntermediaryWrapper::from_ptr(wrapper) {
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
    fn dmbc_exchange_offer_intermediary_into_bytes(
        wrapper: *mut ExchangeOfferIntermediaryWrapper,
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let wrapper = match ExchangeOfferIntermediaryWrapper::from_ptr(wrapper) {
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
    fn dmbc_tx_exchange_intermediary_create(
        wrapper: *mut ExchangeOfferIntermediaryWrapper,
        sender_signature: *const c_char,
        intermediary_signature: *const c_char,
        error: *mut Error,
    ) -> *mut ExchangeIntermediaryWrapper {
        let wrapper = match ExchangeOfferIntermediaryWrapper::from_ptr(wrapper) {
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
        let sender_signature = match parse_signature(sender_signature) {
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

        let wrapper = wrapper.unwrap().clone();
        let tx = ExchangeIntermediaryWrapper::new(wrapper, &sender_signature, &intermediary_signature);
        Box::into_raw(Box::new(tx))
    }
}

ffi_fn! {
    fn dmbc_tx_exchange_intermediary_free(wrapper: *const ExchangeIntermediaryWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut ExchangeIntermediaryWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_tx_exchange_intermediary_into_bytes(
        wrapper: *mut ExchangeIntermediaryWrapper,
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let wrapper = match ExchangeIntermediaryWrapper::from_ptr(wrapper) {
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
