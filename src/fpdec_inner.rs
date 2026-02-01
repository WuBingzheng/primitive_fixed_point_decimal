use crate::{ParseError, Rounding};

use core::{
    fmt,
    mem::MaybeUninit,
    num::{IntErrorKind, ParseIntError},
    ops::{AddAssign, SubAssign},
};

use num_traits::{
    identities::{ConstOne, ConstZero, Zero},
    int::PrimInt,
    ops::wrapping::WrappingAdd,
    AsPrimitive, Num,
};

/// The trait for underlying representation.
///
/// Normal users don't need to use this trait.
pub trait FpdecInner:
    PrimInt + ConstOne + ConstZero + AddAssign + SubAssign + WrappingAdd + Zero
{
    const MAX: Self;
    const MIN: Self;
    const TEN: Self;
    const HUNDRED: Self;
    const MAX_POWERS: Self;
    const DIGITS: u32;
    const NEG_MIN_STR: &'static str;

    /// Used by unsigned_abs() method.
    type Unsigned: FpdecInner + AsPrimitive<u8> + AsPrimitive<usize>;

    /// For signed types, this should call their unsigned_abs();
    /// for unsigned types, this should return self directly.
    fn unsigned_abs(self) -> Self::Unsigned;

    /// Return 10 to the power of `i`.
    fn get_exp(i: usize) -> Option<Self>;

    /// Calculate `self * b / c`.
    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self>;

    // works only when: diff_scale in range [-Self::DIGITS, Self::DIGITS]
    // diff_scale = scale (self + rhs - result)
    fn checked_mul_ext(self, rhs: Self, diff_scale: i32, rounding: Rounding) -> Option<Self> {
        if diff_scale > 0 {
            // self * rhs / diff_exp

            // If diff_scale is in range [Self::DIGITS+1, Self::DIGITS*2], we
            // could do division twice (with exp[DIGITS] and exp[diff_scale-DIGITS])
            // to avoid returning `None` directly, but that's not enough.
            // Because `MAX * MAX / exp[DIGITS]` still overflows. For
            // simplicity's sake, we do not handle this case which is rare.
            let exp = Self::get_exp(diff_scale as usize)?;
            self.calc_mul_div(rhs, exp, rounding)
        } else if diff_scale < 0 {
            // self * rhs * diff_exp
            let exp = Self::get_exp(-diff_scale as usize)?;
            self.checked_mul(&rhs)?.checked_mul(&exp)
        } else {
            self.checked_mul(&rhs)
        }
    }

    // works only when: diff_scale in range [-Self::DIGITS, Self::DIGITS]
    // diff_scale = scale (self + rhs - result)
    fn checked_div_ext(self, rhs: Self, diff_scale: i32, rounding: Rounding) -> Option<Self> {
        if diff_scale > 0 {
            // self / rhs / diff_exp
            let exp = Self::get_exp(diff_scale as usize)?;
            let q = self.rounding_div(rhs, rounding)?;
            q.rounding_div(exp, rounding)
        } else if diff_scale < 0 {
            // self * diff_exp / rhs

            // If diff_scale is in range [-Self::DIGITS*2, -Self::DIGITS-1], we
            // could do multiplication twice (with exp[DIGITS] and exp[-diff_scale-DIGITS])
            // to avoid returning `None` directly. But keep same with
            // `checked_mul()`, we do not handle this case which is rare.
            let exp = Self::get_exp(-diff_scale as usize)?;
            self.calc_mul_div(exp, rhs, rounding)
        } else {
            self.rounding_div(rhs, rounding)
        }
    }

    // diff_scale = scale (src - dst)
    fn round_diff_with_rounding(self, diff_scale: i32, rounding: Rounding) -> Self {
        if diff_scale <= 0 {
            return self;
        }

        match Self::get_exp(diff_scale as usize) {
            None => Self::ZERO,
            Some(exp) => {
                // self / exp * exp
                self.rounding_div(exp, rounding).unwrap() * exp
            }
        }
    }

    /// Calculate rounding division.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{FpdecInner, Rounding};
    /// assert_eq!(8.rounding_div(3, Rounding::Floor), Some(2));
    /// assert_eq!(8.rounding_div(3, Rounding::Round), Some(3));
    /// assert_eq!(8.rounding_div(3, Rounding::Ceiling), Some(3));
    /// assert_eq!(8.rounding_div(-3, Rounding::Floor), Some(-3));
    /// assert_eq!(8.rounding_div(-3, Rounding::Round), Some(-3));
    /// assert_eq!(8.rounding_div(-3, Rounding::Ceiling), Some(-2));
    /// assert_eq!((-23).rounding_div(-5, Rounding::Round), Some(5));
    /// assert_eq!((-21).rounding_div(-5, Rounding::Round), Some(4));
    /// assert_eq!(120_i8.rounding_div(121_i8, Rounding::Round), Some(1));
    /// assert_eq!(120_i8.rounding_div(-121_i8, Rounding::Round), Some(-1));
    /// ```
    fn rounding_div(self, b: Self, rounding: Rounding) -> Option<Self> {
        let q = self.checked_div(&b)?;
        let remain = self % b;
        if remain == Self::ZERO {
            return Some(q);
        }

        if (self ^ b) >= Self::ZERO {
            // unsigned types, or signed types and self and b have same sign
            match rounding {
                Rounding::Floor | Rounding::TowardsZero => q,
                Rounding::Ceiling | Rounding::AwayFromZero => q + Self::ONE,
                Rounding::Round => {
                    if Self::MIN < Self::ZERO {
                        // signed type
                        // We need to check: abs(remain) * 2 >= abs(b), but we
                        // want to avoid overflow and minimize conditional branch.
                        // So we do not use "*2" or abs().
                        // `self` and `b` have same sign. So does `remain`.
                        // If b>0, then we check: remain - (b - remain) >= 0;
                        // else, we check: remain - (b - remain) <= 0.
                        // Finally, we get:
                        if (remain - (b - remain)) ^ b >= Self::ZERO {
                            q + Self::ONE
                        } else {
                            q
                        }
                    } else {
                        // unsigned type
                        if remain >= b - remain {
                            q + Self::ONE
                        } else {
                            q
                        }
                    }
                }
            }
        } else {
            // signed types and self and b have different sign.
            match rounding {
                Rounding::Floor | Rounding::AwayFromZero => q - Self::ONE,
                Rounding::Ceiling | Rounding::TowardsZero => q,
                Rounding::Round => {
                    let r = remain.unsigned_abs();
                    if r >= b.unsigned_abs() - r {
                        q - Self::ONE
                    } else {
                        q
                    }
                }
            }
        }
        .into() // Some()
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
            Ok(num) => {
                // Return -num.
                // The FpdecInner works for both signed and unsigned types.
                // But unsigned type does not support negative operation.
                // So we use Two's Complement to calculate negative.
                Ok((!num).wrapping_add(&Self::ONE))
            }
            Err(err) => {
                if err.kind() == &IntErrorKind::PosOverflow
                    && s.trim_start_matches('0') == Self::NEG_MIN_STR
                {
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

    fn display_fmt(self, scale: i32, f: &mut fmt::Formatter) -> fmt::Result {
        // The buffer is 250 long. 50 is for the number, and 200 is for
        // the padding zeros for specified precision and big scales.
        // We panic if the string is too long.
        let mut buf: [MaybeUninit<u8>; 250] = [MaybeUninit::uninit(); 250];
        assert!(scale.abs() <= 200);

        let offset = display_num(self.unsigned_abs(), scale, f.precision(), &mut buf);

        // SAFETY: offset is updated along with buf
        let buf = unsafe {
            core::slice::from_raw_parts((&buf[offset..]).as_ptr() as *const _, buf.len() - offset)
        };

        // SAFETY: all data is valid charactor
        let s = unsafe { str::from_utf8_unchecked(buf) };

        f.pad_integral(self >= Self::ZERO, "", s)
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

// We assume the number is non-negative here. The caller should handle the sign.
fn display_num<I>(
    uns: I,
    scale: i32,
    precision: Option<usize>,
    buf: &mut [MaybeUninit<u8>],
) -> usize
where
    I: FpdecInner + AsPrimitive<u8> + AsPrimitive<usize>,
{
    if scale <= 0 {
        let mut offset = buf.len();

        // padding 0 for precision
        let precision = precision.unwrap_or(0);
        if precision != 0 {
            assert!(precision + (-scale) as usize <= 200);

            offset = pad_zeros(precision, buf);

            // point '.'
            offset -= 1;
            buf[offset].write(b'.');
        }

        // padding 0 for negative scale
        if scale < 0 && !uns.is_zero() {
            offset = pad_zeros((-scale) as usize, &mut buf[..offset]);
        }

        return dump_single(uns, &mut buf[..offset]);
    }

    // now, scale > 0
    let scale = scale as usize;

    // calculate integer and fraction parts
    let (int, frac, exp) = match I::get_exp(scale) {
        Some(exp) => (uns / exp, uns % exp, Some(exp)),
        None => (I::ZERO, uns, None),
    };

    match precision {
        // no precition set, remove fraction tailing zeros
        None => {
            if frac.is_zero() {
                return dump_single(int, buf);
            }

            // remove fraction tailing zeros
            let (frac, zeros) = {
                let mut zeros = 0;
                let mut n = frac;
                while (n % I::TEN).is_zero() {
                    n = n / I::TEN;
                    zeros += 1;
                }
                (n, zeros)
            };

            dump_decimal(int, frac, scale - zeros, buf)
        }

        // set precision = 0, do not show the '.' char
        Some(precision) if precision == 0 => match exp {
            Some(exp) => {
                if frac.saturating_add(frac) >= exp {
                    dump_single(int + I::ONE, buf)
                } else {
                    dump_single(int, buf)
                }
            }
            None => dump_single(I::ZERO, buf),
        },

        // set precision > 0
        Some(precision) => {
            if precision == scale {
                dump_decimal(int, frac, scale, buf)
            } else if precision > scale {
                assert!(precision <= 200);
                let offset = pad_zeros(precision - scale, buf);
                dump_decimal(int, frac, scale, &mut buf[..offset])
            } else {
                let frac = match I::get_exp(scale - precision) {
                    Some(exp) => frac.rounding_div(exp, Rounding::Round).unwrap(),
                    None => I::ZERO,
                };
                dump_decimal(int, frac, precision, buf)
            }
        }
    }
}

// dump: "int . frac"
fn dump_decimal<I>(int: I, frac: I, scale: usize, buf: &mut [MaybeUninit<u8>]) -> usize
where
    I: FpdecInner + AsPrimitive<u8> + AsPrimitive<usize>,
{
    let mut offset = dump_single(frac, buf);

    offset = pad_zeros(scale - (buf.len() - offset), &mut buf[..offset]);

    offset -= 1;
    buf[offset].write(b'.');

    dump_single(int, &mut buf[..offset])
}

// dump a single integer number
// This is much faster than using integers' Display.
fn dump_single<I>(n: I, buf: &mut [MaybeUninit<u8>]) -> usize
where
    I: FpdecInner + AsPrimitive<u8> + AsPrimitive<usize>,
{
    static DECIMAL_PAIRS: &[u8; 200] = b"\
        0001020304050607080910111213141516171819\
        2021222324252627282930313233343536373839\
        4041424344454647484950515253545556575859\
        6061626364656667686970717273747576777879\
        8081828384858687888990919293949596979899";

    let mut offset = buf.len();
    let mut remain = n;

    // Format per two digits from the lookup table.
    while remain >= I::TEN {
        offset -= 2;

        let pair: usize = (remain % I::HUNDRED).as_();
        remain = remain / I::HUNDRED;
        buf[offset + 0].write(DECIMAL_PAIRS[pair * 2 + 0]);
        buf[offset + 1].write(DECIMAL_PAIRS[pair * 2 + 1]);
    }

    // Format the last remaining digit, if any.
    if remain != I::ZERO || n == I::ZERO {
        offset -= 1;
        let remain: u8 = remain.as_();
        buf[offset].write(b'0' + remain);
    }

    offset
}

fn pad_zeros(n: usize, buf: &mut [MaybeUninit<u8>]) -> usize {
    let mut offset = buf.len();
    for _ in 0..n {
        offset -= 1;
        buf[offset].write(b'0');
    }
    offset
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
    fn do_test_format<I>(s: &str, scale: i32, n: I)
    where
        I: FpdecInner + fmt::Display + fmt::Debug + Num<FromStrRadixErr = ParseIntError>,
    {
        //println!("test: {s}, {scale}, {n}");
        assert_eq!(I::try_from_str(s, scale), Ok(n));

        //println!("test: {s} {scale} {n}");
        let (n1, scale1) = I::try_from_str_only(s).unwrap();
        let n2 = n1 * I::TEN.pow((scale - scale1) as u32);
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
        do_test_format("12300", -2, 123_i8);
        do_test_format("1230", -1, 123_i8);
        do_test_format("123", 0, 123_i8);
        do_test_format("12.3", 1, 123_i8);
        do_test_format("1.23", 2, 123_i8);
        do_test_format("0.123", 3, 123_i8);
        do_test_format("0.0123", 4, 123_i8);
        do_test_format("0.00123", 5, 123_i8);
        do_test_format("0.000123", 6, 123_i8);

        do_test_format("12000", -2, 120_i8);
        do_test_format("1200", -1, 120_i8);
        do_test_format("120", 0, 120_i8);
        do_test_format("12", 1, 120_i8);
        do_test_format("1.2", 2, 120_i8);
        do_test_format("0.12", 3, 120_i8);
        do_test_format("0.012", 4, 120_i8);
        do_test_format("0.0012", 5, 120_i8);
        do_test_format("0.00012", 6, 120_i8);

        // negative with i8::MIN
        do_test_format("-12800", -2, -128_i8);
        do_test_format("-1280", -1, -128_i8);
        do_test_format("-128", 0, -128_i8);
        do_test_format("-12.8", 1, -128_i8);
        do_test_format("-1.28", 2, -128_i8);
        do_test_format("-0.128", 3, -128_i8);
        do_test_format("-0.0128", 4, -128_i8);
        do_test_format("-0.00128", 5, -128_i8);
        do_test_format("-0.000128", 6, -128_i8);

        // u8
        // positive
        do_test_format("12300", -2, 123_u8);
        do_test_format("1230", -1, 123_u8);
        do_test_format("123", 0, 123_u8);
        do_test_format("12.3", 1, 123_u8);
        do_test_format("1.23", 2, 123_u8);
        do_test_format("0.123", 3, 123_u8);
        do_test_format("0.0123", 4, 123_u8);
        do_test_format("0.00123", 5, 123_u8);
        do_test_format("0.000123", 6, 123_u8);

        do_test_format("12000", -2, 120_u8);
        do_test_format("1200", -1, 120_u8);
        do_test_format("120", 0, 120_u8);
        do_test_format("12", 1, 120_u8);
        do_test_format("1.2", 2, 120_u8);
        do_test_format("0.12", 3, 120_u8);
        do_test_format("0.012", 4, 120_u8);
        do_test_format("0.0012", 5, 120_u8);
        do_test_format("0.00012", 6, 120_u8);

        do_test_format("25500", -2, 255_u8);
        do_test_format("2550", -1, 255_u8);
        do_test_format("255", 0, 255_u8);
        do_test_format("25.5", 1, 255_u8);
        do_test_format("2.55", 2, 255_u8);
        do_test_format("0.255", 3, 255_u8);
        do_test_format("0.0255", 4, 255_u8);
        do_test_format("0.00255", 5, 255_u8);
        do_test_format("0.000255", 6, 255_u8);
    }

    #[test]
    fn test_format_num_only() {
        do_test_format_num_only(0);
        do_test_format_num_only(1_u8);
        do_test_format_num_only(12_u8);
        do_test_format_num_only(123_u8);
        do_test_format_num_only(255_u8);
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

        do_test_format_num_only(1_u128);
        do_test_format_num_only(12_u128);
        do_test_format_num_only(123_u128);

        do_test_format_num_only(u32::MAX);
        do_test_format_num_only(u64::MAX);
        do_test_format_num_only(u128::MAX);
        do_test_format_num_only(u32::MAX / 2);
        do_test_format_num_only(u64::MAX / 2);
        do_test_format_num_only(u128::MAX / 2);
    }

    #[test]
    fn test_options() {
        let d = TestFmt {
            n: 12_3470,
            scale: 4,
        };
        let d2 = TestFmt {
            n: 12_5470,
            scale: 4,
        };
        let n = TestFmt {
            n: -12_3470,
            scale: 4,
        };
        let z = TestFmt { n: 0, scale: 4 };

        assert_eq!(std::format!("{}", &d), "12.347");
        assert_eq!(std::format!("{}", &n), "-12.347");
        assert_eq!(std::format!("{}", &z), "0");

        // width, fill, alignment
        assert_eq!(std::format!("{:x>10}", &d), "xxxx12.347");
        assert_eq!(std::format!("{:0>10}", &d), "000012.347");
        assert_eq!(std::format!("{:x>10}", &n), "xxx-12.347");
        assert_eq!(std::format!("{:010}", &n), "-00012.347");

        // precision
        assert_eq!(std::format!("{:.0}", &d), "12");
        assert_eq!(std::format!("{:.0}", &d2), "13");
        assert_eq!(std::format!("{:.2}", &d), "12.35");
        assert_eq!(std::format!("{:.4}", &d), "12.3470");
        assert_eq!(std::format!("{:.6}", &d), "12.347000");
        assert_eq!(std::format!("{:.0}", &n), "-12");
        assert_eq!(std::format!("{:.2}", &n), "-12.35");
        assert_eq!(std::format!("{:.4}", &n), "-12.3470");
        assert_eq!(std::format!("{:.6}", &n), "-12.347000");
        assert_eq!(std::format!("{:.0}", &z), "0");
        assert_eq!(std::format!("{:.2}", &z), "0.00");
        assert_eq!(std::format!("{:.4}", &z), "0.0000");
        assert_eq!(std::format!("{:.6}", &z), "0.000000");

        // sign
        assert_eq!(std::format!("{:+}", &d), "+12.347");
        assert_eq!(std::format!("{:+}", &n), "-12.347");
        assert_eq!(std::format!("{:+}", &z), "+0");

        // all
        assert_eq!(std::format!("{:x>+10.2}", &d), "xxxx+12.35");
        assert_eq!(std::format!("{:+010.2}", &d), "+000012.35");

        // neg-scale
        let d = TestFmt { n: 12, scale: -4 };
        assert_eq!(std::format!("{}", &d), "120000");
        assert_eq!(std::format!("{:.0}", &d), "120000");
        assert_eq!(std::format!("{:.2}", &d), "120000.00");
    }

    fn do_test_rounding_div_no_rem(a: i32, b: i32) {
        let q = a / b;
        assert_eq!(a.rounding_div(b, Rounding::Floor).unwrap(), q);
        assert_eq!(a.rounding_div(b, Rounding::Ceiling).unwrap(), q);
        assert_eq!(a.rounding_div(b, Rounding::Round).unwrap(), q);

        assert_eq!((-a).rounding_div(-b, Rounding::Floor).unwrap(), q);
        assert_eq!((-a).rounding_div(-b, Rounding::Ceiling).unwrap(), q);
        assert_eq!((-a).rounding_div(-b, Rounding::Round).unwrap(), q);

        assert_eq!(a.rounding_div(-b, Rounding::Floor).unwrap(), -q);
        assert_eq!(a.rounding_div(-b, Rounding::Ceiling).unwrap(), -q);
        assert_eq!(a.rounding_div(-b, Rounding::Round).unwrap(), -q);

        assert_eq!((-a).rounding_div(b, Rounding::Floor).unwrap(), -q);
        assert_eq!((-a).rounding_div(b, Rounding::Ceiling).unwrap(), -q);
        assert_eq!((-a).rounding_div(b, Rounding::Round).unwrap(), -q);
    }

    fn do_test_rounding_div_small(a: i32, b: i32) {
        let q = a / b;
        let one = if a ^ b > 0 { 1 } else { -1 };
        assert_eq!(a.rounding_div(b, Rounding::TowardsZero).unwrap(), q);
        assert_eq!(a.rounding_div(b, Rounding::AwayFromZero).unwrap(), q + one);
        assert_eq!(a.rounding_div(b, Rounding::Round).unwrap(), q);
    }

    fn do_test_rounding_div_small4(a: i32, b: i32) {
        let q = a / b;
        assert_eq!(a.rounding_div(b, Rounding::Floor).unwrap(), q);
        assert_eq!(a.rounding_div(b, Rounding::Ceiling).unwrap(), q + 1);
        assert_eq!(a.rounding_div(b, Rounding::Round).unwrap(), q);

        assert_eq!((-a).rounding_div(-b, Rounding::Floor).unwrap(), q);
        assert_eq!((-a).rounding_div(-b, Rounding::Ceiling).unwrap(), q + 1);
        assert_eq!((-a).rounding_div(-b, Rounding::Round).unwrap(), q);

        assert_eq!(a.rounding_div(-b, Rounding::Floor).unwrap(), -q - 1);
        assert_eq!(a.rounding_div(-b, Rounding::Ceiling).unwrap(), -q);
        assert_eq!(a.rounding_div(-b, Rounding::Round).unwrap(), -q);

        assert_eq!((-a).rounding_div(b, Rounding::Floor).unwrap(), -q - 1);
        assert_eq!((-a).rounding_div(b, Rounding::Ceiling).unwrap(), -q);
        assert_eq!((-a).rounding_div(b, Rounding::Round).unwrap(), -q);
    }

    fn do_test_rounding_div_big(a: i32, b: i32) {
        let q = a / b;
        let one = if a ^ b > 0 { 1 } else { -1 };
        assert_eq!(a.rounding_div(b, Rounding::TowardsZero).unwrap(), q);
        assert_eq!(a.rounding_div(b, Rounding::AwayFromZero).unwrap(), q + one);
        assert_eq!(a.rounding_div(b, Rounding::Round).unwrap(), q + one);
    }

    fn do_test_rounding_div_big4(a: i32, b: i32) {
        let q = a / b;
        assert_eq!(a.rounding_div(b, Rounding::Floor).unwrap(), q);
        assert_eq!(a.rounding_div(b, Rounding::Ceiling).unwrap(), q + 1);
        assert_eq!(a.rounding_div(b, Rounding::Round).unwrap(), q + 1);

        assert_eq!((-a).rounding_div(-b, Rounding::Floor).unwrap(), q);
        assert_eq!((-a).rounding_div(-b, Rounding::Ceiling).unwrap(), q + 1);
        assert_eq!((-a).rounding_div(-b, Rounding::Round).unwrap(), q + 1);

        assert_eq!(a.rounding_div(-b, Rounding::Floor).unwrap(), -q - 1);
        assert_eq!(a.rounding_div(-b, Rounding::Ceiling).unwrap(), -q);
        assert_eq!(a.rounding_div(-b, Rounding::Round).unwrap(), -q - 1);

        assert_eq!((-a).rounding_div(b, Rounding::Floor).unwrap(), -q - 1);
        assert_eq!((-a).rounding_div(b, Rounding::Ceiling).unwrap(), -q);
        assert_eq!((-a).rounding_div(b, Rounding::Round).unwrap(), -q - 1);
    }

    #[test]
    fn test_rounding_div() {
        do_test_rounding_div_no_rem(10, 5);
        do_test_rounding_div_no_rem(10, 2);
        do_test_rounding_div_no_rem(144, 12);

        do_test_rounding_div_small4(7, 3);
        do_test_rounding_div_small4(21, 5);
        do_test_rounding_div_small4(22, 5);
        do_test_rounding_div_small4(i32::MAX, i32::MAX - 1);

        do_test_rounding_div_big4(8, 3);
        do_test_rounding_div_big4(23, 5);
        do_test_rounding_div_big4(24, 5);
        do_test_rounding_div_big4(i32::MAX, 37);

        do_test_rounding_div_small(i32::MIN, 20);
        do_test_rounding_div_small(i32::MIN, -20);
        do_test_rounding_div_small(100, i32::MIN);
        do_test_rounding_div_small(-100, i32::MIN);

        do_test_rounding_div_big(i32::MIN, 37);
        do_test_rounding_div_big(i32::MIN, -37);
        do_test_rounding_div_big(i32::MIN + 100, i32::MIN);
        do_test_rounding_div_big(i32::MAX, i32::MIN);

        assert_eq!(23_u32.rounding_div(5_u32, Rounding::Round).unwrap(), 5_u32);
        assert_eq!(21_u32.rounding_div(5_u32, Rounding::Round).unwrap(), 4_u32);
        assert_eq!(101_u32.rounding_div(100_u32, Rounding::Round).unwrap(), 1);
        assert_eq!(199_u32.rounding_div(100_u32, Rounding::Round).unwrap(), 2);
        assert_eq!(149_u32.rounding_div(100_u32, Rounding::Round).unwrap(), 1);
        assert_eq!(150_u32.rounding_div(100_u32, Rounding::Round).unwrap(), 2);
    }
}
