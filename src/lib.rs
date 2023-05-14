//! Approximate number of significant digits in base 10.
//!
//! XXXX

mod fixdec16;
mod fixdec32;
mod fixdec64;
mod fixdec128;

#[macro_use]
mod define_macro;

pub use crate::fixdec16::DIGITS as FIXDEC16_DIGITS;
pub use crate::fixdec32::DIGITS as FIXDEC32_DIGITS;
pub use crate::fixdec64::DIGITS as FIXDEC64_DIGITS;
pub use crate::fixdec128::DIGITS as FIXDEC128_DIGITS;

pub use crate::fixdec16::FixDec16;
pub use crate::fixdec32::FixDec32;
pub use crate::fixdec64::FixDec64;
pub use crate::fixdec128::FixDec128;

/// Error in converting from string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Empty string.
    Empty,
    /// Invalid digit in the string.
    Invalid,
    /// Overflow.
    Overflow,
    /// Too many precisions with Rounding::Error specified.
    Precision,
}

/// Rounding kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rounding {
    /// Return the nearest number. Round away from 0 for half-way (0.5).
    Round,
    /// Round toward 0. Just truncate the extra precision. It's equivalent
    /// to `Floor` for positive numbers, and `Ceiling` for negative numbers.
    Down,
    /// Round away from 0. It's equivalent to `Ceiling` for positive numbers,
    /// and `Floor` for negative numbers.
    Up,
    /// Return Option::None or Result::Err if need rounding.
    Unexpected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_amount_types() {
        use std::str::FromStr;

        let amount = FixDec64::<8>::from_str("80000").unwrap();
        let rate = FixDec16::<4>::from_str("0.015").unwrap();

        let fee = FixDec64::<8>::from_str("1200").unwrap();
        assert_eq!(amount.checked_mul(rate.into()).unwrap(), fee);
    }
}

