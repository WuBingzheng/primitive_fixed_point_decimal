#![cfg_attr(not(feature = "std"), no_std)]
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
#[cfg(feature = "num-traits-impls")]
mod num_traits_impls;
#[cfg(feature = "pyo3")]
mod pyo3_conv;
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
            Self::Empty => "empty string",
            Self::Invalid => "invalid digit in the string",
            Self::Overflow => "overflow",
            Self::Precision => "precision out of range",
        };
        write!(f, "{s}")
    }
}

impl core::error::Error for ParseError {}

/// Rounding kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Rounding {
    /// towards the nearest integer; a tie (exactly half-way) goes away from zero
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
        $crate::ConstScaleFpdec::try_from($n).unwrap()
    };
    ($n:expr, $scale:expr) => {
        $crate::OobScaleFpdec::try_from(($n, $scale)).unwrap()
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

/// `f * 10^n`, or `f / 10^n` when `divide`, without ever forming a power of ten the float
/// cannot hold (`f32` stops at `10^38`, `f64` at `10^308`): `n` is applied in chunks. A float
/// is divided by `10^n` rather than multiplied by `10^-n`, which is not exact.
#[allow(unused_imports)]
use num_traits::float::FloatCore; // `powi` etc. for `no_std`

macro_rules! pow10_scaler {
    ($name:ident, $float:ty, $chunk:expr) => {
        pub(crate) fn $name(mut f: $float, n: u32, divide: bool) -> $float {
            let step = |f: $float, n: u32| {
                let p = (10.0 as $float).powi(n as i32);
                if divide { f / p } else { f * p }
            };
            let mut n = n;
            while n > $chunk && f != 0.0 && f.is_finite() {
                f = step(f, $chunk);
                n -= $chunk;
            }
            step(f, n.min($chunk)) // once `f` is 0 / inf / NaN the rest cannot change it
        }
    };
}
pow10_scaler!(scale_f32, f32, 30);
pow10_scaler!(scale_f64, f64, 300);
