mod fpdec_inner;
mod none_prec_common;
mod static_prec_fpdec;

pub use int_div_cum_error::Rounding;
pub use crate::static_prec_fpdec::StaticPrecFpdec;

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

use std::num::{ParseIntError, IntErrorKind};
impl From<ParseIntError> for ParseError {
    fn from(pie: ParseIntError) -> Self {
        match pie.kind() {
            IntErrorKind::Empty => ParseError::Empty,
            IntErrorKind::InvalidDigit => ParseError::Invalid,
            _ => ParseError::Overflow,
        }
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
