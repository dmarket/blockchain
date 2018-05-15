use libc::c_char;

use capi::builder::BuilderContext;
use capi::common::*;
use assets::AssetBundle;
use transactions::transfer::TRANSFER_ID;
use transactions::builders::transaction::TransferBuilder;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_transfer_set_from_public_key(
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

        match context.guard(TRANSFER_ID) {
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

        let builder: &mut TransferBuilder = context.unwrap_mut();
        builder.from(public_key);
        true
    }
}

ffi_fn! {
    fn dmbc_transfer_set_to_public_key(
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

        match context.guard(TRANSFER_ID) {
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

        let builder: &mut TransferBuilder = context.unwrap_mut();
        builder.to(public_key);
        true
    }
}

ffi_fn! {
    fn dmbc_transfer_set_seed(
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

        match context.guard(TRANSFER_ID) {
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

        let builder: &mut TransferBuilder = context.unwrap_mut();
        builder.seed(seed);
        return true
    }
}

ffi_fn! {
    fn dmbc_transfer_set_amount(
        context: *mut BuilderContext,
        amount: u64,
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

        match context.guard(TRANSFER_ID) {
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

        let builder: &mut TransferBuilder = context.unwrap_mut();
        builder.amount(amount);
        return true
    }
}

ffi_fn! {
    fn dmbc_transfer_set_info(
        context: *mut BuilderContext,
        info: *const c_char,
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

        match context.guard(TRANSFER_ID) {
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

        let info = match parse_str(info) {
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

        let builder: &mut TransferBuilder = context.unwrap_mut();
        builder.data_info(info);

        true
    }
}

ffi_fn! {
    fn dmbc_transfer_add_asset(
        context: *mut BuilderContext,
        asset: *const AssetBundle,
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

        match context.guard(TRANSFER_ID) {
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

        if asset.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Invalid asset pointer.".to_string()));
                }
                return false;
            }
        } 

        let asset = AssetBundle::from_ptr(asset);
        let builder: &mut TransferBuilder = context.unwrap_mut();
        builder.add_asset(asset.clone());

        true
    }
}