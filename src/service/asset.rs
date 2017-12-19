use exonum::crypto::PublicKey;
use exonum::encoding::{CheckedOffset, Field, Offset, Result as ExonumResult};
use exonum::encoding::serialize::WriteBufferWrapper;
use exonum::encoding::serialize::json::ExonumJson;
use exonum::storage::StorageKey;
use serde_json::value::Value;
use std::{fmt, mem};
use std::error::Error;
use std::string::ToString;
use uuid;
use uuid::Uuid;

pub const ASSET_HASH_ID_MAX_LENGTH: usize = 10 * 1024; // 10 KBytes

encoding_struct! {
    struct MetaAsset {
        const SIZE = 12;

        field data: &str   [0 => 8]
        field amount: u32  [8 => 12]
    }
}

impl MetaAsset {
    pub fn count(assets: &[MetaAsset]) -> u64 {
        assets.iter().fold(
            0,
            |acc, asset| acc + asset.amount() as u64,
        )
    }

    pub fn is_valid(&self) -> bool {
        self.data().len() <= ASSET_HASH_ID_MAX_LENGTH
    }
}

encoding_struct! {
    struct AssetInfo {
        const SIZE = 36;

        field creator: &PublicKey [0  => 32]
        field amount:  u32        [32 => 36]
    }
}

/// A 128-bit (16 byte) buffer containing the ID.
type AssetIDBytes = [u8; 16];

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
    InvalidCharacter(char, usize),
    UnexpectedErrorAt(usize),
}

const SIMPLE_LENGTH: usize = 32;

/// Converts a ParseError to a string.
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidLength(found) => {
                write!(
                    f,
                    "Invalid length; expecting {}, found {}",
                    SIMPLE_LENGTH,
                    found
                )
            }
            ParseError::InvalidCharacter(found, pos) => {
                write!(
                    f,
                    "Invalid character; found `{}` (0x{:02x}) at offset {}",
                    found,
                    found as usize,
                    pos
                )
            }
            ParseError::UnexpectedErrorAt(pos) => write!(f, "Unexpected, at {}", pos),
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "AssetID parse error"
    }
}

impl AssetID {
    pub fn zero() -> AssetID {
        AssetID { bytes: [0u8; 16] }
    }

