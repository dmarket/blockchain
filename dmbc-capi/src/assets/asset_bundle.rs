use assets::AssetId;
use std::mem;

encoding_struct! {
    /// A bundle of assets with the same id.
    struct AssetBundle {
        id:     AssetId,
        amount: u64,
    }
}

impl AssetBundle {
    pub fn from_ptr<'a>(ptr: *const AssetBundle) -> &'a Self {
        unsafe { mem::transmute(ptr) }
    }
}
