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
//!   precision. For example, `FixDec6416<2>` represents `2` decimal
//!   precision and its range represented is `-327.68` ~ `327.67`.
//!
//! - For the *out-of-band* type, we do NOT save the precision with our decimal
//!   types, so it's your job to save it somewhere and apply it in the
//!   following operations later. For example, `OobPrecFpdec16` represents
//!   significant digits only but no precision information.
//!
//! Generally, the *static* type is more convenient and suitable for most
//! scenarios. For example, in traditional currency exchange, you can use
//! `FixDec6464<2>` to represent balance, e.g. `1234.56` USD and `8888800.00` JPY.
//! And use `FixDec6432<6>` to represent all market prices since 6-digit-precision
//! is big enough for all currency pairs, e.g. `146.4730` JPY/USD and `0.006802` USD/JPY:
//!
//! ```rust
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
//! ```rust
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
//! # Characteristics
//!
//! It is a common idea to use integers to represent decimals. But we have
//! some specialties.
//!
//! The `+`, `-` and comparison operations only perform between same types in
//! same precision. There is no implicitly type or precision conversion.
//! This makes sence. For example, if you use `FixDec64<2>` to represent
//! balance and `FixDec64<6>` to represent exchange rates, there should be
//! no above operations between balance `FixDec64<2>` and exchange rates
//! `FixDec64<6>`.
//!
//! However, the `*` and `/` operations accept operand with different
//! precisions. Certainly we need to multiply between balance `FixDec64<2>`
//! and exchange rates `FixDec64<6>` to get another balance.
//!
//! Besides, the `*` and `/` operations can specify the precision of the
//! results. For example, the product of balance and exchange rate is still
//! balance, which of another asset, so the result should be `FixDec64<2>`
//! too, but not `FixDec64<2+6>`. Another example, you want to get the
//! exchange rate `FixDec64<6>` by dividing two balance `FixDec64<2>`.
//!
//! # Conversion
//!
//! Meanwhile the conversion can be made explicitly.
//!
//! Different types are converted into each other by `Into` and `TryInto`
//! trait. Use `Into` to convert from less-bit-type to more-bit-type, and
//! use `TryInto` for the opposite direction because it may overflow.
//! The conversion keeps the precision.
//!
//! Different precisions of same type are converted into each other by
//! `rescale()` function.
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

mod fixdec16;
mod fixdec32;
mod fixdec64;
mod fixdec128;

mod fpdec_16;

// fpdec_16/32/64/128
//   -> define_both_fpdecs
//        -+-> define_calculations
//         |-> define_static_prec_fpdec -> define_common
//         |-> define_oob_prec_fpdec    -> define_common
mod define_calculations;
mod define_common;
mod define_static_prec_fpdec;
mod define_oob_prec_fpdec;
mod define_both_fpdecs;
mod define_macro;
mod utils;

pub use crate::fixdec16::DIGITS as FIXDEC16_DIGITS;
pub use crate::fixdec32::DIGITS as FIXDEC32_DIGITS;
pub use crate::fixdec64::DIGITS as FixDec64_DIGITS;
pub use crate::fixdec128::DIGITS as FIXDEC128_DIGITS;

pub use crate::fixdec16::FixDec16;
pub use crate::fixdec32::FixDec32;
pub use crate::fixdec64::FixDec64;
pub use crate::fixdec128::FixDec128;

pub use crate::fpdec_16::{StaticPrecFpdec16, OobPrecFpdec16};

pub use crate::utils::OobFmt;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rounding {
    Round,
    Floor,
    Ceil,
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
