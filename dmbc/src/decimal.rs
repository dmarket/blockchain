use std::string::ToString;
use std::ops::Mul;
use extprim::u128::u128;

const BITS_PER_DIGIT: usize = 4;
const UFRACT64_DIGITS: usize = 16;

/// 64-bit unsigned packed binary-coded decimal fraction.
///
/// Range is from 0 to 0.9999999999999999.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct UFract64(u64);

impl UFract64 {
    pub fn from_digits(digits: [u8; UFRACT64_DIGITS]) -> Self {
        let mut fract = 0u64;

        let mut shift = 64;

        for digit in &digits {
            debug_assert!(*digit < 10, "Each digit must be less than 10.");

            shift -= BITS_PER_DIGIT;
            fract |= ((*digit & 0xF) as u64) << shift;
        }

        UFract64(fract)
    }

    pub fn to_digits(&self) -> [u8; UFRACT64_DIGITS] {
        let mut digits = [0u8; UFRACT64_DIGITS];
        for i in 0..UFRACT64_DIGITS {
            digits[i] = self.digit(i);
        }
        digits
    }

    #[inline]
    pub fn digit(&self, index: usize) -> u8 {
        assert!(index < UFRACT64_DIGITS);
        ((self.0 >> ((UFRACT64_DIGITS - 1 - index) * BITS_PER_DIGIT)) & 0xF) as u8
    }

    #[inline]
    pub fn set_digit(&mut self, index: usize, digit: u8) {
        assert!(index < 16);
        let shift = index * BITS_PER_DIGIT;
        let placed_digit = (digit as u64) << shift;
        let masked_fract = self.0 & !(0xF << shift);
        self.0 = masked_fract | placed_digit;
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
        assert_eq!(97, fract * 123_456_789);
    }
}
