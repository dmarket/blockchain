use std::mem;
use std::ptr;

use exonum::messages::Message;
use exonum::storage::StorageValue;
use libc::{c_char, size_t};

use assets::TradeAsset;
use capi::common::*;
use transactions::bid_ask::AskOffer;

use error::{Error, ErrorKind};

ffi_fn! {
    fn dmbc_tx_bid_ask_create(
        public_key: *const c_char,
        asset: *mut TradeAsset,
        seed: u64,
        data_info: *const c_char,
        error: *mut Error,
    ) {
        let public_key = match parse_public_key(public_key) {
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

        if asset.is_null() {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Text("Invalid asset pointer.".to_string()));
                }
                return false;
            }
        }

        let asset = TradeAsset::from_ptr(asset);
    }
}