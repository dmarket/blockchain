use std::convert::From;
use std::ops::{Add, Div, Mul, Sub};

pub use self::error::Error;
pub use self::fields::Field;
pub use self::segments::SegmentField;

mod error;
#[macro_use]
mod fields;
#[macro_use]
mod spec;

mod segments;

/// Type alias usable for reference in buffer
pub type Offset = u32;

/// Type alias that should be returned in `check` method of `Field`
pub type Result = ::std::result::Result<CheckedOffset, Error>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct CheckedOffset {
    offset: Offset,
}

impl CheckedOffset {
    /// create checked value
    pub fn new(offset: Offset) -> CheckedOffset {
        CheckedOffset { offset: offset }
    }

    /// return unchecked offset
    pub fn unchecked_offset(self) -> Offset {
        self.offset
    }
}

#[macro_export]
macro_rules! implement_default_ops_checked {
    ($trait_name:ident $function:ident $checked_function:ident) => {
        impl $trait_name<CheckedOffset> for CheckedOffset {
            type Output = ::std::result::Result<CheckedOffset, Error>;
            fn $function(self, rhs: CheckedOffset) -> Self::Output {
                self.offset
                    .$checked_function(rhs.offset)
                    .map(CheckedOffset::new)
                    .ok_or(Error::OffsetOverflow)
            }
        }
        impl $trait_name<Offset> for CheckedOffset {
            type Output = ::std::result::Result<CheckedOffset, Error>;
            fn $function(self, rhs: Offset) -> Self::Output {
                self.offset
                    .$checked_function(rhs)
                    .map(CheckedOffset::new)
                    .ok_or(Error::OffsetOverflow)
            }
        }
    };
}

implement_default_ops_checked!{Add add checked_add }
implement_default_ops_checked!{Sub sub checked_sub }
implement_default_ops_checked!{Mul mul checked_mul }
implement_default_ops_checked!{Div div checked_div }

impl From<Offset> for CheckedOffset {
    fn from(offset: Offset) -> CheckedOffset {
        CheckedOffset::new(offset)
    }
}