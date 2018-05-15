use std::mem;

use libc::c_char;

use capi::builder::BuilderContext;
use capi::common::*;
use assets::Fees;
use transactions::add_assets::ADD_ASSETS_ID;
use transactions::builders::transaction::AddAssetBuilder;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_add_assets_set_public_key(
        context: *mut BuilderContext, 
        public_key: *const c_char, 
        error: *mut Error
    ) -> bool {

        let context = match BuilderContext::from_ptr(context) {
            Ok(context) => context,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false
                }
            }
        };

        match context.guard(ADD_ASSETS_ID) {
            Ok(_) => {},
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false
                }
            }
        }

        let public_key = match parse_public_key(public_key) {
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

        let builder: &mut AddAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
        builder.public_key(public_key);
        true
    }
}

ffi_fn! {
    fn dmbc_add_assets_set_seed(
        context: *mut BuilderContext,
        seed: u64,
        error: *mut Error,
    ) -> bool {

        let context = match BuilderContext::from_ptr(context) {
            Ok(context) => context,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false
                }
            }
        };

        match context.guard(ADD_ASSETS_ID) {
            Ok(_) => {},
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false
                }
            }
        }

        let builder: &mut AddAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
        builder.seed(seed);
        return true
    }
}

ffi_fn! {
    fn dmbc_add_assets_add_asset(
        context: *mut BuilderContext,
        name: *const c_char,
        count: u64,
        fees: *const Fees,
        receiver_key: *const c_char,
        error: *mut Error,
    ) -> bool {
        let context = match BuilderContext::from_ptr(context) {
            Ok(context) => context,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false
                }
            }
        };

        match context.guard(ADD_ASSETS_ID) {
            Ok(_) => {},
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return false
                }
            }
        }

        if fees.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Invalid fees pointer.".to_string()));
                }
                return false;
            }
        } 
        let fees: &Fees = unsafe { mem::transmute(fees) };

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

        let builder: &mut AddAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
        builder.add_asset(name, count, fees.clone(), &receiver_key);

        true
    }
}