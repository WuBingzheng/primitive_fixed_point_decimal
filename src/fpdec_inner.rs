use crate::ParseError;
use core::fmt;
use int_div_cum_error::{checked_divide, checked_divide_with_rounding, PrimSignedInt, Rounding};
use num_traits::Num;

pub trait FpdecInner: Sized + PrimSignedInt {
    const MAX: Self;
    const MIN: Self;
    const MAX_POWERS: Self;
    const DIGITS: u32;

    fn get_exp(i: usize) -> Option<Self>;

    fn calc_mul_div(
        self,
        b: Self,
        c: Self,
        rounding: Rounding,
        cum_error: Option<&mut Self>,
    ) -> Option<Self>;

    fn checked_mul_ext(
        self,
        rhs: Self,
        diff_scale: i32, // = P + Q - R
        rounding: Rounding,
        cum_error: Option<&mut Self>,
    ) -> Option<Self> {
        if diff_scale > 0 {
            // self * rhs / diff_exp
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

    fn checked_div_ext(
        self,
        rhs: Self,
        diff_scale: i32, // = P - Q - R
        rounding: Rounding,
        cum_error: Option<&mut Self>,
    ) -> Option<Self> {
        if diff_scale > 0 {
            // self / rhs / diff_exp
            let exp = Self::get_exp(diff_scale as usize)?;
            let q = checked_divide(self, rhs, rounding, None)?;
            checked_divide(q, exp, rounding, cum_error)
        } else if diff_scale < 0 {
            // self * diff_exp / rhs
            let exp = Self::get_exp(-diff_scale as usize)?;
            self.calc_mul_div(exp, rhs, rounding, cum_error)
        } else {
            self.checked_mul(&rhs)
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
                let ret = self / exp * exp;
                let remain = self - ret;
                let carry = checked_divide_with_rounding(remain, exp, rounding).unwrap();
                ret + carry * exp
            }
        }
    }

    fn try_from_str(s: &str, scale: i32) -> Result<Self, ParseError>
    where
        Self: Num,
        ParseError: From<<Self as Num>::FromStrRadixErr>,
    {
        // sign
        let (s, is_neg) = match s.as_bytes().first() {
            None => return Err(ParseError::Empty),
            Some(b'-') => (&s[1..], true),
            Some(b'+') => (&s[1..], false),
            _ => (s, false),
        };

        if s == "0" || s == "0." {
            return Ok(Self::ZERO);
        }
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        // fraction part
        let (int_str, frac_num) = if let Some((int_str, frac_str)) = s.split_once('.') {
            let frac_len = frac_str.len();
            if frac_len as i32 > scale {
                return Err(ParseError::Precision);
            }

            // here scale > 0
            let scale = scale as usize;

            let mut frac_num = Self::from_str_radix(frac_str, 10)?;

            if frac_len < scale {
                let diff_exp = Self::get_exp(scale - frac_len).ok_or(ParseError::Overflow)?;
                frac_num = frac_num
                    .checked_mul(&diff_exp)
                    .ok_or(ParseError::Overflow)?;
            }

            (int_str, frac_num)
        } else {
            (s, Self::ZERO)
        };

        // integer part
        let inner = if scale > 0 {
            match Self::get_exp(scale as usize) {
                Some(exp) => Self::from_str_radix(int_str, 10)?
                    .checked_mul(&exp)
                    .ok_or(ParseError::Overflow)?
                    .checked_add(&frac_num)
                    .ok_or(ParseError::Overflow)?,
                None => {
                    if int_str != "0" {
                        return Err(ParseError::Overflow);
                    }
                    frac_num
                }
            }
        } else {
            if s.len() <= -scale as usize {
                return Err(ParseError::Precision);
            }
            let end = s.len() - (-scale) as usize;
            if !int_str[end..].chars().all(|ch| ch == '0') {
                return Err(ParseError::Precision);
            }

            Self::from_str_radix(&int_str[..end], 10)?
        };

        if is_neg {
            Ok(-inner)
        } else {
            Ok(inner)
        }
    }

    fn try_from_str_only(s: &str) -> Result<(Self, i32), ParseError>
    where
        Self: Num,
        ParseError: From<<Self as Num>::FromStrRadixErr>,
    {
        // sign
        let (s, is_neg) = match s.as_bytes().first() {
            None => return Err(ParseError::Empty),
            Some(b'-') => (&s[1..], true),
            Some(b'+') => (&s[1..], false),
            _ => (s, false),
        };

        if s == "0" || s == "0." {
            return Ok((Self::ZERO, 0));
        }
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        let (inner, scale) = if let Some((int_str, frac_str)) = s.split_once('.') {
            let int_num = Self::from_str_radix(int_str, 10)?;
            let frac_num = Self::from_str_radix(frac_str, 10)?;

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
            (inner, frac_str.len() as i32)
        } else {
            // only integer part
            let new_int_str = s.trim_end_matches('0');
            let diff = s.len() - new_int_str.len();
            (Self::from_str_radix(new_int_str, 10)?, -(diff as i32))
        };

        let inner = if is_neg { -inner } else { inner };
        Ok((inner, scale))
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

        fn strip_zeros<I>(mut n: I) -> (I, usize)
        where
            I: PrimSignedInt + Sized,
        {
            let mut zeros = 0;
            let ten = I::from(10).unwrap();
            while (n % ten).is_zero() {
                // TODO optimize?
                n = n / ten;
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
                    let (frac, zeros) = strip_zeros(frac.abs());
                    if i.is_zero() && (self ^ exp).is_negative() {
                        write!(f, "-0.{:0>width$}", frac, width = scale - zeros)
                    } else {
                        write!(f, "{}.{:0>width$}", i, frac, width = scale - zeros)
                    }
                }
            }
            None => {
                if !self.is_negative() {
                    let (n, zeros) = strip_zeros(self);
                    write!(f, "0.{:0>width$}", n, width = scale - zeros)
                } else if self != Self::MIN {
                    let (n, zeros) = strip_zeros(-self);
                    write!(f, "-0.{:0>width$}", n, width = scale - zeros)
                } else {
                    let ten = Self::from(10).unwrap();
                    let front = self / ten;
                    let last = self % ten;
                    write!(f, "-0.{:0>width$}{}", -front, -last, width = scale - 1)
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
