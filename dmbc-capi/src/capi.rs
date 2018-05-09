use std::ptr; 
use std::ffi::CStr;
use std::mem;

use libc::{c_char, c_void};
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;

use transactions::builders::transaction::{Builder, AddAssetBuilder};
use transactions::add_assets::ADD_ASSETS_ID;

use error::{Error, ErrorKind};

/// Service identifier.
pub const SERVICE_ID: u16 = 2;

pub struct BuilderContext {
    context_ptr: *mut c_void,
    message_type: u16,
}

ffi_fn! {
    fn dmbc_builder_create(
        network_id: u8,
        protocol_version: u8,
        service_id: u16,
        message_type: u16,
        error: *mut Error,
    ) -> *mut BuilderContext {

        let context_ptr: *mut c_void = match message_type {
            ADD_ASSETS_ID => {
                let builder = Builder::new(network_id, protocol_version, service_id)
                    .tx_add_asset();
                unsafe { mem::transmute(&builder) }
            },
            _ => {
                unsafe {
                    if !error.is_null() {
                        *error = Error::new(ErrorKind::Text(format!("Unknown message type '{}'", message_type)));
                    }
                    return ptr::null_mut();
                }
            }
        };

        Box::into_raw(
            Box::new(
                BuilderContext {
                context_ptr,
                message_type
            })
        )
    }
}

ffi_fn! {
    fn dmbc_builder_free(context: *const BuilderContext) {
        unsafe { Box::from_raw(context as *mut BuilderContext); }
    }
}

ffi_fn! {
    fn dmbc_add_assets_set_public_key(
        context: *const BuilderContext, 
        public_key: *const c_char, 
        error: *mut Error
    ) -> bool {
        let context = unsafe { &*context };
        if context.message_type != ADD_ASSETS_ID {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Different builder type".to_string()));
                }
                return false;
            }
        }

        let public_key_result = unsafe { CStr::from_ptr(public_key).to_str() };
        let public_key = match public_key_result {
            Ok(pk_str) => {
                match PublicKey::from_hex(pk_str) {
                    Ok(pk) => pk,
                    Err(err) => unsafe {
                        if !error.is_null() {
                            *error = Error::new(ErrorKind::Hex(err));
                        }
                        return false;
                    },
                }
            },
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = Error::new(ErrorKind::Str(err));
                    }
                    return false;
                }
            },
        };

        let builder: &mut AddAssetBuilder = unsafe { mem::transmute(context.context_ptr) };

        builder.public_key(public_key);
        true
    }
}

ffi_fn! {
    fn debug(context: *const BuilderContext) {
        let context = unsafe { &*context };
        let builder: &mut AddAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
        println!("{:?}", builder.public_key);
    }
}