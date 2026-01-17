#![no_std]
#![doc = include_str!("../README.md")]

// modules:
//
//     ConstScaleFpdec                             OobScaleFpdec
//            ^                                          ^
//            +---------------\           /--------------+
//            |               |           |              |
// +----------+--------+  +---+-----------+---+  +-------+---------+
// | const_scale_fpdec |  | none_scale_common |  | oob_scale_fpdec |
// +-------------------+  +-------------------+  +-----------------+
// +---------------------------------------------------------------+
// |                fpdec_inner: FpdecInner trait                  |
// +---------------------------------------------------------------+
// +------------------------------------+  +-----------------------+
// |    inner_shorts: i8,i16,i32,i64    |  |   inner_i128: i128    |
// |                  u8,u16,u32,u64    |  |               u128    |
// +------------------------------------+  +-----------------------+
mod const_scale_fpdec;
mod fpdec_inner;
mod inner_i128;
mod inner_shorts;
mod none_scale_common;
mod oob_scale_fpdec;

pub use crate::const_scale_fpdec::ConstScaleFpdec;
pub use crate::fpdec_inner::FpdecInner;
pub use crate::oob_scale_fpdec::{OobFmt, OobScaleFpdec};

/// Error in converting from string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParseError {
    /// Empty string.
    Empty,
    /// Invalid digit in the string.
    Invalid,
    /// Overflow.
    Overflow,
    /// Precision out of range.
    Precision,
}

use core::num::{IntErrorKind, ParseIntError};
impl From<ParseIntError> for ParseError {
    fn from(pie: ParseIntError) -> Self {
        match pie.kind() {
            IntErrorKind::Empty => ParseError::Empty,
            IntErrorKind::InvalidDigit => ParseError::Invalid,
            _ => ParseError::Overflow,
        }
    }
}

use core::fmt;
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ParseError::Empty => "empty string",
            ParseError::Invalid => "invalid digit in the string",
            ParseError::Overflow => "overflow",
            ParseError::Precision => "precision out of range",
        };
        write!(f, "{s}")
    }
}

impl core::error::Error for ParseError {}

/// Rounding kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Rounding {
    /// towards the nearest integer
    #[default]
    Round,
    /// towards negative infinity
    Floor,
    /// towards positive infinity
    Ceiling,
    /// towards zero
    TowardsZero,
    /// away from zero
    AwayFromZero,
}

/// Build decimal from integer or float number easily.
///
/// It accepts 1 argument for `ConstScaleFpdec`, and accepts 1 extra
/// argument for `OobScaleFpdec`, the out-of-band scale of course.
///
/// Panics:
///
/// It wraps `TryFrom` trait and will panic if `try_from()` fails.
///
/// Examples:
///
/// ```
/// use primitive_fixed_point_decimal::{ConstScaleFpdec, OobScaleFpdec, fpdec};
/// type DecConst = ConstScaleFpdec<i64, 2>;
/// type DecOob = OobScaleFpdec<i64>;
///
/// let d1: DecConst = fpdec!(1.23); // 1 argument for ConstScaleFpdec
/// let d2: DecOob = fpdec!(1.23, 2); // 2 arguments for OobScaleFpdec
/// ```
#[macro_export]
macro_rules! fpdec {
    ($n:expr) => {
        primitive_fixed_point_decimal::ConstScaleFpdec::try_from($n).unwrap()
    };
    ($n:expr, $scale:expr) => {
        primitive_fixed_point_decimal::OobScaleFpdec::try_from(($n, $scale)).unwrap()
    };
}

/// Used by method `checked_mul_ratio()` only.
pub trait IntoRatioInt<T> {
    fn to_int(self) -> T;
}

/// For primitive integer types.
impl<I, T> IntoRatioInt<T> for I
where
    I: Into<T> + num_traits::PrimInt,
{
    fn to_int(self) -> T {
        self.into()
    }
}
