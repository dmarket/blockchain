use std::mem;
use std::ptr;

use messages::Message;
use storage::StorageValue;
use libc::{c_char, size_t};

use assets::AssetBundle;
use capi::common::*;
use transactions::transfer_fees_payers::{TransferOfferWrapper, TransferWithFeesPayerWrapper};

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_transfer_fees_payer_offer_create(
        from_public_key: *mut c_char,
        to_public_key: *mut c_char,
        fees_payer_public_key: *mut c_char,
        amount: u64,
        seed: u64,
        memo: *const c_char,
        error: *mut Error,
    ) -> *mut TransferOfferWrapper {
        let from = match parse_public_key(from_public_key) {
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

        let to = match parse_public_key(to_public_key) {
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

        let fees_payer = match parse_public_key(fees_payer_public_key) {
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

        let wrapper = TransferOfferWrapper::new(&from, &to, &fees_payer, amount, seed, memo);
        Box::into_raw(Box::new(wrapper))
    }
}

ffi_fn! {
    fn dmbc_transfer_fees_payer_offer_free(wrapper: *const TransferOfferWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut TransferOfferWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_transfer_fees_payer_offer_add_asset(
        wrapper: *mut TransferOfferWrapper,
        asset: *mut AssetBundle,
        error: *mut Error,
    ) -> bool {
        let wrapper = match TransferOfferWrapper::from_ptr(wrapper) {
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
        wrapper.add_asset(asset.clone());
        true
    }
}

ffi_fn! {
    fn dmbc_transfer_fees_payer_offer_into_bytes(
       wrapper: *mut TransferOfferWrapper,
        length: *mut size_t,
        error: *mut Error 
    ) -> *const u8 {
        let wrapper = match TransferOfferWrapper::from_ptr(wrapper) {
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
    fn dmbc_tx_transfer_fees_payer_create(
        wrapper: *mut TransferOfferWrapper,
        fees_payer_signature: *const c_char,
        error: *mut Error,
    ) -> *mut TransferWithFeesPayerWrapper {
        let wrapper = match TransferOfferWrapper::from_ptr(wrapper) {
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

        let fees_payer_signature = match parse_signature(fees_payer_signature) {
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
        let tx = TransferWithFeesPayerWrapper::new(wrapper, &fees_payer_signature);
        Box::into_raw(Box::new(tx))
    }
}

ffi_fn! {
    fn dmbc_tx_transfer_fees_payer_free(wrapper: *const TransferWithFeesPayerWrapper) {
        if !wrapper.is_null() {
            unsafe { Box::from_raw(wrapper as *mut TransferWithFeesPayerWrapper); }
        }
    }
}

ffi_fn! {
    fn dmbc_tx_transfer_fees_payer_into_bytes(
        wrapper: *mut TransferWithFeesPayerWrapper,
        length: *mut size_t,
        error: *mut Error
    ) -> *const u8 {
        let wrapper = match TransferWithFeesPayerWrapper::from_ptr(wrapper) {
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