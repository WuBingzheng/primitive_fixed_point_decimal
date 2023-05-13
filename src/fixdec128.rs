use core::str::FromStr;
use std::fmt;
use std::ops::{Neg, Add, Sub, AddAssign, SubAssign};
use super::{ParseError, Rounding};

/// Approximate number of significant digits in base 10.
pub const DIGITS: u32 = 38_u32;

/// Precision limit.
pub const MAX_PRECISION: u32 = 24_u32;

/// A 128-bits primitive fixed-point decimal type.
///
/// It has about 38 significant digits in base 10.
///
/// See [the module-level documentation](super) for more information.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct FixDec128<const P: u32> {
    inner: i128,
}

impl<const P: u32> FixDec128<P> {
    const EXP: i128 = 10_i128.pow(P);

    pub const ZERO: FixDec128<P> = FixDec128 { inner: 0 };
    pub const ONE: FixDec128<P> = FixDec128 { inner: FixDec128::<P>::EXP };

    /// Largest value, `(2^127 - 1) / 10^P` .
    pub const MAX: FixDec128<P> = FixDec128 { inner: i128::MAX };

    /// Smallest value, `-(2^127 / 10^P)` .
    pub const MIN: FixDec128<P> = FixDec128 { inner: i128::MIN };

    /// Smallest positive value, `10^-P` .
    pub const MIN_POSITIVE: FixDec128<P> = FixDec128 { inner: 1 };

    /// Computes the absolute value of self.
    /// 
    /// # Overflow behavior
    ///
    /// The absolute value of FixDec128::MIN cannot be represented as a FixDec128,
    /// and attempting to calculate it will cause an overflow. This means that
    /// code in debug mode will trigger a panic on this case and optimized code
    /// will return FixDec128::MIN without a panic.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use primitive_fixed_point_decimal::FixDec128;
    /// assert_eq!(FixDec128::<4>::ONE.abs(), FixDec128::<4>::ONE);
    /// assert_eq!(FixDec128::<4>::MAX.abs(), FixDec128::<4>::MAX);
    /// assert_eq!((-FixDec128::<4>::ONE).abs(), FixDec128::<4>::ONE);
    /// assert_eq!((-FixDec128::<4>::MAX).abs(), FixDec128::<4>::MAX);
    /// assert_eq!(FixDec128::<4>::ZERO.abs(), FixDec128::<4>::ZERO);
    /// ```
    pub const fn abs(self) -> Self {
        FixDec128 { inner: self.inner.abs() }
    }

    /// Checked absolute value. Computes `self.abs()`, returning `None` if `self == MIN`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use primitive_fixed_point_decimal::FixDec128;
    /// assert_eq!((-FixDec128::<4>::ONE).checked_abs(), Some(FixDec128::<4>::ONE));
    /// assert_eq!(FixDec128::<4>::MIN.checked_abs(), None);
    /// ```
    pub const fn checked_abs(self) -> Option<Self> {
        FixDec128::from_opt_inner(self.inner.checked_abs())
    }

