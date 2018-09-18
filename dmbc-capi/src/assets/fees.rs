use decimal;
use std::mem;

evo_encoding_struct! {
    /// Fee data for specific kind of operations.
    struct Fee {
        fixed:      u64,
        fraction: decimal::UFract64,
    }
}

evo_encoding_struct! {
    /// Third party fee data, part of `AssetInfo`.
    struct Fees {
        trade:    Fee,
        exchange: Fee,
        transfer: Fee,
    }
}

impl Fees {
    pub fn from_ptr<'a>(ptr: *const Fees) -> &'a Self {
        unsafe { mem::transmute(ptr) }
    }
}
