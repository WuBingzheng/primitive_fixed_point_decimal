use crate::ParseError;
use int_div_cum_error::{
    PrimSignedInt,
    Rounding,
    checked_divide,
    checked_divide_with_rounding,
};
use num_traits::Num;

pub trait FpdecInner: Sized + PrimSignedInt {
    fn get_exp(i: usize) -> Option<Self>;

    fn calc_mul_div(self,
        b: Self,
        c: Self,
        rounding: Rounding,
        cum_error: Option<&mut Self>,
    ) -> Option<Self>;

    fn checked_mul_ext(
        self,
        rhs: Self,
        diff_precision: i32, // = P + Q - R
        rounding: Rounding,
        cum_error: Option<&mut Self>,
    ) -> Option<Self> {
        if diff_precision > 0 {
            // self * rhs / diff_exp
            let exp = Self::get_exp(diff_precision as usize)?;
            self.calc_mul_div(rhs, exp, rounding, cum_error)

        } else if diff_precision < 0 {
            // self * rhs * diff_exp
            let exp = Self::get_exp(-diff_precision as usize)?;
            self.checked_mul(&rhs)?.checked_mul(&exp)

        } else {
            self.checked_mul(&rhs)
        }
    }

    fn checked_div_ext(
        self,
        rhs: Self,
        diff_precision: i32, // = P - Q - R
        rounding: Rounding,
        cum_error: Option<&mut Self>,
    ) -> Option<Self> {
        if diff_precision > 0 {
            // self / rhs / diff_exp
            let exp = Self::get_exp(diff_precision as usize)?;
            let q = checked_divide(self, rhs, rounding, None)?;
            checked_divide(q, exp, rounding, cum_error)

        } else if diff_precision < 0 {
            // self * diff_exp / rhs
            let exp = Self::get_exp(-diff_precision as usize)?;
            self.calc_mul_div(exp, rhs, rounding, cum_error)

        } else {
            self.checked_mul(&rhs)
        }
    }

    fn shrink_with_rounding(
        self,
        diff_precision: i32, // = src - dst
        rounding: Rounding,
    ) -> Self {
        if diff_precision <= 0 {
            return self;
        }

        match Self::get_exp(diff_precision as usize) {
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

    fn try_from_str(s: &str, precision: i32) -> Result<Self, ParseError>
        where Self: Num,
              ParseError: From<<Self as Num>::FromStrRadixErr>
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
            if frac_len as i32 > precision {
                return Err(ParseError::Precision);
            }

            // here precision > 0
            let precision = precision as usize;

            let mut frac_num = Self::from_str_radix(frac_str, 10)?;

            if frac_len < precision {
                let diff_exp = Self::get_exp(precision - frac_len)
                    .ok_or(ParseError::Overflow)?;
                frac_num = frac_num.checked_mul(&diff_exp)
                    .ok_or(ParseError::Overflow)?;
            }

            (int_str, frac_num)
        } else {
            (s, Self::ZERO)
        };

        // integer part
        let inner = if precision > 0 {
            match Self::get_exp(precision as usize) {
                Some(exp) => {
                    Self::from_str_radix(int_str, 10)?
                        .checked_mul(&exp)
                        .ok_or(ParseError::Overflow)?
                        .checked_add(&frac_num)
                        .ok_or(ParseError::Overflow)?
                }
                None => {
                    if int_str != "0" {
                        return Err(ParseError::Overflow);
                    }
                    frac_num
                }
            }
        } else {
            if s.len() <= -precision as usize {
                return Err(ParseError::Precision);
            }
            let end = s.len() - (-precision) as usize;
            if *&int_str[end..].chars().all(|ch| ch == '0') {
                return Err(ParseError::Precision);
            }

            Self::from_str_radix(&int_str[..end], 10)?
        };

        if is_neg { Ok(-inner) } else { Ok(inner) }
    }

    fn display_fmt(self, precision: i32, f: &mut std::fmt::Formatter)
        -> Result<(), std::fmt::Error> 
        where Self: std::fmt::Display
    {
        if self.is_zero() {
            return write!(f, "0");
        }
        if precision == 0 {
            return write!(f, "{}", self);
        }
        if precision < 0 {
            return write!(f, "{}{:0>width$}", self, 0, width=(-precision) as usize);
        }

        // precision > 0
        let precision = precision as usize;

        fn strip_zeros<I>(mut n: I) -> (I, usize)
            where I: PrimSignedInt + Sized
        {
            let mut zeros = 0;
            let ten = I::from(10).unwrap();
            while (n % ten).is_zero() { // TODO optimize?
                n = n / ten;
                zeros += 1;
            }
            (n, zeros)
        }

        match Self::get_exp(precision) {
            Some(exp) => {
                let i = self / exp;
                let mut frac = self % exp;
                if frac.is_zero() {
                    write!(f, "{}", i)
                } else {
                    if frac.is_negative() {
                        frac = -frac;
                    }
                    let (frac, zeros) = strip_zeros(frac);
                    write!(f, "{}.{:0>width$}", i, frac, width=precision-zeros)
                }
            }
            None => {
                if !self.is_negative() {
                    let (n, zeros) = strip_zeros(self);
                    write!(f, "0.{:0>width$}", n, width=precision-zeros)
                } else if self != Self::MIN {
                    let (n, zeros) = strip_zeros(-self);
                    write!(f, "-0.{:0>width$}", n, width=precision-zeros)
                } else {
                    let ten = Self::from(10).unwrap();
                    let front = self / ten;
                    let last = self % ten;
                    write!(f, "-0.{:0>width$}{}", -front, -last, width=precision-1)
                }
            }
        }
    }

    fn checked_from_int(self, precision: i32) -> Result<Self, ParseError> {
        if precision > 0 {
            let exp = Self::get_exp(precision as usize)
                .ok_or(ParseError::Overflow)?;
            self.checked_mul(&exp)
                .ok_or(ParseError::Overflow)
        } else if precision < 0{
            let exp = Self::get_exp(-precision as usize)
                .ok_or(ParseError::Precision)?;
            if !(self % exp).is_zero() {
                return Err(ParseError::Precision);
            }
            Ok(self / exp)
        } else {
            Ok(self)
        }
    }
}
