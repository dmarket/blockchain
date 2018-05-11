use std::ptr; 
use std::ffi::CStr;
use std::mem;
use std::str::FromStr;

use libc::{c_char, c_void, size_t};
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;
use exonum::messages::Message;

use assets::{Fee, Fees};
use decimal::UFract64;
use transactions::builders::transaction::{Builder, AddAssetBuilder};
use transactions::add_assets::ADD_ASSETS_ID;

use error::{Error, ErrorKind};

/// Service identifier.
pub const SERVICE_ID: u16 = 2;

#[derive(Debug, Clone)]
pub struct BuilderContext {
    context_ptr: *mut c_void,
    message_type: u16,
}

impl BuilderContext {
    fn from_ptr<'a>(context: *mut BuilderContext) -> Result<&'a BuilderContext, Error> {
        if context.is_null() {
            return Err(
                Error::new(
                    ErrorKind::Text("context isn't initialized".to_string())
                )
            );
        }
        Ok( unsafe { &*context } )
    }

    fn guard(&self, message_type: u16) -> Result<(), Error> {
        if self.message_type != message_type {
            return Err(
                Error::new(ErrorKind::Text("Different builder type".to_string()))
            );
        }
        Ok(())
    }
}

fn hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    strs.join("")
}

fn parse_str<'a>(string: *const c_char) -> Result<&'a str, Error> {
    match unsafe { CStr::from_ptr(string).to_str() } {
        Ok(string_str) => Ok(string_str),
        Err(err) => Err(Error::new(ErrorKind::Utf8(err))),
    }
}

fn parse_public_key(public_key: *const c_char) -> Result<PublicKey, Error> {
    match parse_str(public_key) {
        Ok(pk_str) => {
            match PublicKey::from_hex(pk_str) {
                Ok(pk) => Ok(pk),
                Err(err) => Err(
                    Error::new(ErrorKind::Hex(err))
                )
            }
        },
        Err(err) => Err(err)
    }
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
                unsafe { mem::transmute(Box::new(builder)) }
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
    fn dmbc_builder_tx_create(
        context: *mut BuilderContext,
        length: *mut size_t,
        error: *mut Error,
    ) -> *const u8 {
        let context = match BuilderContext::from_ptr(context) {
            Ok(context) => context,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null();
                }
            }
        };

        let mut bytes = match context.message_type {
            ADD_ASSETS_ID => {
                let builder: &mut AddAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
                match builder.build() {
                    Ok(tx) => { tx.raw().body().to_vec()},
                    Err(err) => {
                        unsafe {
                            if !error.is_null() {
                                *error = err;
                            }
                            return ptr::null();
                        }
                    }
                }
            }
            _ => {
                unsafe {
                    if !error.is_null() {
                        *error = Error::new(ErrorKind::Text("Unknown context, not implemented".to_string()));
                    }
                    return ptr::null();
                }
            }
        };

        if length.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("length argument is null".to_string()));
                }
                return ptr::null();
            }
        }
        
        bytes.shrink_to_fit();
        let to_print = hex_string(bytes.clone());
        println!("{}", to_print);

        assert!(bytes.len() == bytes.capacity());
        let ptr = bytes.as_ptr();
        let length = unsafe { &mut *length };
        let len = bytes.len() as size_t;
        *length = len;

        mem::forget(bytes);
        ptr
    }
}

ffi_fn! {
    fn dmbc_builder_tx_free(ptr: *mut u8, len: size_t) {
        let len = len as usize;
        unsafe {
            drop(Vec::from_raw_parts(ptr, len, len));
        }
    }
}

ffi_fn! {
    fn dmbc_builder_free(context: *const BuilderContext) {
        if !context.is_null() {
            unsafe { Box::from_raw(context as *mut BuilderContext); }
        }
    }
}

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

fn dmbc_fee_create(
    fixed: u64, 
    fraction: *const c_char, 
    error: *mut Error,
) -> Option<Fee> {
    if fraction.is_null() {
        unsafe {
            if !error.is_null() {
                *error = Error::new(ErrorKind::Text("fraction is nul".to_string()));
            }
            return None
        }
    }

    let fraction_result = unsafe { CStr::from_ptr(fraction).to_str() };
    let fraction_str = match fraction_result {
        Ok(fraction_str) => fraction_str,
        Err(err) => {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Utf8(err));
                }
                return None
            }
        },
    };

    let fraction = match UFract64::from_str(fraction_str) {
        Ok(fraction) => fraction,
        Err(err) => {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text(err.to_string()));
                }
                return None
            }
        }
    };

    Some(Fee::new(fixed, fraction))
}

ffi_fn! {
    fn dmbc_fees_create(
        trade_fixed: u64, 
        trade_fraction: *const c_char,
        exchnage_fixed: u64, 
        exchange_fraction: *const c_char,
        transfer_fixed: u64, 
        transfer_fraction: *const c_char,
        error: *mut Error
    ) -> *mut Fees {
        let trade = dmbc_fee_create(trade_fixed, trade_fraction, error);
        let exchange = dmbc_fee_create(exchnage_fixed, exchange_fraction, error);
        let transfer = dmbc_fee_create(transfer_fixed, transfer_fraction, error);

        if trade.is_none() || exchange.is_none() || transfer.is_none() {
            return ptr::null_mut()
        }

        Box::into_raw(
            Box::new(
                Fees::new(
                    trade.unwrap(), 
                    exchange.unwrap(), 
                    transfer.unwrap()
                )
            )
        )
    }
}

ffi_fn! {
    fn dmbc_fees_free(fees: *const c_char) {
        if !fees.is_null() {
            unsafe { Box::from_raw(fees as *mut Fees); }
        }
    }
}

ffi_fn! {
    fn debug(context: *const BuilderContext) {
        let context = unsafe { &*context };
        let builder: &mut AddAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
        println!("{:?}", builder);
    }
}