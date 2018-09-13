// Copyright 2017 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Cryptography related types and functions.
//!
//! [Sodium library](https://github.com/jedisct1/libsodium) is used under the hood through
//! [sodiumoxide rust bindings](https://github.com/dnaq/sodiumoxide).

use std::default::Default;
use std::fmt;
use std::ops::{Index, Range, RangeFrom, RangeFull, RangeTo};

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::{Serialize, Serializer};
use sodiumoxide::crypto::hash::sha256::{hash as hash_sodium, Digest};
use sodiumoxide::crypto::sign::ed25519::{
    PublicKey as PublicKeySodium,
    Signature as SignatureSodium,
};

use hex::{encode as encode_hex, FromHex, FromHexError};

pub use sodiumoxide::crypto::hash::sha256::DIGESTBYTES as HASH_SIZE;
pub use sodiumoxide::crypto::sign::ed25519::{
    PUBLICKEYBYTES as PUBLIC_KEY_LENGTH, SECRETKEYBYTES as SECRET_KEY_LENGTH,
    SEEDBYTES as SEED_LENGTH, SIGNATUREBYTES as SIGNATURE_LENGTH,
};

/// The size to crop the string in debug messages.
const BYTES_IN_DEBUG: usize = 4;

/// Calculates an SHA-256 hash digest of a bytes slice.
///
/// # Examples
///
/// ```
/// use exonum::crypto;
///
/// # crypto::init();
/// let data = [1, 2, 3];
/// let hash = crypto::hash(&data);
/// # drop(hash);
/// ```
pub fn hash(data: &[u8]) -> Hash {
    let dig = hash_sodium(data);
    Hash(dig)
}

macro_rules! implement_public_sodium_wrapper {
    ($(#[$attr:meta])* struct $name:ident, $name_from:ident, $size:expr) => (
    #[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
    $(#[$attr])*
    pub struct $name($name_from);

    impl $name {
        /// Creates a new instance filled with zeros.
        pub fn zero() -> Self {
            $name::new([0; $size])
        }
    }

    impl $name {
        /// Creates a new instance from bytes array.
        pub fn new(ba: [u8; $size]) -> Self {
            $name($name_from(ba))
        }

        /// Creates a new instance from bytes slice.
        pub fn from_slice(bs: &[u8]) -> Option<Self> {
            $name_from::from_slice(bs).map($name)
        }

        /// Returns the hex representation of the binary data.
        /// Lower case letters are used (e.g. f9b4ca).
        pub fn to_hex(&self) -> String {
            encode_hex(self)
        }
    }

    impl AsRef<[u8]> for $name {
        fn as_ref(&self) -> &[u8] {
            self.0.as_ref()
        }
    }

    impl ::std::str::FromStr for $name {
        type Err = FromHexError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            $name::from_hex(s)
        }
    }

    impl ToString for $name {
        fn to_string(&self) -> String {
            self.to_hex()
        }
    }

    impl fmt::Debug for $name {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, stringify!($name))?;
            write!(f, "(")?;
            for i in &self[0..BYTES_IN_DEBUG] {
                write!(f, "{:02X}", i)?
            }
            write!(f, ")")
        }
    }
    )
}

implement_public_sodium_wrapper! {
/// Ed25519 public key used to verify digital signatures.
///
/// # Examples
///
/// ```
/// use exonum::crypto;
///
/// # crypto::init();
/// let (public_key, _) = crypto::gen_keypair();
/// # drop(public_key);
/// ```
    struct PublicKey, PublicKeySodium, PUBLIC_KEY_LENGTH
}

implement_public_sodium_wrapper! {
/// SHA-256 hash.
///
/// # Examples
///
/// ```
/// use exonum::crypto::{self, Hash};
///
/// let data = [1, 2, 3];
/// let hash_from_data = crypto::hash(&data);
/// let default_hash = Hash::default();
/// # drop(hash_from_data);
/// # drop(default_hash);
/// ```
    struct Hash, Digest, HASH_SIZE
}

implement_public_sodium_wrapper! {
/// Ed25519 digital signature.
///
/// # Examples
///
/// ```
/// use exonum::crypto;
///
/// # crypto::init();
/// let (public_key, secret_key) = crypto::gen_keypair();
/// let data = [1, 2, 3];
/// let signature = crypto::sign(&data, &secret_key);
/// assert!(crypto::verify(&signature, &data, &public_key));
/// ```
    struct Signature, SignatureSodium, SIGNATURE_LENGTH
}

macro_rules! implement_serde {
    ($name:ident) => {
        impl FromHex for $name {
            type Error = FromHexError;

            fn from_hex<T: AsRef<[u8]>>(v: T) -> Result<Self, Self::Error> {
                let bytes = Vec::<u8>::from_hex(v)?;
                if let Some(self_value) = Self::from_slice(bytes.as_ref()) {
                    Ok(self_value)
                } else {
                    Err(FromHexError::InvalidStringLength)
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let hex_string = encode_hex(&self[..]);
                ser.serialize_str(&hex_string)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct HexVisitor;

                impl<'v> Visitor<'v> for HexVisitor {
                    type Value = $name;
                    fn expecting(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                        write!(fmt, "expecting str.")
                    }
                    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        $name::from_hex(s).map_err(|_| de::Error::custom("Invalid hex"))
                    }
                }
                deserializer.deserialize_str(HexVisitor)
            }
        }
    };
}

implement_serde! {Hash}
implement_serde! {PublicKey}
implement_serde! {Signature}

macro_rules! implement_index_traits {
    ($newtype:ident) => {
        impl Index<Range<usize>> for $newtype {
            type Output = [u8];
            fn index(&self, _index: Range<usize>) -> &[u8] {
                let inner = &self.0;
                inner.0.index(_index)
            }
        }
        impl Index<RangeTo<usize>> for $newtype {
            type Output = [u8];
            fn index(&self, _index: RangeTo<usize>) -> &[u8] {
                let inner = &self.0;
                inner.0.index(_index)
            }
        }
        impl Index<RangeFrom<usize>> for $newtype {
            type Output = [u8];
            fn index(&self, _index: RangeFrom<usize>) -> &[u8] {
                let inner = &self.0;
                inner.0.index(_index)
            }
        }
        impl Index<RangeFull> for $newtype {
            type Output = [u8];
            fn index(&self, _index: RangeFull) -> &[u8] {
                let inner = &self.0;
                inner.0.index(_index)
            }
        }
    };
}
implement_index_traits! {Hash}
implement_index_traits! {PublicKey}
implement_index_traits! {Signature}

/// Returns hash consisting of zeros.
impl Default for Hash {
    fn default() -> Hash {
        Hash::zero()
    }
}
