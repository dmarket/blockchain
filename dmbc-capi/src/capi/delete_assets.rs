use std::mem;

use libc::c_char;

use capi::builder::BuilderContext;
use capi::common::*;
use assets::AssetBundle;
use transactions::delete_assets::DELETE_ASSETS_ID;
use transactions::builders::transaction::DelAssetBuilder;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_delete_assets_set_public_key(
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

        match context.guard(DELETE_ASSETS_ID) {
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

        let builder: &mut DelAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
        builder.public_key(public_key);
        true
    }
}

ffi_fn! {
    fn dmbc_delete_assets_set_seed(
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

        match context.guard(DELETE_ASSETS_ID) {
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

        let builder: &mut DelAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
        builder.seed(seed);
        return true
    }
}

ffi_fn! {
    fn dmbc_delete_assets_add_asset(
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

        match context.guard(DELETE_ASSETS_ID) {
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
        let asset: &AssetBundle = unsafe { mem::transmute(asset) };

        let builder: &mut DelAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
        builder.add_asset(asset.clone());

        true
    }
}