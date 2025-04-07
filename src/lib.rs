//! Primitive fixed-point decimal types.
//!
//! It's necessary to represent decimals accurately in some scenarios,
//! such as financial field. Primitive floating-point types (`f32`
//! and `f64`) can not accurately represent decimal fractions because
//! they represent values in binary. Here we use integer types to
//! represent values, and handle fractions in base 10.
//!
//! Primitive integers `i16`, `i32`, `i64` and `i128` are used to represent
//! values, which can represent about 4, 9, 18 and 38 decimal significant
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
//! # Specify Precision
//!
//! There are 2 ways to specify the precision: *static* and *out-of-band*:
//!
//! - For the *static* type, we use Rust's *const generics* to specify the
//!   precision. For example, `StaticPrecFpdec16<2>` represents `2` decimal
//!   precision and its range represented is `-327.68` ~ `327.67`.
//!
//! - For the *out-of-band* type, we do NOT save the precision with our decimal
//!   types, so it's your job to save it somewhere and apply it in the
//!   following operations later. For example, `OobPrecFpdec16` represents
//!   significant digits only but no precision information.
//!
//! Generally, the *static* type is more convenient and suitable for most
//! scenarios. For example, in traditional currency exchange, you can use
//! `StaticPrecFpdec64<2>` to represent balance, e.g. `1234.56` USD and `8888800.00` JPY.
//! And use `StaticPrecFpdec32<6>` to represent all market prices since 6-digit-precision
//! is big enough for all currency pairs, e.g. `146.4730` JPY/USD and `0.006802` USD/JPY:
//!
//! ```ignore
//! type Balance = StaticPrecFpdec64<2>;
//! type Price = StaticPrecFpdec32<6>; // 6 is big enough for all markets
//!
//! let usd = Balance::try_from(1234.56).unwrap();
//! let price = Price::try_from(146.4730).unwrap();
//!
//! let jpy: Balance = usd.checked_mul(price).unwrap();
//! ```
//!
//! However in some scenarios, such as in cryptocurrency exchange, the
//! price differences across various markets are very significant. For
//! example `81234.0` in BTC/USDT and `0.000004658` in PEPE/USDT. Here
//! we need to select different precisions for each market. So it's
//! the *Out-of-band* type:
//!
//! ```ignore
//! type Balance = OobPrecFpdec64;
//! type Price = OobPrecFpdec32; // no precision set
//!
//! let btc_usdt = Market {
//!     base_asset_precision: 8,
//!     quote_asset_precision: 6,
//!     price_precision: 1,
//! };
//!
//! // we need tell the precisions to `try_from_f64()` method
//! let btc = Balance::try_from_f64(0.34, btc_usdt.base_asset_precision).unwrap();
//! let price = Price::try_from_f64(81234.0, btc_usdt.price_precision).unwrap();
//!
//! // we need tell the precision difference to `checked_mul()` method
//! let diff = btc_usdt.base_asset_precision + btc_usdt.price_precision - btc_usdt.quote_asset_precision;
//! let usdt = btc.checked_mul(price, diff).unwrap();
//! ```
//!
//! Obviously it's verbose to use, but offers greater flexibility.
//!
//! You can even use the 2 types at same time. For example, use *out-of-band*
//! type for balance which have different precisions for different assets; and
//! use *static* type for fee-rate which has a fixed precision:
//!
//! ```ignore
//! type Balance = OobPrecFpdec64; // out-of-band type
//! type FeeRate = StaticPrecFpdec16<6>; // static type
//!
//! let btc = Balance::try_from_f64(0.34, btc_precision).unwrap();
//! let fee_rate = FeeRate::try_from(0.0002).unwrap();
//!
//! let fee = btc.checked_mul_static(fee_rate).unwrap();
//! ```
//!
//! # Characteristics
//!
//! It is a common idea to use integers to represent decimals. But we have
//! some specialties.
//!
//! The `+`, `-` and comparison operations only perform between same types in
//! same precision. There is no implicitly type or precision conversion.
//! This makes sence. For example, you do not want to add balance type by
//! exchange rate type. This also makes the operations very fast.
//!
//! However, the `*` and `/` operations accept operand with different
//! precisions. Certainly we need to multiply between balance type
//! and exchange rates type.
//!
//! # Features
//!
//! - `serde` enables serde traits integration (`Serialize`/`Deserialize`)
//!   for *static* precision type. While the *out-of-band* type does not
//!   support serde at all.
//!
//! # Status
//!
//! More tests are need before ready for production.

mod fpdec_16;
mod fpdec_32;
mod fpdec_64;
mod fpdec_128;

// fpdec_16/32/64/128
//   -+-> define_convert
//    \-> define_both_fpdecs
//          -+-> define_calculations
//           |-> define_static_prec_fpdec -> define_common
//           \-> define_oob_prec_fpdec    -> define_common
mod define_convert;
mod define_both_fpdecs;
mod define_calculations;
mod define_static_prec_fpdec;
mod define_oob_prec_fpdec;
mod define_common;

mod oob_fmt;
mod utils;

pub use crate::fpdec_16::{StaticPrecFpdec16, OobPrecFpdec16};
pub use crate::fpdec_32::{StaticPrecFpdec32, OobPrecFpdec32};
pub use crate::fpdec_64::{StaticPrecFpdec64, OobPrecFpdec64};
pub use crate::fpdec_128::{StaticPrecFpdec128, OobPrecFpdec128};

pub use crate::oob_fmt::OobFmt;

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

/// Rounding kind.
///
/// This works right for non-negative numbers only by now, for
/// perfomance considerations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    Round,
    Floor,
    Ceil,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_amount_types() {
        use std::str::FromStr;

        let amount = StaticPrecFpdec64::<8>::from_str("80000").unwrap();
        let rate = StaticPrecFpdec16::<4>::from_str("0.015").unwrap();

        let fee = StaticPrecFpdec64::<8>::from_str("1200").unwrap();
        assert_eq!(amount.checked_mul(rate.into()).unwrap(), fee);
    }
}
