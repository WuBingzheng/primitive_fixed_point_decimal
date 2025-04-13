//use std::str::FromStr;
use int_div_cum_error::{
    PrimSignedInt,
    Rounding,
    checked_divide,
    checked_divide_with_rounding,
    checked_divide_with_cum_error,
};
use num_traits::Num;

use crate::ParseError;

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
        _rounding: Rounding,
    ) -> Self {
        if diff_precision <= 0 {
            return self;
        }

        match Self::get_exp(diff_precision as usize) {
            None => Self::ZERO,
            Some(exp) => {
                // self / exp * exp
                let ret = self / exp * exp;
                ret
                /*
                let _remain = self - ret;
                let carry = match rounding {
                    Rounding::Floor => 0,
                    Rounding::Ceil => if remain == 0 { 0 } else { exp },
                    Rounding::Round => if remain * 2 < exp { 0 } else { exp },
                };
                ret + carry
                */
            }
        }
    }

    fn try_from_str(s: &str, precision: i32) -> Result<Self, ParseError>
        where Self: Num
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

            //let mut frac_num = Self::from_str_radix(frac_str, 10)?; // TODO use ?
            let Ok(mut frac_num) = Self::from_str_radix(frac_str, 10) else {panic!()};

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
                    let Ok(x) = Self::from_str_radix(int_str, 10) else {panic!()};
                    x
                    //Self::from_str_radix(int_str, 10)?
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

            //Self::from_str_radix(&int_str[..end], 10)?
            let Ok(x) = Self::from_str_radix(&int_str[..end], 10) else {panic!()};
            x
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

        todo!();

        /*
        // precision > 0
        let precision = precision as usize;

        if precision <= $digits {
            let exp = ALL_EXPS[precision];
            let i = a / exp;
            let mut frac = a % exp;
            if frac == 0 {
                write!(f, "{}", i)
            } else {
                if frac < 0 {
                    frac = -frac;
                }
                let mut zeros = 0;
                while frac % 10 == 0 { // TODO optimize
                    frac /= 10;
                    zeros += 1;
                }
                write!(f, "{}.{:0>width$}", i, frac, width=precision-zeros)
            }
        } else if a >= 0 {
            write!(f, "0.{:0>width$}", a, width=precision)
        } else {
            write!(f, "-0.{:0>width$}", a.unsigned_abs(), width=precision)
        }
        */
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

macro_rules! calc_mul_div_higher {
    (
        $a:expr, $b:expr, $c:expr,
        $rounding:expr, $cum_error:expr,
        $origin_type:ty, $higher_type:ty
    ) => {
        {
            match $cum_error {
                Some(cum_error) => {
                    let mut higher_cum_error = *cum_error as $higher_type;
                    let q = checked_divide_with_cum_error(
                        $a as $higher_type * $b as $higher_type,
                        $c as $higher_type,
                        $rounding,
                        &mut higher_cum_error)?;

                    *cum_error = higher_cum_error as $origin_type;
                    <$origin_type>::try_from(q).ok()
                }
                None => {
                    let q = checked_divide_with_rounding(
                        $a as $higher_type * $b as $higher_type,
                        $c as $higher_type,
                        $rounding)?;
                    <$origin_type>::try_from(q).ok()
                }
            }
        }
    }
}

impl FpdecInner for i16 {
    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i16; 5] = [
            1,
            10_i16.pow(1), 10_i16.pow(2), 10_i16.pow(3), 10_i16.pow(4),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding, cum_error: Option<&mut Self>)
        -> Option<Self>
    {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i16, i32)
    }
}

impl FpdecInner for i32 {
    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i32; 10] = [
            1,
            10_i32.pow(1), 10_i32.pow(2), 10_i32.pow(3), 10_i32.pow(4),
            10_i32.pow(5), 10_i32.pow(6), 10_i32.pow(7), 10_i32.pow(8),
            10_i32.pow(9)
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding, cum_error: Option<&mut Self>)
        -> Option<Self>
    {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i32, i64)
    }
}

