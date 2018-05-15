use std::mem;

use transactions::builders::transaction::AddAssetBuilder;
use capi::builder::BuilderContext;

ffi_fn! {
    fn debug(context: *const BuilderContext) {
        let context = unsafe { &*context };
        let builder: &mut AddAssetBuilder = unsafe { mem::transmute(context.context_ptr) };
        println!("{:?}", builder);
    }
}