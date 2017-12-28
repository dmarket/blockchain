use exonum::crypto::PublicKey;
use exonum::encoding::{CheckedOffset, Field, Offset, Result as ExonumResult};
use exonum::encoding::serialize::WriteBufferWrapper;
use exonum::encoding::serialize::json::ExonumJson;
use exonum::storage::StorageKey;
use serde_json::value::Value;
use std::{fmt, mem};
use std::convert::From;
use std::error::Error;
use std::string::ToString;
use uuid;
use uuid::Uuid;

pub const ASSET_HASH_ID_MAX_LENGTH: usize = 10 * 1024; // 10 KBytes

encoding_struct! {
    struct Fee {
        const SIZE = 16;

        field tax:   u64  [0 => 8]
        field ratio: u64  [8 => 16]
    }
}

encoding_struct! {
    struct Fees {
        const SIZE = 24;

        field trade:    Fee [0 => 8]
        field exchange: Fee [8 => 16]
        field transfer: Fee [16 => 24]
    }
}

encoding_struct! {
    struct MetaAsset {
        const SIZE = 20;

        field data: &str   [0 => 8]
        field amount: u32  [8 => 12]
        field fees: Fees   [12 => 20]
    }
}

impl MetaAsset {
    pub fn is_valid(&self) -> bool {
        let trade_fee_is_ok = self.fees().trade().ratio() != 0;
        let exchange_fee_is_ok = self.fees().exchange().ratio() != 0;
        let transfer_fee_is_ok = self.fees().transfer().ratio() != 0;
        let data_is_ok = self.data().len() <= ASSET_HASH_ID_MAX_LENGTH;

        trade_fee_is_ok && exchange_fee_is_ok && transfer_fee_is_ok && data_is_ok
    }
}

encoding_struct! {
    struct AssetInfo {
        const SIZE = 44;

        field creator: &PublicKey [0  => 32]
        field amount:  u32        [32 => 36]
        field fees:    Fees       [36 => 44]
    }
}

/// A 128-bit (16 byte) buffer containing the ID.
type AssetIdBytes = [u8; 16];

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetId {
    /// The 128-bit number stored in 16 bytes
    bytes: AssetIdBytes,
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
        "AssetId parse error"
    }
}

impl AssetId {
    pub fn zero() -> AssetId {
        AssetId { bytes: [0u8; 16] }
    }

