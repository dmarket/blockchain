
//! A definition of `StorageValue` trait and implementations for common types.

use std::borrow::Cow;
use std::mem;

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use crypto::{PublicKey, PUBLIC_KEY_LENGTH};
use messages::{MessageBuffer, RawMessage};

pub trait StorageValue: Sized {

    /// Serialize a value into a vector of bytes.
    fn into_bytes(self) -> Vec<u8>;

    /// Deserialize a value from bytes.
    fn from_bytes(value: Cow<[u8]>) -> Self;
}

/// No-op implementation.
impl StorageValue for () {
    fn into_bytes(self) -> Vec<u8> {
        Vec::new()
    }

    fn from_bytes(_value: Cow<[u8]>) -> Self {
        ()
    }
}

impl StorageValue for u8 {
    fn into_bytes(self) -> Vec<u8> {
        vec![self]
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        value[0]
    }
}

/// Uses little-endian encoding.
impl StorageValue for u16 {
    fn into_bytes(self) -> Vec<u8> {
        let mut v = vec![0; 2];
        LittleEndian::write_u16(&mut v, self);
        v
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        LittleEndian::read_u16(value.as_ref())
    }
}

/// Uses little-endian encoding.
impl StorageValue for u32 {
    fn into_bytes(self) -> Vec<u8> {
        let mut v = vec![0; 4];
        LittleEndian::write_u32(&mut v, self);
        v
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        LittleEndian::read_u32(value.as_ref())
    }
}

/// Uses little-endian encoding.
impl StorageValue for u64 {
    fn into_bytes(self) -> Vec<u8> {
        let mut v = vec![0; mem::size_of::<u64>()];
        LittleEndian::write_u64(&mut v, self);
        v
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        LittleEndian::read_u64(value.as_ref())
    }
}

impl StorageValue for i8 {
    fn into_bytes(self) -> Vec<u8> {
        vec![self as u8]
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        value[0] as i8
    }
}

/// Uses little-endian encoding.
impl StorageValue for i16 {
    fn into_bytes(self) -> Vec<u8> {
        let mut v = vec![0; 2];
        LittleEndian::write_i16(&mut v, self);
        v
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        LittleEndian::read_i16(value.as_ref())
    }
}

/// Uses little-endian encoding.
impl StorageValue for i32 {
    fn into_bytes(self) -> Vec<u8> {
        let mut v = vec![0; 4];
        LittleEndian::write_i32(&mut v, self);
        v
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        LittleEndian::read_i32(value.as_ref())
    }
}

/// Uses little-endian encoding.
impl StorageValue for i64 {
    fn into_bytes(self) -> Vec<u8> {
        let mut v = vec![0; 8];
        LittleEndian::write_i64(&mut v, self);
        v
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        LittleEndian::read_i64(value.as_ref())
    }
}

impl StorageValue for PublicKey {
    fn into_bytes(self) -> Vec<u8> {
        self.as_ref().to_vec()
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        PublicKey::from_slice(value.as_ref()).unwrap()
    }
}

impl StorageValue for RawMessage {
    fn into_bytes(self) -> Vec<u8> {
        self.as_ref().to_vec()
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        Self::new(MessageBuffer::from_vec(value.into_owned()))
    }
}

impl StorageValue for Vec<u8> {
    fn into_bytes(self) -> Vec<u8> {
        self
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        value.into_owned()
    }
}

/// Uses UTF-8 string serialization.
impl StorageValue for String {
    fn into_bytes(self) -> Vec<u8> {
        String::into_bytes(self)
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        String::from_utf8(value.into_owned()).unwrap()
    }
}

pub trait StorageKey {
    /// Returns the size of the serialized key in bytes.
    fn size(&self) -> usize;

    /// Serializes the key into the specified buffer of bytes.
    ///
    /// The caller must guarantee that the size of the buffer is equal to the precalculated size
    /// of the serialized key.
    // TODO: should be unsafe (ECR-174)?
    fn write(&self, buffer: &mut [u8]);

