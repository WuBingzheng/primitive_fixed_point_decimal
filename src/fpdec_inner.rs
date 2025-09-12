use crate::ParseError;

use core::ops::{AddAssign, SubAssign};
use core::{fmt, num::ParseIntError};

use int_div_cum_error::{CumErr, DivCumErr, Rounding};
use num_traits::{
    identities::{ConstOne, ConstZero},
    int::PrimInt,
    ops::wrapping::WrappingAdd,
    Num,
};

/// The trait for underlying representation.
///
/// Normal users don't need to use this trait.
pub trait FpdecInner:
    PrimInt + ConstOne + ConstZero + AddAssign + SubAssign + WrappingAdd + DivCumErr
{
    const MAX: Self;
    const MIN: Self;
    const TEN: Self;
    const MAX_POWERS: Self;
    const DIGITS: u32;
    const NEG_MIN_STR: &'static str;

    /// Return 10 to the power of `i`.
    fn get_exp(i: usize) -> Option<Self>;

    /// Calculate `self * b / c`.
    fn calc_mul_div(
        self,
        b: Self,
        c: Self,
        rounding: Rounding,
        cum_error: Option<&mut CumErr<Self>>,
    ) -> Option<Self>;

    // works only when: diff_scale in range [-Self::DIGITS, Self::DIGITS]
    fn checked_mul_ext(
        self,
        rhs: Self,
        diff_scale: i32, // = scale (self + rhs - result)
        rounding: Rounding,
        cum_error: Option<&mut CumErr<Self>>,
    ) -> Option<Self> {
        if diff_scale > 0 {
            // self * rhs / diff_exp

            // If diff_scale is in range [Self::DIGITS+1, Self::DIGITS*2], we
            // could do division twice (with exp[DIGITS] and exp[diff_scale-DIGITS])
            // to avoid returning `None` directly, but that's not enough.
            // Because `MAX * MAX / exp[DIGITS]` still overflows. For
            // simplicity's sake, we do not handle this case which is rare.
            let exp = Self::get_exp(diff_scale as usize)?;
            self.calc_mul_div(rhs, exp, rounding, cum_error)
        } else if diff_scale < 0 {
            // self * rhs * diff_exp
            let exp = Self::get_exp(-diff_scale as usize)?;
            self.checked_mul(&rhs)?.checked_mul(&exp)
        } else {
            self.checked_mul(&rhs)
        }
    }

    // works only when: diff_scale in range [-Self::DIGITS, Self::DIGITS]
    fn checked_div_ext(
        self,
        rhs: Self,
        diff_scale: i32, // = scale (self - rhs - result)
        rounding: Rounding,
        cum_error: Option<&mut CumErr<Self>>,
    ) -> Option<Self> {
        if diff_scale > 0 {
            // self / rhs / diff_exp
            let exp = Self::get_exp(diff_scale as usize)?;
            let q = self.checked_div_with_rounding(rhs, rounding)?;
            q.checked_div_with_opt_cum_err(exp, rounding, cum_error)
        } else if diff_scale < 0 {
            // self * diff_exp / rhs

            // If diff_scale is in range [-Self::DIGITS*2, -Self::DIGITS-1], we
            // could do multiplication twice (with exp[DIGITS] and exp[-diff_scale-DIGITS])
            // to avoid returning `None` directly. But keep same with
            // `checked_mul()`, we do not handle this case which is rare.
            let exp = Self::get_exp(-diff_scale as usize)?;
            self.calc_mul_div(exp, rhs, rounding, cum_error)
        } else {
            self.checked_div_with_opt_cum_err(rhs, rounding, cum_error)
        }
    }

    fn round_diff_with_rounding(
        self,
        diff_scale: i32, // = src - dst
        rounding: Rounding,
    ) -> Self {
        if diff_scale <= 0 {
            return self;
        }

        match Self::get_exp(diff_scale as usize) {
            None => Self::ZERO,
            Some(exp) => {
                // self / exp * exp
                self.checked_div_with_rounding(exp, rounding).unwrap() * exp
            }
        }
    }

    // INTERNAL
    // The FpdecInner works for both signed and unsigned types.
    // Sometimes we need to calculate negative value of signed type,
    // for example calculating the aboslute value for a negative
    // value. But unsigned type does not support negative operation.
    // So we use Two's Complement to calculate negative for signed
    // type. The caller must ensure that the input is signed type.
    fn calc_negative(self) -> Self {
        (!self).wrapping_add(&Self::ONE)
    }

    // INTERNAL
    // Parse an string as negative.
    // We try to parse it as positive first. If fail for overflow,
    // then it maybe the MIN value.
    fn parse_int_as_negative(s: &str) -> Result<Self, ParseIntError>
    where
        Self: Num<FromStrRadixErr = ParseIntError>,
    {
        match Self::from_str_radix(s, 10) {
            Ok(num) => Ok(num.calc_negative()),
            Err(err) => {
                if s.trim_start_matches('0') == Self::NEG_MIN_STR {
                    Ok(Self::MIN)
                } else {
                    Err(err)
                }
            }
        }
    }

    fn try_from_str(s: &str, scale: i32) -> Result<Self, ParseError>
    where
        Self: Num<FromStrRadixErr = ParseIntError>,
    {
        let (num, raw_scale) = Self::try_from_str_only(s)?;
        if num.is_zero() {
            Ok(num)
        } else if raw_scale == scale {
            Ok(num)
        } else if raw_scale > scale {
            Err(ParseError::Precision)
        } else {
            Self::get_exp((scale - raw_scale) as usize)
                .ok_or(ParseError::Precision)?
                .checked_mul(&num)
                .ok_or(ParseError::Overflow)
        }
    }

    // Guess and return the scale by the input string.
    fn try_from_str_only(s: &str) -> Result<(Self, i32), ParseError>
    where
        Self: Num<FromStrRadixErr = ParseIntError>,
    {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        if let Some((int_str, frac_str)) = s.split_once('.') {
            let int_num = Self::from_str_radix(int_str, 10)?;

            let frac_num = if s.as_bytes()[0] == b'-' {
                Self::parse_int_as_negative(frac_str)?
            } else {
                Self::from_str_radix(frac_str, 10)?
            };

            let inner = if int_num.is_zero() {
                // only fraction part
                frac_num
            } else {
                // exp * integer + fraction
                Self::get_exp(frac_str.len())
                    .ok_or(ParseError::Precision)?
                    .checked_mul(&int_num)
                    .ok_or(ParseError::Overflow)?
                    .checked_add(&frac_num)
                    .ok_or(ParseError::Overflow)?
            };
            Ok((inner, frac_str.len() as i32))
        } else {
            // only integer part
            if s == "0" || s == "-0" || s == "+0" {
                return Ok((Self::ZERO, 0));
            }
            let new_int_str = s.trim_end_matches('0');
            let diff = s.len() - new_int_str.len();
            Ok((Self::from_str_radix(new_int_str, 10)?, -(diff as i32)))
        }
    }

    fn display_fmt(self, scale: i32, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    where
        Self: fmt::Display,
    {
        if self.is_zero() {
            return write!(f, "0");
        }
        if scale == 0 {
            return write!(f, "{}", self);
        }
        if scale < 0 {
            return write!(f, "{}{:0>width$}", self, 0, width = (-scale) as usize);
        }

        // scale > 0
        let scale = scale as usize;

        fn abs_strip_zeros<I>(mut n: I) -> (I, usize)
        where
            I: FpdecInner,
        {
            if n < I::ZERO {
                n = n.calc_negative();
            }

            let mut zeros = 0;
            while (n % I::TEN).is_zero() {
                // TODO optimize?
                n = n / I::TEN;
                zeros += 1;
            }
            (n, zeros)
        }

        match Self::get_exp(scale) {
            Some(exp) => {
                let i = self / exp;
                let frac = self % exp;
                if frac.is_zero() {
                    write!(f, "{}", i)
                } else {
                    let (frac, zeros) = abs_strip_zeros(frac);
                    if i.is_zero() && (self ^ exp) < Self::ZERO {
                        write!(f, "-0.{:0>width$}", frac, width = scale - zeros)
                    } else {
                        write!(f, "{}.{:0>width$}", i, frac, width = scale - zeros)
                    }
                }
            }
            None => {
                if self >= Self::ZERO {
                    let (n, zeros) = abs_strip_zeros(self);
                    write!(f, "0.{:0>width$}", n, width = scale - zeros)
                } else if self != Self::MIN {
                    let (n, zeros) = abs_strip_zeros(self);
                    write!(f, "-0.{:0>width$}", n, width = scale - zeros)
                } else {
                    let front = (self / Self::TEN).calc_negative();
                    let last = (self % Self::TEN).calc_negative();
                    write!(f, "-0.{:0>width$}{}", front, last, width = scale - 1)
                }
            }
        }
    }

    fn checked_from_int(self, scale: i32) -> Result<Self, ParseError> {
        if scale > 0 {
            let exp = Self::get_exp(scale as usize).ok_or(ParseError::Overflow)?;
            self.checked_mul(&exp).ok_or(ParseError::Overflow)
        } else if scale < 0 {
            let exp = Self::get_exp(-scale as usize).ok_or(ParseError::Precision)?;
            if !(self % exp).is_zero() {
                return Err(ParseError::Precision);
            }
            Ok(self / exp)
        } else {
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::fmt;

    struct TestFmt<I> {
        n: I,
        scale: i32,
    }
    impl<I: FpdecInner + fmt::Display> fmt::Display for TestFmt<I> {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            self.n.display_fmt(self.scale, f)
        }
    }
    fn do_test_format(s: &str, scale: i32, n: i8) {
        assert_eq!(i8::try_from_str(s, scale), Ok(n));

        //println!("test: {s} {scale} {n}");
        let (n1, scale1) = i8::try_from_str_only(s).unwrap();
        let n2 = n1 * 10_i8.pow((scale - scale1) as u32);
        assert_eq!(n2, n);

        let ts = TestFmt { n, scale };
        assert_eq!(std::format!("{}", &ts), s);
    }

    fn do_test_format_num_only<I>(n: I)
    where
        I: FpdecInner + fmt::Display + fmt::Debug + Num<FromStrRadixErr = ParseIntError>,
    {
        for scale in -100..100 {
            let ts = TestFmt { n, scale };
            let out = std::format!("{}", ts);

            //println!("scale:{scale}, n:{n}, out:{out}");
            assert_eq!(I::try_from_str(&out, scale), Ok(n));
        }
    }

    #[test]
    fn test_format() {
        // empty
        assert_eq!(i8::try_from_str("", 2), Err(ParseError::Empty));

        // zero
        assert_eq!(i8::try_from_str("0", 2), Ok(0));
        assert_eq!(i8::try_from_str("0.0", 2), Ok(0));
        assert_eq!(i8::try_from_str("-0", 2), Ok(0));
        assert_eq!(i8::try_from_str("-0.0", 2), Ok(0));
        assert_eq!(i8::try_from_str("+0", 2), Ok(0));
        assert_eq!(i8::try_from_str("+0.0", 2), Ok(0));

        // positive
        do_test_format("12300", -2, 123);
        do_test_format("1230", -1, 123);
        do_test_format("123", 0, 123);
        do_test_format("12.3", 1, 123);
        do_test_format("1.23", 2, 123);
        do_test_format("0.123", 3, 123);
        do_test_format("0.0123", 4, 123);
        do_test_format("0.00123", 5, 123);
        do_test_format("0.000123", 6, 123);

        do_test_format("12000", -2, 120);
        do_test_format("1200", -1, 120);
        do_test_format("120", 0, 120);
        do_test_format("12", 1, 120);
        do_test_format("1.2", 2, 120);
        do_test_format("0.12", 3, 120);
        do_test_format("0.012", 4, 120);
        do_test_format("0.0012", 5, 120);
        do_test_format("0.00012", 6, 120);

        // negative with i8::MIN
        do_test_format("-12800", -2, -128);
        do_test_format("-1280", -1, -128);
        do_test_format("-128", 0, -128);
        do_test_format("-12.8", 1, -128);
        do_test_format("-1.28", 2, -128);
        do_test_format("-0.128", 3, -128);
        do_test_format("-0.0128", 4, -128);
        do_test_format("-0.00128", 5, -128);
        do_test_format("-0.000128", 6, -128);
    }
    #[test]
    fn test_format_num_only() {
        do_test_format_num_only(0);
        // do_test_format_num_only(1_u8);
        // do_test_format_num_only(12_u8);
        // do_test_format_num_only(123_u8);
        // do_test_format_num_only(255_u8);
        do_test_format_num_only(1_i8);
        do_test_format_num_only(12_i8);
        do_test_format_num_only(123_i8);
        do_test_format_num_only(-1_i8);
        do_test_format_num_only(-12_i8);
        do_test_format_num_only(-123_i8);
        do_test_format_num_only(-128_i8);

        do_test_format_num_only(1_i128);
        do_test_format_num_only(12_i128);
        do_test_format_num_only(123_i128);
        do_test_format_num_only(-1_i128);
        do_test_format_num_only(-12_i128);
        do_test_format_num_only(-123_i128);

        do_test_format_num_only(i32::MAX);
        do_test_format_num_only(i32::MIN);
        do_test_format_num_only(i64::MAX);
        do_test_format_num_only(i64::MIN);
        do_test_format_num_only(i128::MAX);
        do_test_format_num_only(i128::MIN);
        do_test_format_num_only(i32::MAX / 2);
        do_test_format_num_only(i32::MIN / 2);
        do_test_format_num_only(i64::MAX / 2);
        do_test_format_num_only(i64::MIN / 2);
        do_test_format_num_only(i128::MAX / 2);
        do_test_format_num_only(i128::MIN / 2);
    }
}