    /// Creates unique `AssetId` from
    /// `&str` and `&PublicKey`
    /// # Example:
    /// ```
    /// # extern crate exonum;
    /// # extern crate dmbc;
    /// #
    /// # fn main () {
    /// #
    /// # use exonum::crypto::{PublicKey};
    /// # use exonum::encoding::serialize::FromHex;
    /// # use dmbc::service::asset::AssetId;
    ///
    /// let data = "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f";
    /// let public_key = PublicKey::from_hex("3115dbc2ff73f4819672d5e9e421692305a9de1a18e4389df041c0cf6c8918a8").unwrap();
    ///
    /// let assetid = AssetId::new(&data, &public_key).unwrap();
    /// assert_eq!(assetid.to_string(), "82c1f90bed24508e9ce74b536f97fa9c");
    /// # }
    /// ```
    pub fn new(data: &str, pub_key: &PublicKey) -> Result<AssetId, ParseError> {
        let s = pub_key.to_hex();
        let ful_s = s + &data;

        let uuid = Uuid::new_v5(&uuid::NAMESPACE_DNS, &ful_s);
        AssetId::from_bytes(uuid.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    pub fn from_bytes(b: &[u8]) -> Result<AssetId, ParseError> {
        let len = b.len();
        if len != mem::size_of::<AssetIdBytes>() {
            return Err(ParseError::InvalidLength(len));
        }

        let mut assetid = AssetId::zero();
        assetid.bytes.copy_from_slice(b);
        Ok(assetid)
    }

    pub fn from_str(us: &str) -> Result<AssetId, ParseError> {
        let len = us.len();
        if len != mem::size_of::<AssetIdBytes>() * 2 {
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

        AssetId::from_bytes(&bytes)
    }
}

impl ToString for AssetId {
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


impl StorageKey for AssetId {
    fn size(&self) -> usize {
        mem::size_of::<AssetIdBytes>()
    }

    fn read(buffer: &[u8]) -> Self {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(buffer);
        AssetId { bytes: bytes }
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.bytes)
    }
}

impl<'a> Field<'a> for AssetId {
    fn field_size() -> Offset {
        mem::size_of::<AssetIdBytes>() as Offset
    }

    unsafe fn read(buffer: &'a [u8], from: Offset, to: Offset) -> AssetId {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&buffer[from as usize..to as usize]);
        AssetId { bytes: bytes }
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

impl ExonumJson for AssetId {
    fn deserialize_field<B: WriteBufferWrapper>(
        value: &Value,
        buffer: &mut B,
        from: Offset,
        to: Offset,
    ) -> Result<(), Box<Error>> {
        let val = value.as_str().ok_or("Can't cast json as string")?;
        match AssetId::from_str(&val) {
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

impl fmt::Debug for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AssetId({})", self.to_string())
    }
}

encoding_struct! {
    struct Asset {
        const SIZE = 20;

        field id: AssetId [0 =>  16]
        field amount:  u32 [16 => 20]
    }
}

impl Asset {
    pub fn is_eq(&self, other: &Asset) -> bool {
        self.id() == other.id()
    }

    pub fn is_available_to_transfer(&self, other: &Asset) -> bool {
        self.amount() >= other.amount()
    }

    pub fn from_meta_asset(meta_asset: &MetaAsset, pub_key: &PublicKey) -> Asset {
        Asset::from_parts(&meta_asset.data(), meta_asset.amount(), pub_key)
    }

    pub fn from_parts(s: &str, amount: u32, pub_key: &PublicKey) -> Asset {
        let id = AssetId::new(s, pub_key).unwrap();
        Asset::new(id, amount)
    }

    pub fn into_trade_asset(&self, price: u64) -> TradeAsset {
        TradeAsset::new(self.id(), self.amount(), price)
    }
}

impl From<TradeAsset> for Asset {
    fn from(trade_asset: TradeAsset) -> Asset {
        Asset::new(trade_asset.id(), trade_asset.amount())
    }
}

encoding_struct! {
    struct TradeAsset {
        const SIZE = 28;

        field id: AssetId [0 => 16]
        field amount: u32 [16 => 20]
        field price: u64  [20 => 28] 
    }
}

impl TradeAsset {
    pub fn total_price(&self) -> u64 {
        self.amount() as u64 * self.price()
    }
}

#[cfg(test)]
mod tests {
    use super::AssetId;
    use super::ParseError::*;
    use exonum::encoding::{Field, Offset};

    #[test]
    fn assetid_zero() {
        let assetid = AssetId::zero();
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

        let assetid = AssetId::from_bytes(&b).unwrap();

        assert_eq!(assetid.as_bytes(), &b);
    }

    #[test]
    fn assetid_from_str() {
        // Invalid
        assert_eq!(AssetId::from_str(""), Err(InvalidLength(0)));
        assert_eq!(AssetId::from_str("!"), Err(InvalidLength(1)));
        assert_eq!(
            AssetId::from_str("67e5504410b1426%9247bb680e5fe0c8"),
            Err(InvalidCharacter('%', 15))
        );

        // Valid
        assert!(AssetId::from_str("00000000000000000000000000000000").is_ok());
        assert!(AssetId::from_str("67e5504410b1426f9247bb680e5fe0c8").is_ok());
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
        let assetid = AssetId::from_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap();

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

        let assetid = AssetId::from_bytes(&b).unwrap();
        let expected = "a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8";

        assert_eq!(assetid.to_string(), expected);
    }

    #[test]
    fn assetid_read() {
        let buffer = vec![0; 16];
        unsafe {
            let assetid = AssetId::read(&buffer, 0, 16);
            assert_eq!(assetid, AssetId::zero());
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
            let assetid = AssetId::read(&buffer, 0, buffer.len() as Offset);
            let expected = AssetId::from_bytes(&buffer).unwrap();
            assert_eq!(assetid, expected);
        }

        let mut extended_buffer = vec![0xde, 0xad];
        extended_buffer.extend(&buffer);
        unsafe {
            let assetid = AssetId::read(&extended_buffer, 2, extended_buffer.len() as Offset);
            let expected = AssetId::from_bytes(&buffer).unwrap();
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
        let assetid = AssetId::from_bytes(&expected).unwrap();
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
        let assetid = AssetId::from_bytes(&expected[2..18]).unwrap();
        let mut buffer = vec![0; expected.len()];

        assetid.write(&mut buffer, 2, 18);
        assert_eq!(buffer, expected);
    }
}
