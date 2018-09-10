use std::mem;

use byteorder::{ByteOrder, LittleEndian};

use super::{CheckedOffset, Error, Offset, Result};
use crypto::{PublicKey, Signature};

/// Trait for all types that could be a field in `encoding`.
pub trait Field<'a> {

    /// Read Field from buffer, with given position,
    /// beware of memory unsafety,
    /// you should `check` `Field` before `read`.
    unsafe fn read(buffer: &'a [u8], from: Offset, to: Offset) -> Self;

    /// Write Field to buffer, in given position
    /// `write` doesn't lead to memory unsafety.
    fn write(&self, buffer: &mut Vec<u8>, from: Offset, to: Offset);
    /// Field's header size
    fn field_size() -> Offset;

    /// Checks if data in the buffer could be deserialized.
    /// Returns an index of latest data seen.
    #[allow(unused_variables)]
    fn check(
        buffer: &'a [u8],
        from: CheckedOffset,
        to: CheckedOffset,
        latest_segment: CheckedOffset,
    ) -> ::std::result::Result<CheckedOffset, Error>;
}

/// implement field for all types that has writer and reader functions
///
/// - reader signature is `fn (&[u8]) -> T`
/// - writer signature is `fn (&mut [u8], T)`
#[macro_export]
macro_rules! implement_std_field {
    ($name:ident $fn_read:expr; $fn_write:expr) => {
        impl<'a> Field<'a> for $name {
            fn field_size() -> $crate::encoding::Offset {
                mem::size_of::<$name>() as $crate::encoding::Offset
            }

            unsafe fn read(
                buffer: &'a [u8],
                from: $crate::encoding::Offset,
                to: $crate::encoding::Offset,
            ) -> $name {
                $fn_read(&buffer[from as usize..to as usize])
            }

            fn write(
                &self,
                buffer: &mut Vec<u8>,
                from: $crate::encoding::Offset,
                to: $crate::encoding::Offset,
            ) {
                $fn_write(&mut buffer[from as usize..to as usize], *self)
            }

            fn check(
                _: &'a [u8],
                from: $crate::encoding::CheckedOffset,
                to: $crate::encoding::CheckedOffset,
                latest_segment: CheckedOffset,
            ) -> $crate::encoding::Result {
                debug_assert_eq!((to - from)?.unchecked_offset(), Self::field_size());
                Ok(latest_segment)
            }
        }
    };
}

/// Implement field helper for all POD types
/// it writes POD type as byte array in place.
///
/// **Beware of platform specific data representation.**
#[macro_export]
macro_rules! implement_pod_as_ref_field {
    ($name:ident) => {
        impl<'a> Field<'a> for &'a $name {
            fn field_size() -> $crate::encoding::Offset {
                ::std::mem::size_of::<$name>() as $crate::encoding::Offset
            }

            unsafe fn read(
                buffer: &'a [u8],
                from: $crate::encoding::Offset,
                _: $crate::encoding::Offset,
            ) -> &'a $name {
                ::std::mem::transmute(&buffer[from as usize])
            }

            fn write(
                &self,
                buffer: &mut Vec<u8>,
                from: $crate::encoding::Offset,
                to: $crate::encoding::Offset,
            ) {
                let ptr: *const $name = *self as *const $name;
                let slice = unsafe {
                    ::std::slice::from_raw_parts(ptr as *const u8, ::std::mem::size_of::<$name>())
                };
                buffer[from as usize..to as usize].copy_from_slice(slice);
            }

            fn check(
                _: &'a [u8],
                from: $crate::encoding::CheckedOffset,
                to: $crate::encoding::CheckedOffset,
                latest_segment: $crate::encoding::CheckedOffset,
            ) -> $crate::encoding::Result {
                debug_assert_eq!((to - from)?.unchecked_offset(), Self::field_size());
                Ok(latest_segment)
            }
        }
    };
}

impl<'a> Field<'a> for bool {
    fn field_size() -> Offset {
        1
    }

    unsafe fn read(buffer: &'a [u8], from: Offset, _: Offset) -> Self {
        buffer[from as usize] == 1
    }

    fn write(&self, buffer: &mut Vec<u8>, from: Offset, _: Offset) {
        buffer[from as usize] = if *self { 1 } else { 0 }
    }

    fn check(
        buffer: &'a [u8],
        from: CheckedOffset,
        to: CheckedOffset,
        latest_segment: CheckedOffset,
    ) -> Result {
        debug_assert_eq!((to - from)?.unchecked_offset(), Self::field_size());

        let from: Offset = from.unchecked_offset();
        if buffer[from as usize] != 0 && buffer[from as usize] != 1 {
            Err(Error::IncorrectBoolean {
                position: from,
                value: buffer[from as usize],
            })
        } else {
            Ok(latest_segment)
        }
    }
}

impl<'a> Field<'a> for u8 {
    fn field_size() -> Offset {
        mem::size_of::<Self>() as Offset
    }

    unsafe fn read(buffer: &'a [u8], from: Offset, _: Offset) -> Self {
        buffer[from as usize]
    }

    fn write(&self, buffer: &mut Vec<u8>, from: Offset, _: Offset) {
        buffer[from as usize] = *self;
    }

    fn check(
        _: &'a [u8],
        from: CheckedOffset,
        to: CheckedOffset,
        latest_segment: CheckedOffset,
    ) -> Result {
        debug_assert_eq!((to - from)?.unchecked_offset(), Self::field_size());
        Ok(latest_segment)
    }
}

impl<'a> Field<'a> for i8 {
    fn field_size() -> Offset {
        mem::size_of::<Self>() as Offset
    }

    unsafe fn read(buffer: &'a [u8], from: Offset, _: Offset) -> Self {
        buffer[from as usize] as i8
    }

    fn write(&self, buffer: &mut Vec<u8>, from: Offset, _: Offset) {
        buffer[from as usize] = *self as u8;
    }

    fn check(
        _: &'a [u8],
        from: CheckedOffset,
        to: CheckedOffset,
        latest_segment: CheckedOffset,
    ) -> Result {
        debug_assert_eq!((to - from)?.unchecked_offset(), Self::field_size());
        Ok(latest_segment)
    }
}

implement_std_field!{u16 LittleEndian::read_u16; LittleEndian::write_u16}
implement_std_field!{i16 LittleEndian::read_i16; LittleEndian::write_i16}
implement_std_field!{u32 LittleEndian::read_u32; LittleEndian::write_u32}
implement_std_field!{i32 LittleEndian::read_i32; LittleEndian::write_i32}
implement_std_field!{u64 LittleEndian::read_u64; LittleEndian::write_u64}
implement_std_field!{i64 LittleEndian::read_i64; LittleEndian::write_i64}

implement_pod_as_ref_field! {Signature}
implement_pod_as_ref_field! {PublicKey}