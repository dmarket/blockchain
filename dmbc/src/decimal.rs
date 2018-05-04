use std::error::Error;
use std::string::ToString;
use std::str::FromStr;
use std::ops::Mul;
use std::fmt;

use extprim::u128::u128;
use exonum::encoding::{Field, Offset, CheckedOffset};
use exonum::encoding::serialize::json::ExonumJson;
use exonum::encoding::serialize::WriteBufferWrapper;
use serde_json;

const BITS_PER_DIGIT: usize = 4;
const UFRACT64_DIGITS: usize = 16;

/// 64-bit unsigned packed binary-coded decimal fraction.
///
/// Range is from 0 to 0.9999999999999999.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct UFract64(u64);

impl UFract64 {
    /// Create a new `UFract64` from an array of bytes each representing a
    /// decimal place.
    pub fn from_digits(digits: [u8; UFRACT64_DIGITS]) -> Self {
        let mut fract = 0u64;

        let mut shift = 64;

        // Put each digit into a four bit nibble, with the most significant
        // nibble in u64 representing the most significant decimal place.
        for digit in &digits {
            debug_assert!(*digit < 10, "Each digit must be less than 10.");

            shift -= BITS_PER_DIGIT;
            fract |= ((*digit & 0xF) as u64) << shift;
        }

        UFract64(fract)
    }

    /// Get an array of bytes each representing a decimal place.
    pub fn to_digits(&self) -> [u8; UFRACT64_DIGITS] {
        let mut digits = [0u8; UFRACT64_DIGITS];
        for i in 0..UFRACT64_DIGITS {
            digits[i] = self.digit(i);
        }
        digits
    }

    /// Get the value of a decimal place at index.
    #[inline]
    pub fn digit(&self, index: usize) -> u8 {
        assert!(index < UFRACT64_DIGITS);
        ((self.0 >> ((UFRACT64_DIGITS - 1 - index) * BITS_PER_DIGIT)) & 0xF) as u8
    }

    /// Set the value of a decimal place at index.
    #[inline]
    pub fn set_digit(&mut self, index: usize, digit: u8) {
        assert!(index < 16);
        let shift = index * BITS_PER_DIGIT;
        let placed_digit = (digit as u64) << shift;
        let masked_fract = self.0 & !(0xF << shift);
        self.0 = masked_fract | placed_digit;
    }

    /// True if the number is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Mul<u64> for UFract64 {
    type Output = u64;

    fn mul(self, other: u64) -> u64 {
        if self.0 == 0 || other == 0 {
            return 0;
        }

        let mut result = u128::new(0);
        for i in 0..UFRACT64_DIGITS {
            let digit = self.digit(i) as u64;
            if digit != 0 {
                result += u128::new(other) * u128::new(digit) * u128::new(10).pow((UFRACT64_DIGITS-i) as u32);
            }
        }
        result /= u128::new(10).pow((UFRACT64_DIGITS+1) as u32);

        result.low64()
    }
}

impl ToString for UFract64 {
    fn to_string(&self) -> String {
        let mut result = String::with_capacity(2 + UFRACT64_DIGITS);
        result += "0.";
        result.extend(self.to_digits().into_iter().map(|digit| (digit + b'0') as char));
        result
    }
}

/// An error type representing a failure to parse the fraction.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FromStrError;

impl fmt::Display for FromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl Error for FromStrError {
    fn description(&self) -> &'static str {
        "Malformed UFract64 string representation."
    }
}

impl FromStr for UFract64 {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, FromStrError> {
        let mut s = s;

        if &s.as_bytes()[..2] == b"0." {
            s = s.split_at(2).1;
        }

        let mut digits = [0u8; UFRACT64_DIGITS];
        for (i, ch) in s.bytes().enumerate() {
            if i > UFRACT64_DIGITS || !ch.is_ascii_digit() {
                return Err(FromStrError);
            }
            digits[i] = ch - b'0';
        }

        Ok(UFract64::from_digits(digits))
    }
}

impl<'a> Field<'a> for UFract64 {
    // Just proxy all the calls to the u64 impl.

    fn field_size() -> Offset {
        u64::field_size()
    }

    unsafe fn read(buffer: &'a [u8], from: Offset, to: Offset) -> Self {
        UFract64(u64::read(buffer, from, to))
    }

    fn write(&self, buffer: &mut Vec<u8>, from: Offset, to: Offset) {
        self.0.write(buffer, from, to);
    }

    fn check(
        buffer: &'a [u8],
        from: CheckedOffset,
        to: CheckedOffset,
        latest_segment: CheckedOffset)
    -> Result<CheckedOffset, ::exonum::encoding::Error> {
        u64::check(buffer, from, to, latest_segment)
    }
}

impl ExonumJson for UFract64 {
    fn serialize_field(&self)
        -> Result<serde_json::value::Value, Box<Error + Send + Sync>> {
        Ok(serde_json::Value::String(self.to_string()))
    }

    fn deserialize_field<B: WriteBufferWrapper>(
        value: &serde_json::Value,
        buffer: &mut B,
        from: Offset,
        to: Offset,
    ) -> Result<(), Box<Error>> {
        let value = value.as_str().ok_or("UFract64 value is not a string")?;
        match str::parse::<UFract64>(value) {
            Ok(fract) => {
                buffer.write(from, to, fract);
                Ok(())
            }
            Err(err) => Err(Box::new(err)),
        }
    }
}

#[cfg(test)]
mod test {
    use std::string::ToString;

    use super::UFract64;

    #[test]
    fn create_from_digits() {
        let digits = [1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7];
        let fract = UFract64::from_digits(digits);
        assert_eq!(fract.to_digits(), digits);
    }

    #[test]
    fn to_string() {
        let digits = [1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7];
        let fract = UFract64::from_digits(digits);
        assert_eq!("0.1234567891234567", fract.to_string());
    }

    #[test]
    fn from_string() {
        let digits = [1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7];
        let fract_1 = UFract64::from_digits(digits);
        let fract_2 = "0.1234567891234567".parse().unwrap();
        assert_eq!(fract_1, fract_2);
    }

    #[test]
    fn multiply_percent() {
        let digits = [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let fract = UFract64::from_digits(digits);
        assert_eq!(1u64, fract * 100);
    }

    #[test]
    fn multiply_long() {
        let digits = [7, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let fract = UFract64::from_digits(digits);
        assert_eq!(97, fract * 1_23);
    }

    #[test]
    fn multiply_longer() {
        let digits = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0, 0];
        let fract = UFract64::from_digits(digits);
        assert_eq!(15241578, fract * 123_456_789);
    }
}
