#![no_std]

//! Primitive fixed-point decimal types.
//!
//! Rust built-in `f32` and `f64` types have two drawbacks:
//!
//! 1. can not represent decimal numbers in base 10 accurately, because they are in base 2;
//!
//! 2. can not guarantee the fraction precision, because they are floating-point.
//!
//! This crate provides fixed-point decimal types to address the issues by
//!
//! 1. using integer types to represent numbers with a scaling factor (also
//!    called as "scale") in base 10 to achieve the accuracy. This is a
//!    [common idea](https://en.wikipedia.org/wiki/Fixed-point_arithmetic#Representation).
//!    Many other decimal crates do the same thing;
//!
//! 2. specifying the scale staticly to guarantee the fraction precision.
//!    The scale is bound to the decimal type. It's fixed-point. Surprisingly,
//!    it seems that [no crate has done this before](https://github.com/WuBingzheng/primitive_fixed_point_decimal/blob/master/COMPARISON.md).
//!
//! For example, `ConstScaleFpdec<i64, 4>` means using `i64` as the underlying
//! representation, and `4` is the static scale.
//!
//! The "primitive" in the crate name means straightforward representation,
//! compact memory layout, high performance, and clean APIs, just like Rust's
//! primitive number types.
//!
//! This crate is `no_std`.
//!
//!
//! # Distinctive
//!
//! Although other decimal crates also claim to be fixed-point, they all
//! bind the scale to each decimal *instance*, which changes during operations.
//! They're more like
//! [decimal floating point](https://en.wikipedia.org/wiki/Decimal_floating_point).
//! See the [comparison documentation](https://github.com/WuBingzheng/primitive_fixed_point_decimal/blob/master/COMPARISON.md)
//! for details.
//!
//! While this crate binds the scale to decimal *type*.
//! The decimal types keep their scale for their whole lifetime
//! instead of changing their scale during operations.
//!
//! The `+`, `-` and comparison operations only perform between same types
//! in same scale. There is no implicitly type or scale conversion.
//! This makes sence, for we do not want to add balance type by
//! fee-rate type. Even for two balance types we do not want to add
//! USD currency by CNY. This also makes the operations very fast.
//!
//! However, the `*` and `/` operations accept operand with different
//! types and scales, and allow the result's scale specified.
//! Certainly we need to multiply between balance type and fee-rate type
//! and get balance type.
//!
//! If each decimal type has a fixed scale in you application, which means
//! all the decimal instances under each type have the same scale, it's
//! suitable for this crate. Otherwise, you should use other floating-point
//! decimal crates.
//!
//! See the examples below for more details.
//!
//!
//! # Specify Scale
//!
//! There are 2 ways to specify the scale: *const* and *out-of-band*:
//!
//! - For the *const* type [`ConstScaleFpdec`], we use Rust's *const generics*
//!   to specify the scale. For example, `ConstScaleFpdec<i64, 4>` means
//!   scale is 4.
//!
//! - For the *out-of-band* type [`OobScaleFpdec`], we do NOT save the
//!   scale with decimal types, so it's your job to save it somewhere
//!   and apply it in the following operations later. For example,
//!   `OobScaleFpdec<i64>` takes no scale information.
//!
//! Generally, the *const* type is more convenient and suitable for most
//! scenarios. For example, in traditional currency exchange, you can use
//! `ConstScaleFpdec<i64, 2>` to represent balance, e.g. `1234.56` USD and
//! `8888800.00` JPY. And use `ConstScaleFpdec<i32, 6>` to represent all
//! market prices since 6-digit-scale is big enough for all currency
//! pairs, e.g. `146.4730` JPY/USD and `0.006802` USD/JPY:
//!
//! ```
//! use primitive_fixed_point_decimal::{ConstScaleFpdec, fpdec};
//! type Balance = ConstScaleFpdec<i64, 2>; // 2 is enough for all currencies
//! type Price = ConstScaleFpdec<i32, 6>; // 6 is enough for all markets
//!
//! let usd: Balance = fpdec!(1234.56);
//! let price: Price = fpdec!(146.4730);
//!
//! let jpy: Balance = usd.checked_mul(price).unwrap();
//! assert_eq!(jpy, fpdec!(180829.70688));
//! ```
//!
//! However in some scenarios, such as in cryptocurrency exchange, the
//! price differences across various markets are very significant. For
//! example `81234.0` in BTC/USDT and `0.000004658` in PEPE/USDT. Here
//! we need to select different scales for each market. So it's
//! the *Out-of-band* type:
//!
//! ```
//! use primitive_fixed_point_decimal::{OobScaleFpdec, fpdec};
//! type Balance = OobScaleFpdec<i64>; // no global scale set
//! type Price = OobScaleFpdec<i32>; // no global scale set
//!
//! // each market has its own scale configuration
//! struct Market {
//!     base_asset_scale: i32,
//!     quote_asset_scale: i32,
//!     price_scale: i32,
//! }
//!
//! // let's take BTC/USDT market as example
//! let btc_usdt = Market {
//!     base_asset_scale: 8,
//!     quote_asset_scale: 6,
//!     price_scale: 1,
//! };
//!
//! // we need tell the scale to `fpdec!`
//! let btc: Balance = fpdec!(0.34, btc_usdt.base_asset_scale);
//! let price: Price = fpdec!(81234.0, btc_usdt.price_scale);
//!
//! // we need tell the scale difference to `checked_mul()` method
//! let diff = btc_usdt.base_asset_scale + btc_usdt.price_scale - btc_usdt.quote_asset_scale;
//! let usdt = btc.checked_mul(price, diff).unwrap();
//! assert_eq!(usdt, fpdec!(27619.56, btc_usdt.quote_asset_scale));
//! ```
//!
//! Obviously it's verbose to use, but offers greater flexibility.
//!
//! Another example is the SQL `Decimal` data type.
//! In the server end, the scale of each decimal column is fixed on created
//! (at runtime), so it fits `OobScaleFpdec`.
//! While in the client end, the application knows the business logical and
//! the scale of each decimal column ahead (at compilation time), so it fits
//! `ConstScaleFpdec`.
//!
//!
//! # Cumulative Error
//!
//! As is well known, integer division can lead to precision loss; multiplication
//! of decimals can also create higher precision and may potentially cause
//! precision loss.
//!
//! What we are discussing here is another issue: multiple multiplication and
//! division may cause cumulative error, thereby exacerbating the issue of
//! precision loss. See [`int-div-cum-error`](https://docs.rs/int-div-cum-error)
//! for more information.
//!
//! In this crate, functions with the `cum_error` parameter provide control
//! over cumulative error based on `int-div-cum-error`.
//!
//! Take the transaction fees in an exchange as an example. An order may be
//! executed in multiple deals, with each deal independently charged a fee.
//! For instance, the funds scale is 2 decimal places, one order quantity
//! is `10.00` USD, and the fee rate is `0.003`. If the order is executed all
//! at once, the fee would be `10.00 × 0.003 = 0.03` USD. However, if the
//! order is executed in five separate deals, each worth 2.00 USD, then the
//! fee for each deal would be `2.00 × 0.003 = 0.006` USD, which rounds up
//! to `0.01` USD. Then the total fee for the 5 deals would be `0.05` USD,
//! which is significantly higher than the original `0.03` USD.
//!
//! However, this issue can be avoid if using the cum_error mechanism.
//!
//! ```
//! use primitive_fixed_point_decimal::{ConstScaleFpdec, Rounding, fpdec};
//! type Balance = ConstScaleFpdec<i64, 2>;
//! type FeeRate = ConstScaleFpdec<i16, 6>;
//!
//! let deal: Balance = fpdec!(2.00); // 2.00 for each deal
//! let fee_rate: FeeRate = fpdec!(0.003);
//!
//! // normal case
//! let mut total_fee = Balance::ZERO;
//! for _ in 0..5 {
//!     total_fee += deal.checked_mul(fee_rate).unwrap(); // 2.00*0.003=0.006 ~> 0.01
//! }
//! assert_eq!(total_fee, fpdec!(0.05)); // 0.05 is too big
//!
//! // use `cum_error`
//! let mut cum_error = 0;
//! let mut total_fee = Balance::ZERO;
//! for _ in 0..5 {
//!     total_fee += deal.checked_mul_ext(fee_rate, Rounding::Round, Some(&mut cum_error)).unwrap();
//! }
//! assert_eq!(total_fee, fpdec!(0.03)); // 0.03 is right
//! ```
//!
//!
//! # Features
//!
//! - `serde` enables serde traits integration (`Serialize`/`Deserialize`).

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
pub use int_div_cum_error::Rounding;

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