    /// Checked integer addition. Computes `self + rhs`, returning `None` if overflow occurred.
    ///
    /// The right operand must have the same precision with self. So you can not add
    /// `FixDec128<4>` by `FixDec128<5>`.
    ///
    /// # Examples
    /// 
    /// ```
    /// use std::str::FromStr;
    /// use primitive_fixed_point_decimal::FixDec128;
    /// let left = FixDec128::<4>::from_str("123.4").unwrap();
    /// let right = FixDec128::<4>::from(5);
    /// let res = FixDec128::<4>::from_str("128.4").unwrap();
    /// assert_eq!(left.checked_add(right), Some(res));
    /// assert_eq!(FixDec128::<4>::MAX.checked_add(left), None);
    /// ```
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        FixDec128::from_opt_inner(self.inner.checked_add(rhs.inner))
    }

    /// Checked integer subtraction. Computes `self - rhs`, returning `None` if overflow occurred.
    ///
    /// The right operand must have the same precision with self. So you can not subtract
    /// `FixDec128<4>` by `FixDec128<5>`.
    ///
    /// # Examples
    /// 
    /// ```
    /// use std::str::FromStr;
    /// use primitive_fixed_point_decimal::FixDec128;
    /// let left = FixDec128::<4>::from_str("128.4").unwrap();
    /// let right = FixDec128::<4>::from(5);
    /// let res = FixDec128::<4>::from_str("123.4").unwrap();
    /// assert_eq!(left.checked_sub(right), Some(res));
    /// assert_eq!(FixDec128::<4>::MIN.checked_sub(left), None);
    /// ```
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        FixDec128::from_opt_inner(self.inner.checked_sub(rhs.inner))
    }

    /// Checked integer multiplication. Computes `self * rhs`, returning `None` if overflow occurred.
    ///
    /// The right operand could have different precision, while the result must have the same
    /// precision. So you can multiple `FixDec128<4>` by `FixDec128<2>`, and still get a
    /// `FixDec128<4>`.
    ///
    /// # Examples
    /// 
    /// ```
    /// use primitive_fixed_point_decimal::FixDec128;
    /// let left = FixDec128::<4>::from(3);
    /// let right = FixDec128::<2>::from(4); // different precision
    /// let res = FixDec128::<4>::from(12);
    /// assert_eq!(left.checked_mul(right), Some(res));
    /// assert_eq!(FixDec128::<4>::MAX.checked_mul(left), None);
    /// ```
    pub const fn checked_mul<const SR: u32>(self, rhs: FixDec128<SR>) -> Option<Self> {
        FixDec128::from_opt_inner(calc_mul_div(self.inner, rhs.inner, FixDec128::<SR>::EXP))
    }

    /// Checked integer division. Computes `self / rhs`, returning `None` if `rhs == 0` or
    /// the division results in overflow.
    ///
    /// The right operand could have different precision, while the result must has the same
    /// precision. So you can divide `FixDec128<4>` by `FixDec128<2>`, and still get a
    /// `FixDec128<4>`.
    ///
    /// # Examples
    /// 
    /// ```
    /// use std::str::FromStr;
    /// use primitive_fixed_point_decimal::FixDec128;
    /// let left = FixDec128::<4>::from(3);
    /// let right = FixDec128::<2>::from_str("0.5").unwrap(); // different precision
    /// let res = FixDec128::<4>::from(6);
    /// assert_eq!(left.checked_div(right), Some(res));
    /// assert_eq!(FixDec128::<4>::MAX.checked_div(left), None);
    /// assert_eq!(FixDec128::<4>::MAX.checked_div(FixDec128::<4>::ZERO), None);
    /// ```
    pub const fn checked_div<const SR: u32>(self, rhs: FixDec128<SR>) -> Option<Self> {
        FixDec128::from_opt_inner(calc_mul_div(self.inner, FixDec128::<SR>::EXP, rhs.inner))
    }

    pub const fn checked_mul_int(self, i: i64) -> Option<Self> {
        FixDec128::from_opt_inner(self.inner.checked_mul(i as i128))
    }

    pub const fn checked_div_int(self, i: i64) -> Option<Self> {
        FixDec128::from_opt_inner(self.inner.checked_div(i as i128))
    }

    const fn from_opt_inner(opt: Option<i128>) -> Option<Self> {
        // `const fn` does not support `Option::map()` by now
        if let Some(inner) = opt { Some(FixDec128{ inner }) } else { None }
    }

    /// TODO
    ///
    /// # Examples
    ///
    /// ```
    /// use primitive_fixed_point_decimal::FixDec128;
    /// assert_eq!(&FixDec128::<4>::from_inner(15).to_string(), "0.0015");
    /// ```
    pub const fn from_inner(inner: i128) -> Self {
        FixDec128{ inner }
    }

    /// Read decimal from string, with specified precision and Rounding::Round.
    ///
    /// Equivalent to [`FixDec128::with_precision_and_rounding`] with `round_kind=Rounding::Round`.
    pub fn with_precision(s: &str, precision: u32) -> Result<FixDec128<P>, ParseError> {
        Self::with_precision_and_rounding(s, precision, Rounding::Round)
    }

    /// Read decimal from string, with specified precision and round-up kind.
    ///
    /// # Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{FixDec128, Rounding};
    /// assert_eq!(&FixDec128::<4>::with_precision_and_rounding("42.1234", 2, Rounding::Round).unwrap().to_string(), "42.1200");
    /// ```
    pub fn with_precision_and_rounding(s: &str, precision: u32, round_kind: Rounding)
        -> Result<FixDec128<P>, ParseError> {

        debug_assert!(P <= MAX_PRECISION, "too big precision!");

        // sign part
        let (s, is_negative) = match s.as_bytes().first() {
            None => return Err(ParseError::Empty),
            Some(b'-') => (&s[1..], true),
            Some(b'+') => (&s[1..], false),
            _ => (s, false),
        };

        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        let (int_str, frac_num) = if let Some((int_str, frac_str)) = s.split_once('.') {
            // fraction part
            let mut precision = u32::min(precision, P) as usize;
            let frac_num = if precision < frac_str.len() {
                let (keep, discard) = frac_str.split_at(precision);
                parse_i128(keep)? + round_up(discard, round_kind)?
            } else {
                precision = frac_str.len();
                parse_i128(frac_str)?
            };

            (int_str, frac_num * ALL_EXPS[P as usize - precision])
        } else {
            (s, 0)
        };

        // integer part
        let int_num = parse_i128(int_str)?;

        let Some(mut inner) = int_num.checked_mul(FixDec128::<P>::EXP) else {
            return Err(ParseError::Overflow);
        };
        inner += frac_num;
        if is_negative {
            inner = -inner;
        }
        Ok(FixDec128 { inner })
    }
}

fn parse_i128(s: &str) -> Result<i128, ParseError> {
    if s.len() == 0 {
        Ok(0)
    } else if s.len() < 20 {
        u64::from_str(s).map(|n| n as i128).map_err(|_|ParseError::Invalid)
    } else {
        match u128::from_str(s) {
            Ok(n) => i128::try_from(n).map_err(|_|ParseError::Overflow),
            Err(_) => Err(ParseError::Invalid),
        }
    }
}

