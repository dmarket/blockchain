use std::mem;
use assets::AssetId;

encoding_struct! {
    /// Asset representation to used in `trade` transactions.
    struct TradeAsset {
        id: AssetId,
        amount: u64,
        price:  u64,
    }
}

impl TradeAsset {
    pub fn from_ptr<'a>(ptr: *const TradeAsset) -> &'a Self {
        unsafe { mem::transmute(ptr) }
    }
}