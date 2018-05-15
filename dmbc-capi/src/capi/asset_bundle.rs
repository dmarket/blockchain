use std::ptr; 

use libc::c_char;
use assets::AssetBundle;
use capi::common::parse_asset_id;

use error::Error;

ffi_fn! {
    fn dmbc_asset_create(
        id: *const c_char,
        amount: u64,  
        error: *mut Error,
    ) -> *mut AssetBundle {
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
                AssetBundle::new(asset_id, amount)
            )
        )
    }
}

ffi_fn! {
    fn dmbc_asset_free(asset: *const AssetBundle) {
        if !asset.is_null() {
            unsafe { Box::from_raw(asset as *mut AssetBundle); }
        }
    }
}