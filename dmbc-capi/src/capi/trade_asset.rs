use std::ptr; 

use libc::c_char;
use assets::TradeAsset;
use capi::common::parse_asset_id;

use error::Error;

ffi_fn! {
    fn dmbc_trade_asset_create(
        id: *const c_char,
        amount: u64, 
        price: u64, 
        error: *mut Error,
    ) -> *mut TradeAsset {
        let asset_id = match parse_asset_id(id) {
            Ok(id) => id,
            Err(err) => {
                unsafe {
                    if !error.is_null() {
                        *error = err;
                    }
                    return ptr::null_mut();
                }
            }
        };

    Box::into_raw(
            Box::new(
                TradeAsset::new(asset_id, amount, price)
            )
        )
    }
}

ffi_fn! {
    fn dmbc_trade_asset_free(asset: *const TradeAsset) {
        if !asset.is_null() {
            unsafe { Box::from_raw(asset as *mut TradeAsset); }
        }
    }
}