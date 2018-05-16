use std::ptr;
use std::mem;

use libc::{c_char, size_t};
use exonum::storage::StorageValue;

use assets::AssetBundle;
use transactions::exchange::ExchangeOfferWrapper;
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
        let recipient_key = match parse_public_key(sender_public_key) {
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


        let offer = ExchangeOfferWrapper::new(&sender_key, sender_value, &recipient_key, fee_strategy);
        Box::into_raw(Box::new(offer))
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_free(offer: *const ExchangeOfferWrapper) {
        if !offer.is_null() {
            unsafe { Box::from_raw(offer as *mut ExchangeOfferWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_recipient_add_asset(
        offer: *mut ExchangeOfferWrapper,
        asset: *mut AssetBundle,
        error: *mut Error,
    ) -> bool {
        let offer = match ExchangeOfferWrapper::from_ptr(offer) {
            Ok(offer) => offer,
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
        offer.add_recipient_asset(asset.clone());
        true
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_sender_add_asset(
        offer: *mut ExchangeOfferWrapper,
        asset: *mut AssetBundle,
        error: *mut Error,
    ) -> bool {
        let offer = match ExchangeOfferWrapper::from_ptr(offer) {
            Ok(offer) => offer,
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
        offer.add_sender_asset(asset.clone());
        true
    }
}

ffi_fn! {
    fn dmbc_exchange_offer_into_bytes(
        offer: *mut ExchangeOfferWrapper, 
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let offer = match ExchangeOfferWrapper::from_ptr(offer) {
            Ok(offer) => offer,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null();
                }
            }
        };

        let bytes = offer.unwrap().clone().into_bytes();
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
    fn dmbc_exchange_offer_bytes_free(ptr: *mut u8, len: size_t) {
        let len = len as usize;
        unsafe {
            drop(Vec::from_raw_parts(ptr, len, len));
        }
    }
}