// return Ok(0) for drop and Ok(1) for carry
fn round_up(s: &str, kind: Rounding) -> Result<i128, ParseError> {
    if s.chars().any(|ch| ch.to_digit(10).is_none()) {
        return Err(ParseError::Invalid);
    }
    match kind {
        Rounding::Floor => Ok(0),
        Rounding::Ceil =>
            if s.trim_matches('0').is_empty() { Ok(0) } else { Ok(1) }
        Rounding::Round => {
            if let Some(first) = s.chars().next() {
                if first < '5' { Ok(0) } else { Ok(1) }
            } else {
                Ok(0)
            }
        }
    }
}

const fn calc_mul_div(a: i128, b: i128, c: i128) -> Option<i128> {
    if let Some(r) = a.checked_mul(b) {
        r.checked_div(c)
    } else {
        None // todo!("mul overflow");
    }
}

const ALL_EXPS: [i128; MAX_PRECISION as usize+1] = [1,
    10_i128.pow(1), 10_i128.pow(2), 10_i128.pow(3), 10_i128.pow(4),
    10_i128.pow(5), 10_i128.pow(6), 10_i128.pow(7), 10_i128.pow(8),
    10_i128.pow(9), 10_i128.pow(10), 10_i128.pow(11), 10_i128.pow(12),
    10_i128.pow(13), 10_i128.pow(14), 10_i128.pow(15), 10_i128.pow(16),
    10_i128.pow(17), 10_i128.pow(18), 10_i128.pow(19), 10_i128.pow(20),
    10_i128.pow(21), 10_i128.pow(22), 10_i128.pow(23), 10_i128.pow(24),
];

/// Format the decimal.
///
/// The default precision is `P`. The precision can be specified by `{:.N}`,
/// which will be ignored if larger than `P`.
///
/// # Examples:
///
/// ```
/// use primitive_fixed_point_decimal::FixDec128;
/// assert_eq!(&format!("{}", FixDec128::<4>::ONE), "1.0000");
/// assert_eq!(&format!("{:.2}", FixDec128::<4>::ONE), "1.00");
/// ```
impl<const P: u32> fmt::Display for FixDec128<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let intg = self.inner / FixDec128::<P>::EXP;
        let frac = self.inner % FixDec128::<P>::EXP;

        let (frac, precision) = if let Some(precision) = f.precision() {
            if P as usize > precision {
                (frac / ALL_EXPS[P as usize - precision], precision)
            } else {
                (frac, P as usize)
            }
        } else {
            (frac, P as usize)
        };
        write!(f, "{}.{:0width$}", intg, frac, width=precision)
    }
}

impl<const P: u32> fmt::Debug for FixDec128<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Dec({},{})", self.inner, P)
    }
}

impl<const P: u32> FromStr for FixDec128<P> {
    type Err = ParseError;

    /// Read decimal from string.
    ///
    /// Equivalent to [`FixDec128::with_precision_and_rounding`] with `precision=P`
    /// and `round_kind=Rounding::Round`.
    fn from_str(s: &str) -> Result<Self, ParseError> {
        Self::with_precision(s, P)
    }
}

impl<const P: u32> From<i8> for FixDec128<P> {
    fn from(i: i8) -> Self {
        FixDec128 { inner: i as i128 * FixDec128::<P>::EXP }
    }
}

impl<const P: u32> From<i16> for FixDec128<P> {
    fn from(i: i16) -> Self {
        FixDec128 { inner: i as i128 * FixDec128::<P>::EXP }
    }
}

impl<const P: u32> From<i32> for FixDec128<P> {
    fn from(i: i32) -> Self {
        FixDec128 { inner: i as i128 * FixDec128::<P>::EXP }
    }
}

impl<const P: u32> From<i64> for FixDec128<P> {
    fn from(i: i64) -> Self {
        FixDec128 { inner: i as i128 * FixDec128::<P>::EXP }
    }
}

impl<const P: u32> From<FixDec128<P>> for f64 {
    fn from(fixdec: FixDec128<P>) -> f64 {
        fixdec.inner as f64 / FixDec128::<P>::EXP as f64
    }
}

impl<const P: u32> Neg for FixDec128<P> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        FixDec128 { inner: -self.inner }
    }
}

impl<const P: u32> Add for FixDec128<P> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        FixDec128 { inner: self.inner + rhs.inner }
    }
}

impl<const P: u32> Sub for FixDec128<P> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        FixDec128 { inner: self.inner - rhs.inner }
    }
}

impl<const P: u32> AddAssign for FixDec128<P> {
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner;
    }
}

impl<const P: u32> SubAssign for FixDec128<P> {
    fn sub_assign(&mut self, rhs: Self) {
        self.inner -= rhs.inner;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul() {
        let fd1 = FixDec128::<4>::from_str("123.").unwrap();
        let fd2 = FixDec128::<4>::from(2);
        let fd3 = FixDec128::<4>::from(246);
        assert_eq!(fd1.checked_mul(fd2), Some(fd3));
    }
}