    /// Deserializes the key from the specified buffer of bytes.
    // TODO: should be unsafe (ECR-174)?
    fn read(buffer: &[u8]) -> Self;
}

/// No-op implementation.
impl StorageKey for () {
    fn size(&self) -> usize {
        0
    }

    fn write(&self, _buffer: &mut [u8]) {
        // no-op
    }

    fn read(_buffer: &[u8]) -> Self {
        ()
    }
}

impl StorageKey for u8 {
    fn size(&self) -> usize {
        1
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer[0] = *self
    }

    fn read(buffer: &[u8]) -> Self {
        buffer[0]
    }
}

/// Uses big-endian encoding.
impl StorageKey for u16 {
    fn size(&self) -> usize {
        2
    }

    fn write(&self, buffer: &mut [u8]) {
        BigEndian::write_u16(buffer, *self)
    }

    fn read(buffer: &[u8]) -> Self {
        BigEndian::read_u16(buffer)
    }
}

/// Uses big-endian encoding.
impl StorageKey for u32 {
    fn size(&self) -> usize {
        4
    }

    fn write(&self, buffer: &mut [u8]) {
        BigEndian::write_u32(buffer, *self)
    }

    fn read(buffer: &[u8]) -> Self {
        BigEndian::read_u32(buffer)
    }
}

/// Uses big-endian encoding.
impl StorageKey for u64 {
    fn size(&self) -> usize {
        8
    }

    fn write(&self, buffer: &mut [u8]) {
        BigEndian::write_u64(buffer, *self)
    }

    fn read(buffer: &[u8]) -> Self {
        BigEndian::read_u64(buffer)
    }
}

/// **Not sorted in the natural order.**
impl StorageKey for i8 {
    fn size(&self) -> usize {
        1
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer[0] = *self as u8
    }

    fn read(buffer: &[u8]) -> Self {
        buffer[0] as i8
    }
}

/// Uses big-endian encoding. **Not sorted in the natural order.**
impl StorageKey for i16 {
    fn size(&self) -> usize {
        2
    }

    fn write(&self, buffer: &mut [u8]) {
        BigEndian::write_i16(buffer, *self)
    }

    fn read(buffer: &[u8]) -> Self {
        BigEndian::read_i16(buffer)
    }
}

/// Uses big-endian encoding. **Not sorted in the natural order.**
impl StorageKey for i32 {
    fn size(&self) -> usize {
        4
    }

    fn write(&self, buffer: &mut [u8]) {
        BigEndian::write_i32(buffer, *self)
    }

    fn read(buffer: &[u8]) -> Self {
        BigEndian::read_i32(buffer)
    }
}

/// Uses big-endian encoding. **Not sorted in the natural order.**
impl StorageKey for i64 {
    fn size(&self) -> usize {
        8
    }

    fn write(&self, buffer: &mut [u8]) {
        BigEndian::write_i64(buffer, *self)
    }

    fn read(buffer: &[u8]) -> Self {
        BigEndian::read_i64(buffer)
    }
}

impl StorageKey for PublicKey {
    fn size(&self) -> usize {
        PUBLIC_KEY_LENGTH
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(self.as_ref())
    }

    fn read(buffer: &[u8]) -> Self {
        PublicKey::from_slice(buffer).unwrap()
    }
}

impl StorageKey for Vec<u8> {
    fn size(&self) -> usize {
        self.len()
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(self)
    }

    fn read(buffer: &[u8]) -> Self {
        buffer.to_vec()
    }
}

/// Uses UTF-8 string serialization.
impl StorageKey for String {
    fn size(&self) -> usize {
        self.len()
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(self.as_bytes())
    }

    fn read(buffer: &[u8]) -> Self {
        unsafe { ::std::str::from_utf8_unchecked(buffer).to_string() }
    }
}
