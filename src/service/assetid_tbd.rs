
use exonum::encoding::{CheckedOffset, Field, Offset, Result};
use exonum::encoding::serialize::WriteBufferWrapper;
use exonum::encoding::serialize::json::ExonumJson;
use serde_json;
use serde_json::value::Value;
use std::error::Error;
use std::mem;
use std::result;
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

    pub fn from_bytes(b: &[u8]) -> result::Result<AssetID, ParseError> {
        let len = b.len();
        if len != 16 {
            return Err(ParseError::InvalidLength(len));
        }

        let mut assetid = AssetID::nil();
        assetid.bytes.copy_from_slice(b);
        Ok(assetid)
    }

    pub fn from_str(us: &str) -> result::Result<AssetID, ParseError> {
        let len = us.len();
        if len != 32 {
            return Err(ParseError::InvalidLength(len));
        }

        let mut bytes = [0u8; 16];

        for i in 0..bytes.len() {
            let offset = i * 2;
            let to = offset + 2;
            match u8::from_str_radix(&us[offset..to], 16) {
                Ok(byte) => bytes[i] = byte,
                Err(..) => return Err(ParseError::InvalidString()),
            }
        }

        AssetID::from_bytes(&bytes)
    }
}

impl ToString for AssetID {
    fn to_string(&self) -> String {
        let mut assetid_hex = "".to_string();
        let len = self.bytes.len();
        for i in 0..len {
            let byte_hex = format!("{:2x}", self.bytes[i]);
            assetid_hex += &*byte_hex;
        }
        assetid_hex
    }
}

impl<'a> Field<'a> for AssetID {
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
        let asset_id = AssetID::from_str(&string);
        // TODO: FIX ME
        if asset_id.is_ok() {
            buffer.write(from, to, asset_id.unwrap());
        }
        Ok(())
    }

    fn serialize_field(&self) -> result::Result<Value, Box<Error>> {
        let string = self.to_string();
        Ok(Value::String(string))
    }
}
