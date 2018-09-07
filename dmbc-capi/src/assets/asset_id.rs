use std::error::Error;
use std::fmt;
use std::string::ToString;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::{Serialize, Serializer};

use crypto::PublicKey;
use encoding;
use encoding::{CheckedOffset, Field, Offset};
use exonum::storage::StorageKey;
use uuid;
use uuid::Uuid;

pub const ASSET_ID_LEN: usize = 16;

/// An identifier for an asset.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct AssetId(pub [u8; ASSET_ID_LEN]);

impl AssetId {
    /// Create zero `AssetId`.
    pub fn zero() -> AssetId {
        AssetId([0; 16])
    }

    pub fn from_data(data: &str, pub_key: &PublicKey) -> AssetId {
        let s = pub_key.to_hex();
        let ful_s = s + &data;

        let uuid = Uuid::new_v5(&uuid::NAMESPACE_DNS, &ful_s);
        AssetId::from_slice(uuid.as_bytes()).unwrap()
    }

    /// Create an `AssetId` from a slice of bytes.
    pub fn from_slice(b: &[u8]) -> Result<AssetId, ParseError> {
        let len = b.len();
        if len != ASSET_ID_LEN {
            return Err(ParseError::InvalidLength(len));
        }

        let mut assetid = AssetId::zero();
        assetid.0.copy_from_slice(b);
        Ok(assetid)
    }

    /// Create an `AssetId` from its hexadecimal representation.
    pub fn from_hex(hex: &str) -> Result<AssetId, ParseError> {
        let len = hex.len();
        if len != ASSET_ID_LEN * 2 {
            return Err(ParseError::InvalidLength(len));
        }

        let mut cs = hex.chars().enumerate();
        for (i, c) in cs.by_ref() {
            if !c.is_digit(16) {
                return Err(ParseError::InvalidCharacter(c, i));
            }
        }

        let mut bytes = [0u8; 16];

        for i in 0..bytes.len() {
            let offset = i * 2;
            let to = offset + 2;
            match u8::from_str_radix(&hex[offset..to], 16) {
                Ok(byte) => bytes[i] = byte,
                Err(..) => return Err(ParseError::UnexpectedError(offset)),
            }
        }

        Ok(AssetId(bytes))
    }

    /// Returns the hex representation of the binary data.
    /// Lower case letters are used (e.g. f9b4ca).
    pub fn to_hex(&self) -> String {
        let mut assetid_hex = "".to_string();
        let len = self.0.len();
        for i in 0..len {
            let byte_hex = format!("{:02x}", self.0[i]);
            assetid_hex += &*byte_hex;
        }
        assetid_hex
    }
}

impl<'a> Field<'a> for AssetId {
    fn field_size() -> Offset {
        ASSET_ID_LEN as Offset
    }

    unsafe fn read(buffer: &'a [u8], from: Offset, to: Offset) -> AssetId {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&buffer[from as usize..to as usize]);
        AssetId(bytes)
    }

    fn write(&self, buffer: &mut Vec<u8>, from: Offset, to: Offset) {
        buffer[from as usize..to as usize].copy_from_slice(&self.0);
    }

    fn check(
        _: &'a [u8],
        from: CheckedOffset,
        to: CheckedOffset,
        latest_segment: CheckedOffset,
    ) -> encoding::Result {
        if (to - from)?.unchecked_offset() != Self::field_size() {
            Err(encoding::Error::OffsetOverflow)
        } else {
            Ok(latest_segment)
        }
    }
}

/// Error details for string parsing failures.
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ParseError {
    InvalidLength(usize),
    InvalidCharacter(char, usize),
    UnexpectedError(usize),
}

const SIMPLE_LENGTH: usize = 32;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidLength(found) => write!(
                f,
                "Invalid length; expecting {}, found {}",
                SIMPLE_LENGTH, found
            ),
            ParseError::InvalidCharacter(found, pos) => write!(
                f,
                "Invalid character; found `{}` (0x{:02x}) at offset {}",
                found, found as usize, pos
            ),
            ParseError::UnexpectedError(pos) => write!(f, "Unexpected, at {}", pos),
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "AssetId parse error"
    }
}

impl StorageKey for AssetId {
    fn size(&self) -> usize {
        ASSET_ID_LEN
    }

    fn read(buffer: &[u8]) -> Self {
        let mut bytes = [0; ASSET_ID_LEN];
        bytes.copy_from_slice(buffer);
        AssetId(bytes)
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.0);
    }
}

impl ToString for AssetId {
    fn to_string(&self) -> String {
        self.to_hex()
    }
}

impl fmt::Debug for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AssetId({})", self.to_string())
    }
}

impl Serialize for AssetId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = self.to_hex();
        serializer.serialize_str(&hex_string)
    }
}

impl<'de> Deserialize<'de> for AssetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HexVisitor;

        impl<'v> Visitor<'v> for HexVisitor {
            type Value = AssetId;
            fn expecting(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                write!(fmt, "expecting str.")
            }
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                AssetId::from_hex(s).map_err(|_| de::Error::custom("Invalid hex"))
            }
        }
        deserializer.deserialize_str(HexVisitor)
    }
}
