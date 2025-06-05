//! Primitive fixed-point decimal types.
//!
//! It's necessary to represent decimals accurately in some scenarios,
//! such as financial field. Primitive floating-point types (`f32`
//! and `f64`) can not accurately represent decimal fractions because
//! they represent values in binary. Here we use integer types to
//! represent values, and handle fractions in base 10.
//!
//! Primitive integers `i8`, `i16`, `i32`, `i64` and `i128` are used to represent
//! values, which can represent about 2, 4, 9, 18 and 38 decimal significant
//! digits respectively. So the number `12.345` is stored as `123450`
//! for decimal with precision `4`. See below to find how to specify
//! the precision.
//!
//! In addition, these scenarios generally require *fraction precision*,
//! rather than the *significant digits* like in scientific calculations,
//! so fixed-point is more suitable than floating-point.
//!
//! So here are the *primitive fixed-point* decimal types.
//!
//!
//! # Distinctive
//!
//! It is a common idea to use integers to represent decimals. But we have
//! some specialties.
//!
//! The decimal types here keep their precision for their whole lifetime
//! instead of changing their precision as soon as you perform an operation.
//!
//! The `+`, `-` and comparison operations only perform between same types in
//! same precision. There is no implicitly type or precision conversion.
//! This makes sence, for we do not want to add balance type by
//! fee-rate type. Even for two balance types we do not want to add
//! USD by CNY. This also makes the operations very fast.
//!
//! However, the `*` and `/` operations accept operand with different
//! types and precisions, and allow the result's precision specified. Certainly
//! we need to multiply between balance type and fee-rate type and get
//! fee type.
//!
//! See the examples below for more details.
//!
//!
//! # Specify Precision
//!
//! There are 2 ways to specify the precision: *static* and *out-of-band*:
//!
//! - For the *static* type, [`StaticPrecFpdec`], we use Rust's *const generics*
//!   to specify the precision. For example, `StaticPrecFpdec<i64, 4>` means
//!   4 precision.
//!
//! - For the *out-of-band* type, [`OobPrecFpdec`], we do NOT save the
//!   precision with our decimal types, so it's your job to save it somewhere
//!   and apply it in the following operations later. For example,
//!   `OobPrecFpdec<i64>` takes no precision information.
//!
//! Generally, the *static* type is more convenient and suitable for most
//! scenarios. For example, in traditional currency exchange, you can use
//! `StaticPrecFpdec<i64, 2>` to represent balance, e.g. `1234.56` USD and
//! `8888800.00` JPY. And use `StaticPrecFpdec<i32, 6>` to represent all
//! market prices since 6-digit-precision is big enough for all currency
//! pairs, e.g. `146.4730` JPY/USD and `0.006802` USD/JPY:
//!
//! ```
//! use primitive_fixed_point_decimal::{StaticPrecFpdec, fpdec};
//! type Balance = StaticPrecFpdec<i64, 2>;
//! type Price = StaticPrecFpdec<i32, 6>; // 6 is big enough for all markets
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
//! we need to select different precisions for each market. So it's
//! the *Out-of-band* type:
//!
//! ```
//! use primitive_fixed_point_decimal::{OobPrecFpdec, fpdec};
//! type Balance = OobPrecFpdec<i64>;
//! type Price = OobPrecFpdec<i32>; // no precision set
//!
//! struct Market {
//!     base_asset_precision: i32,
//!     quote_asset_precision: i32,
//!     price_precision: i32,
//! }
//!
//! let btc_usdt = Market {
//!     base_asset_precision: 8,
//!     quote_asset_precision: 6,
//!     price_precision: 1,
//! };
//!
//! // we need tell the precision, for `try_from()` and `fpdec!` both.
//! let btc = Balance::try_from((0.34, btc_usdt.base_asset_precision)).unwrap();
//! let price: Price = fpdec!(81234.0, btc_usdt.price_precision);
//!
//! // we need tell the precision difference to `checked_mul()` method
//! let diff = btc_usdt.base_asset_precision + btc_usdt.price_precision - btc_usdt.quote_asset_precision;
//! let usdt = btc.checked_mul(price, diff).unwrap();
//! assert_eq!(usdt, fpdec!(27619.56, btc_usdt.quote_asset_precision));
//! ```
//!
//! Obviously it's verbose to use, but offers greater flexibility.
//!
//! You can even use the 2 types at same time. For example, use *out-of-band*
//! type for balance which have different precisions for different assets; and
//! use *static* type for fee-rate which has a fixed precision:
//!
//! ```
//! use primitive_fixed_point_decimal::{StaticPrecFpdec, OobPrecFpdec, fpdec};
//! type Balance = OobPrecFpdec<i64>; // out-of-band type
//! type FeeRate = StaticPrecFpdec<i16, 6>; // static type
//!
//! let btc_precision = 8;
//!
//! let btc: Balance = fpdec!(0.34, btc_precision);
//! let fee_rate: FeeRate = fpdec!(0.0002);
//!
//! let fee = btc.checked_mul_static(fee_rate).unwrap();
//! assert_eq!(fee, fpdec!(0.000068, btc_precision));
//! ```
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
//! For instance, the funds precision is 2 decimal places, one order quantity
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
//! use primitive_fixed_point_decimal::{StaticPrecFpdec, OobPrecFpdec, Rounding, fpdec};
//! type Balance = StaticPrecFpdec<i64, 2>;
//! type FeeRate = StaticPrecFpdec<i16, 6>;
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
//! - `serde` enables serde traits integration (`Serialize`/`Deserialize`)
//!   for *static* precision type. While the *out-of-band* type does not
//!   support serde at all.
//!
//!
//! # Status
//!
//! More tests are need before ready for production.


