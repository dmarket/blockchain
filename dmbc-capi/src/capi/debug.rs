use std::mem;

use transactions::builders::transaction::*;
use transactions::add_assets::ADD_ASSETS_ID;
use transactions::delete_assets::DELETE_ASSETS_ID;
use capi::builder::BuilderContext;

ffi_fn! {
    fn debug(context: *const BuilderContext) {
        let context = unsafe { &*context };
        match context.message_type {
            ADD_ASSETS_ID => {
                let builder: &mut AddAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
                println!("{:?}", builder);
            },
            DELETE_ASSETS_ID => {
                let builder: &mut DelAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
                println!("{:?}", builder);
            },
            _ => {
                println!("Undefined");
            }
        }
    }
}