    /// Creates unique `AssetID` from
    /// `&str` and `&PublicKey`
    /// # Example:
    /// ```
    /// # extern crate exonum;
    /// # extern crate dmbc;
    /// #
    /// # fn main () {
    /// #
    /// # use exonum::crypto::{PublicKey, HexValue};
    /// # use dmbc::service::asset::AssetID;
    ///
    /// let data = "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f";
    /// let public_key = PublicKey::from_hex("3115dbc2ff73f4819672d5e9e421692305a9de1a18e4389df041c0cf6c8918a8").unwrap();
    ///
    /// let assetid = AssetID::new(&data, &public_key).unwrap();
    /// assert_eq!(assetid.to_string(), "82c1f90bed24508e9ce74b536f97fa9c");
    /// # }
    /// ```
    pub fn new(data: &str, pub_key: &PublicKey) -> Result<AssetID, ParseError> {
        let s = HexValue::to_hex(pub_key);
        let ful_s = s + &data;

        let uuid = Uuid::new_v5(&uuid::NAMESPACE_DNS, &ful_s);
        AssetID::from_bytes(uuid.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    pub fn from_bytes(b: &[u8]) -> Result<AssetID, ParseError> {
        let len = b.len();
        if len != mem::size_of::<AssetIDBytes>() {
            return Err(ParseError::InvalidLength(len));
        }

        let mut assetid = AssetID::zero();
        assetid.bytes.copy_from_slice(b);
        Ok(assetid)
    }

    pub fn from_str(us: &str) -> Result<AssetID, ParseError> {
        let len = us.len();
        if len != mem::size_of::<AssetIDBytes>() * 2 {
            return Err(ParseError::InvalidLength(len));
        }

        let mut cs = us.chars().enumerate();
        for (i, c) in cs.by_ref() {
            if !c.is_digit(16) {
                return Err(ParseError::InvalidCharacter(c, i));
            }
        }

        let mut bytes = [0u8; 16];

        for i in 0..bytes.len() {
            let offset = i * 2;
            let to = offset + 2;
            match u8::from_str_radix(&us[offset..to], 16) {
                Ok(byte) => bytes[i] = byte,
                Err(..) => return Err(ParseError::UnexpectedErrorAt(offset)),
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
            let byte_hex = format!("{:02x}", self.bytes[i]);
            assetid_hex += &*byte_hex;
        }
        assetid_hex
    }
}


impl StorageKey for AssetID {
    fn size(&self) -> usize {
        mem::size_of::<AssetIDBytes>()
    }

    fn read(buffer: &[u8]) -> Self {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(buffer);
        AssetID { bytes: bytes }
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.bytes)
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
    ) -> ExonumResult {
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
    ) -> Result<(), Box<Error>> {
        let val = value.as_str().ok_or("Can't cast json as string")?;
        match AssetID::from_str(&val) {
            Ok(assetid) => {
                buffer.write(from, to, assetid);
                Ok(())
            }
            Err(error) => Err(Box::new(error)),
        }
    }

    fn serialize_field(&self) -> Result<Value, Box<Error>> {
        Ok(Value::String(self.to_string()))
    }
}

encoding_struct! {
    struct Asset {
        const SIZE = 20;

        field hash_id: AssetID [0 =>  16]
        field amount:  u32 [16 => 20]
    }
}

impl Asset {
    pub fn is_eq(&self, other: &Asset) -> bool {
        self.hash_id() == other.hash_id()
    }

    pub fn is_available_to_transfer(&self, other: &Asset) -> bool {
        self.amount() >= other.amount()
    }

    pub fn count(assets: &[Asset]) -> u64 {
        assets.iter().fold(
            0,
            |acc, asset| acc + asset.amount() as u64,
        )
    }

    pub fn from_meta_asset(meta_asset: &MetaAsset, pub_key: &PublicKey) -> Asset {
        let assetid = AssetID::new(&meta_asset.data(), pub_key).unwrap();
        Asset::new(assetid, meta_asset.amount())
    }
}


#[cfg(test)]
mod tests {
    use super::AssetID;
    use super::ParseError::*;
    use exonum::encoding::{Field, Offset};

    #[test]
    fn assetid_zero() {
        let assetid = AssetID::zero();
        let expected = "00000000000000000000000000000000";

        assert_eq!(assetid.to_string(), expected);
        assert_eq!(assetid.as_bytes(), &[0u8; 16]);
    }

    #[test]
    fn assetid_from_bytes() {
        let b = [
            0xa1,
            0xa2,
            0xa3,
            0xa4,
            0xb1,
            0xb2,
            0xc1,
            0xc2,
            0xd1,
            0xd2,
            0xd3,
            0xd4,
            0xd5,
            0xd6,
            0xd7,
            0xd8,
        ];

        let assetid = AssetID::from_bytes(&b).unwrap();

        assert_eq!(assetid.as_bytes(), &b);
    }

    #[test]
    fn assetid_from_str() {
        // Invalid
        assert_eq!(AssetID::from_str(""), Err(InvalidLength(0)));
        assert_eq!(AssetID::from_str("!"), Err(InvalidLength(1)));
        assert_eq!(
            AssetID::from_str("67e5504410b1426%9247bb680e5fe0c8"),
            Err(InvalidCharacter('%', 15))
        );

        // Valid
        assert!(AssetID::from_str("00000000000000000000000000000000").is_ok());
        assert!(AssetID::from_str("67e5504410b1426f9247bb680e5fe0c8").is_ok());
    }

    #[test]
    fn assetid_as_bytes() {
        let expected = [
            0xa1,
            0xa2,
            0xa3,
            0xa4,
            0xb1,
            0xb2,
            0xc1,
            0xc2,
            0xd1,
            0xd2,
            0xd3,
            0xd4,
            0xd5,
            0xd6,
            0xd7,
            0xd8,
        ];
        let assetid = AssetID::from_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap();

        assert_eq!(assetid.as_bytes(), &expected);
    }

    #[test]
    fn assetid_to_string() {
        let b = [
            0xa1,
            0xa2,
            0xa3,
            0xa4,
            0xb1,
            0xb2,
            0xc1,
            0xc2,
            0xd1,
            0xd2,
            0xd3,
            0xd4,
            0xd5,
            0xd6,
            0xd7,
            0xd8,
        ];

        let assetid = AssetID::from_bytes(&b).unwrap();
        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";

        assert_eq!(assetid.to_string(), expected);
    }

    #[test]
    fn assetid_read() {
        let buffer = vec![0; 16];
        unsafe {
            let assetid = AssetID::read(&buffer, 0, 16);
            assert_eq!(assetid, AssetID::zero());
        }

        let buffer = vec![
            0xa1,
            0xa2,
            0xa3,
            0xa4,
            0xb1,
            0xb2,
            0xc1,
            0xc2,
            0xd1,
            0xd2,
            0xd3,
            0xd4,
            0xd5,
            0xd6,
            0xd7,
            0xd8,
        ];
        unsafe {
            let assetid = AssetID::read(&buffer, 0, buffer.len() as Offset);
            let expected = AssetID::from_bytes(&buffer).unwrap();
            assert_eq!(assetid, expected);
        }

        let mut extended_buffer = vec![0xde, 0xad];
        extended_buffer.extend(&buffer);
        unsafe {
            let assetid = AssetID::read(&extended_buffer, 2, extended_buffer.len() as Offset);
            let expected = AssetID::from_bytes(&buffer).unwrap();
            assert_eq!(assetid, expected);
        }
    }

    #[test]
    fn assetid_write() {
        let expected = [
            0xa1,
            0xa2,
            0xa3,
            0xa4,
            0xb1,
            0xb2,
            0xc1,
            0xc2,
            0xd1,
            0xd2,
            0xd3,
            0xd4,
            0xd5,
            0xd6,
            0xd7,
            0xd8,
        ];
        let assetid = AssetID::from_bytes(&expected).unwrap();
        let mut buffer = vec![0; expected.len()];

        assetid.write(&mut buffer, 0, expected.len() as Offset);
        assert_eq!(buffer, expected);

        let expected = [
            0x0,
            0x0,
            0xa1,
            0xa2,
            0xa3,
            0xa4,
            0xb1,
            0xb2,
            0xc1,
            0xc2,
            0xd1,
            0xd2,
            0xd3,
            0xd4,
            0xd5,
            0xd6,
            0xd7,
            0xd8,
            0x0,
            0x0,
        ];
        let assetid = AssetID::from_bytes(&expected[2..18]).unwrap();
        let mut buffer = vec![0; expected.len()];

        assetid.write(&mut buffer, 2, 18);
        assert_eq!(buffer, expected);
    }
}
