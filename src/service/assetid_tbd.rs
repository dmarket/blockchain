extern crate exonum;
extern crate serde_json;

use std::str::FromStr;
use serde_json::value::Value;
use exonum::encoding::{Field, Offset, CheckedOffset, Result};
use exonum::encoding::serialize::json::ExonumJson;
use exonum::encoding::serialize::WriteBufferWrapper;
use std::mem;
use std::result;
use std::error::Error;
use std::string::ToString;

/// A 128-bit (16 byte) buffer containing the ID.
pub type AssetIDBytes = [u8; 16];

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AssetID {
    /// The 128-bit number stored in 16 bytes
    bytes: AssetIDBytes,
}

/// Error details for string parsing failures.
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ParseError {
    InvalidLength(usize),
    InvalidString(),
}

impl AssetID {
    pub fn nil() -> AssetID {
        AssetID { bytes: [0u8; 16] }
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    pub fn from_bytes(b: &[u8]) -> Result<AssetID, ParseError> {
        let len = b.len();
        if len != 16 {
            return Err(ParseError::InvalidLength(len));
        }

        let mut assetid = AssetID { bytes: [0; 16] };
        assetid.bytes.copy_from_slice(bytes);
        Ok(assetid)
    }
}

impl ToString for AssetID {
    fn to_string(&self) -> String {
        let assetid_hex: String = "";
        for i in 0..16 {
            assetid_hex += format!("{:2x}", self.bytes()[i]);
        }
        assetid_hex
    }
}

impl <'a> Field<'a> for AssetID {
    fn field_size() -> Offset {
        mem::size_of::<AssetIDBytes>() as Offset
    }

    unsafe fn read(buffer: &'a [u8], from: Offset, to: Offset) -> AssetID {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&buffer[from as usize..to as usize]);
        AssetID { bytes: bytes }
    }

    fn write(&self, buffer: &mut Vec<u8>, from: Offset, to: Offset) {
        buffer[from as usize..to as usize].copy_from_slice(&self.bytes);
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

impl ExonumJson for AssetID {
    fn deserialize_field<B: WriteBufferWrapper>(
        value: &Value,
        buffer: &mut B,
        from: Offset,
        to: Offset,
    ) -> result::Result<(), Box<Error>> {
        let string: String = serde_json::from_value(value.clone()).unwrap();
        let uuid = Uuid::from_str(&string).unwrap();
        let asset_id = AssetID::from_uuid(&uuid);
        buffer.write(from, to, asset_id);
        Ok(())
    }

    fn serialize_field(&self) -> result::Result<Value, Box<Error>> {
        let string = self.to_string();
        Ok(Value::String(string))
    }
}