// modules:
//
//     StaticPrecFpdec                            OobPrecFpdec
//            ^                                         ^
//            +---------------\          /--------------+
//            |               |          |              |
// +----------+--------+  +---+----------+---+  +-------+----------+
// | static_prec_fpdec |  | none_prec_common |  |  oob_prec_fpdec  |
// +-------------------+  +------------------+  +------------------+
// +---------------------------------------------------------------+
// |                fpdec_inner: FpdecInner trait                  |
// +---------------------------------------------------------------+
// +------------------------------------+  +-----------------------+
// |    inner_shorts: i8,i16,i32,i64    |  |   inner_i128: i128    |
// +------------------------------------+  +-----------------------+
mod static_prec_fpdec;
mod oob_prec_fpdec;
mod none_prec_common;
mod fpdec_inner;
mod inner_shorts;
mod inner_i128;

pub use int_div_cum_error::Rounding;
pub use crate::static_prec_fpdec::StaticPrecFpdec;
pub use crate::oob_prec_fpdec::{OobPrecFpdec, OobFmt};

/// Error in converting from string.
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParseError {
    /// Empty string.
    #[error("empty string")]
    Empty,
    /// Invalid digit in the string.
    #[error("invalid digit")]
    Invalid,
    /// Overflow.
    #[error("overflow")]
    Overflow,
    /// Precision out of range.
    #[error("precision out of range")]
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

/// Build decimal from integer or float number easily.
///
/// It wraps TryFrom trait and will panic if `try_from()` fails. So it
/// should be used by const numbers or for non-production codes.
///
/// It accepts 1 argument for `StaticPrecFpdec`, and accepts 1 extra
/// argument for `OobPrecFpdec`, the out-of-band precision of course.
///
/// Examples:
///
/// ```
/// use primitive_fixed_point_decimal::{StaticPrecFpdec, OobPrecFpdec, fpdec};
/// type DecStatic = StaticPrecFpdec<i64, 2>;
/// type DecOob = OobPrecFpdec<i64>;
///
/// let d1: DecStatic = fpdec!(1.23); // 1 argument for StaticPrecFpdec
/// let d2: DecOob = fpdec!(1.23, 2); // 2 arguments for OobPrecFpdec
/// ```
#[macro_export]
macro_rules! fpdec {
    ($n:expr) => {
        primitive_fixed_point_decimal::StaticPrecFpdec::try_from($n).unwrap()
    };
    ($n:expr, $precision:expr) => {
        primitive_fixed_point_decimal::OobPrecFpdec::try_from(($n, $precision)).unwrap()
    };
}
