use transactions::builders::transaction::*;
use transactions::add_assets::ADD_ASSETS_ID;
use transactions::delete_assets::DELETE_ASSETS_ID;
use transactions::transfer::TRANSFER_ID;
use capi::builder::BuilderContext;

ffi_fn! {
    fn debug(context: *const BuilderContext) {
        let context = unsafe { &*context };
        match context.message_type {
            DELETE_ASSETS_ID => {
                let builder: &mut DelAssetBuilder = context.unwrap_mut();
                println!("{:?}", builder);
            },
            TRANSFER_ID => {
                let builder: &mut TransferBuilder = context.unwrap_mut();
                println!("{:?}", builder);
            },
            _ => {
                println!("Undefined");
            }
        }
    }
}