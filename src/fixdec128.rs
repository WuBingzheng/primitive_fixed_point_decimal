use core::str::FromStr;
use std::fmt;
use std::num::{ParseIntError, IntErrorKind};
use std::ops::{Neg, Add, Sub, AddAssign, SubAssign};

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct FixDec128<const S: u32> {
    inner: i128,
}

impl<const S: u32> FixDec128<S> {
    const EXP: i128 = 10_i128.pow(S);

    pub const ZERO: FixDec128<S> = FixDec128 { inner: 0 };
    pub const ONE: FixDec128<S> = FixDec128 { inner: FixDec128::<S>::EXP };
    pub const MAX: FixDec128<S> = FixDec128 { inner: i128::MAX };
    pub const MIN: FixDec128<S> = FixDec128 { inner: i128::MIN };

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
    /// The right operand must have the same scale with self. So you can not add `FixDec128<4>`
    /// by `FixDec128<5>`.
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
    /// The right operand must have the same scale with self. So you can not subtract
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
    /// The right operand could have different scale, while the result must has the same scale.
    /// You can multiple `FixDec128<4>` by `FixDec128<2>`, and still get a `FixDec128<4>`.
    ///
    /// # Examples
    /// 
    /// ```
    /// use primitive_fixed_point_decimal::FixDec128;
    /// let left = FixDec128::<4>::from(3);
    /// let right = FixDec128::<2>::from(4); // different scale
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
    /// The right operand could have different scale, while the result must has the same scale.
    /// You can divide `FixDec128<4>` by `FixDec128<2>`, and still get a `FixDec128<4>`.
    ///
    /// # Examples
    /// 
    /// ```
    /// use std::str::FromStr;
    /// use primitive_fixed_point_decimal::FixDec128;
    /// let left = FixDec128::<4>::from(3);
    /// let right = FixDec128::<2>::from_str("0.5").unwrap(); // different scale
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
        // `const fn` does not support `Option::map` by now
        if let Some(inner) = opt {
            Some(FixDec128{ inner })
        } else {
            None
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

impl<const S: u32> fmt::Display for FixDec128<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let intg = self.inner / FixDec128::<S>::EXP;
        let frac = self.inner % FixDec128::<S>::EXP;
        write!(f, "{}.{:0width$}", intg, frac, width=S as usize)
    }
}

impl<const S: u32> fmt::Debug for FixDec128<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({},{})",  self.inner, S)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Empty,
    InvalidDigit,
    PosOverflow,
    NegOverflow,
    Zero,
    TooBigPrecision,
}

impl From<ParseIntError> for ParseError {
    fn from(pi_err: ParseIntError) -> Self {
        match pi_err.kind() {
            IntErrorKind::Empty => ParseError::Empty,
            IntErrorKind::InvalidDigit => ParseError::InvalidDigit,
            IntErrorKind::PosOverflow => ParseError::PosOverflow,
            IntErrorKind::NegOverflow => ParseError::NegOverflow,
            IntErrorKind::Zero => ParseError::Zero,
            &_ => todo!(),
        }
    }
}

impl<const S: u32> FromStr for FixDec128<S> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (intg, frac) = if let Some((intg, frac)) = s.split_once('.') {
            const ALL_EXPS: [u128; 21] = [1, 10, 100,
                    1_000, 10_000, 100_000,
                    1_000_000, 10_000_000, 100_000_000,
                    1_000_000_000, 10_000_000_000, 100_000_000_000,
                    1_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000,
                    1_000_000_000_000_000, 10_000_000_000_000_000, 100_000_000_000_000_000,
                    1_000_000_000_000_000_000, 10_000_000_000_000_000_000, 100_000_000_000_000_000_000,
            ];

            let prec = frac.len();
            if prec > S as usize {
                return Err(ParseError::TooBigPrecision);
            }
            (i128::from_str(intg)?, u128::from_str(frac)? * ALL_EXPS[S as usize - prec])
        } else {
            (i128::from_str(s)?, 0)
        };

        let Some(inner) = intg.checked_mul(FixDec128::<S>::EXP) else {
            if intg > 0 {
                return Err(ParseError::PosOverflow);
            } else {
                return Err(ParseError::NegOverflow);
            }
        };

        Ok(FixDec128 { inner: inner + frac as i128 })
    }
}

impl<const S: u32> From<i8> for FixDec128<S> {
    fn from(i: i8) -> Self {
        FixDec128 { inner: i as i128 * FixDec128::<S>::EXP }
    }
}

impl<const S: u32> From<i16> for FixDec128<S> {
    fn from(i: i16) -> Self {
        FixDec128 { inner: i as i128 * FixDec128::<S>::EXP }
    }
}

impl<const S: u32> From<i32> for FixDec128<S> {
    fn from(i: i32) -> Self {
        FixDec128 { inner: i as i128 * FixDec128::<S>::EXP }
    }
}

impl<const S: u32> From<i64> for FixDec128<S> {
    fn from(i: i64) -> Self {
        FixDec128 { inner: i as i128 * FixDec128::<S>::EXP }
    }
}

impl<const S: u32> From<FixDec128<S>> for f64 {
    fn from(fixdec: FixDec128<S>) -> f64 {
        fixdec.inner as f64 / FixDec128::<S>::EXP as f64
    }
}

impl<const S: u32> Neg for FixDec128<S> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        FixDec128 { inner: -self.inner }
    }
}

impl<const S: u32> Add for FixDec128<S> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        FixDec128 { inner: self.inner + rhs.inner }
    }
}

impl<const S: u32> Sub for FixDec128<S> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        FixDec128 { inner: self.inner - rhs.inner }
    }
}

impl<const S: u32> AddAssign for FixDec128<S> {
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner;
    }
}

impl<const S: u32> SubAssign for FixDec128<S> {
    fn sub_assign(&mut self, rhs: Self) {
        self.inner -= rhs.inner;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul() {
        let fd1 = FixDec128::<4>::from(123);
        let fd2 = FixDec128::<4>::from(2);
        let fd3 = FixDec128::<4>::from(246);
        assert_eq!(fd1.checked_mul(fd2), Some(fd3));
    }
}