impl FpdecInner for i64 {
    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i64; 10] = [
            1,
            10_i64.pow(1), 10_i64.pow(2), 10_i64.pow(3), 10_i64.pow(4),
            10_i64.pow(5), 10_i64.pow(6), 10_i64.pow(7), 10_i64.pow(8),
            10_i64.pow(9)
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding, cum_error: Option<&mut Self>)
    -> Option<Self>
    {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i64, i128)
    }
}

/*
pub const fn checked_div_ext2(
    a: I,
    b: I,
    diff_precision: i32, // = P - Q - R
    rounding: Rounding,
    cum_error: &mut $cum_err_type,
) -> Option<I> {
    if diff_precision > 0 {
        // a / b / diff_exp
        if diff_precision <= $digits {
            let mut tmp: I = 0;
            let Some(r) = rounding_div!(a, ALL_EXPS[diff_precision as usize], rounding, &mut tmp) else {
                return None;
            };
            rounding_div!(r, b, rounding, cum_error)
        } else {
            None
        }
    } else if diff_precision < 0 {
        let diff_precision = -diff_precision as usize;

        // a * diff_exp / b
    if diff_precision <= $digits {
        calc_mul_div(a, ALL_EXPS[diff_precision], b, rounding, cum_error)
    } else {
        None
    }
    } else {
        rounding_div!(a, b, rounding, cum_error)
    }
}

pub fn try_from_str(s: &str, precision: i32) -> Result<I, ParseError> {
    // sign
    let (s, is_neg) = match s.as_bytes().first() {
        None => return Err(ParseError::Empty),
        Some(b'-') => (&s[1..], true),
        Some(b'+') => (&s[1..], false),
        _ => (s, false),
    };

    if s == "0" || s == "0." {
        return Ok(0);
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

        let mut frac_num = <I>::from_str(frac_str)?;

        if frac_len < precision {
            let diff_exp = *ALL_EXPS.get(precision - frac_len)
                .ok_or(ParseError::Overflow)?;
            frac_num = frac_num.checked_mul(diff_exp)
                .ok_or(ParseError::Overflow)?;
        }

        (int_str, frac_num)
    } else {
        (s, 0)
    };

    // integer part
    let inner = if precision > $digits {
        if int_str != "0" {
            return Err(ParseError::Overflow);
        }
        frac_num
    } else if precision >= 0 {
        <I>::from_str(int_str)?
            .checked_mul(ALL_EXPS[precision as usize])
            .ok_or(ParseError::Overflow)?
            .checked_add(frac_num)
            .ok_or(ParseError::Overflow)?

    } else {
        if s.len() <= -precision as usize {
            return Err(ParseError::Precision);
        }
        let end = s.len() - (-precision) as usize;
        if *&int_str[end..].chars().all(|ch| ch == '0') {
            return Err(ParseError::Precision);
        }

        <I>::from_str(&int_str[..end])?
    };

    if is_neg { Ok(-inner) } else { Ok(inner) }
}

fn display_fmt(a: I, precision: i32, f: &mut fmt::Formatter)
    -> Result<(), fmt::Error> 
{
    if a == 0 {
        return write!(f, "0");
    }
    if precision == 0 {
        return write!(f, "{}", a);
    }
    if precision < 0 {
        return write!(f, "{}{:0>width$}", a, 0, width=(-precision) as usize);
    }

    // precision > 0
    let precision = precision as usize;

    if precision <= $digits {
        let exp = ALL_EXPS[precision];
        let i = a / exp;
        let mut frac = a % exp;
        if frac == 0 {
            write!(f, "{}", i)
        } else {
            if frac < 0 {
                frac = -frac;
            }
            let mut zeros = 0;
            while frac % 10 == 0 { // TODO optimize
                frac /= 10;
                zeros += 1;
            }
            write!(f, "{}.{:0>width$}", i, frac, width=precision-zeros)
        }
    } else if a >= 0 {
        write!(f, "0.{:0>width$}", a, width=precision)
    } else {
        write!(f, "-0.{:0>width$}", a.unsigned_abs(), width=precision)
    }
}

fn check_from_int(i2: I, precision: i32) -> Option<I> {
    if precision > $digits {
        None
    } else if precision > 0 {
        i2.checked_mul(ALL_EXPS[precision as usize])
    } else if -precision > $digits {
        Some(0)
    } else {
        i2.checked_div(ALL_EXPS[-precision as usize])
    }
}
*/
