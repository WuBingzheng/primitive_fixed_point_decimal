use crate::fpdec_inner::FpdecInner;
use crate::oob_scale_fpdec::OobScaleFpdec;
use crate::ParseError;

use core::{fmt, num::ParseIntError, ops, str::FromStr};

use int_div_cum_error::{CumErr, Rounding};
#[allow(unused_imports)]
use num_traits::float::FloatCore; // used only for `no_std`
use num_traits::{cast::FromPrimitive, Num, Signed};

/// Const-scale fixed-point decimal.
///
/// `I` is the inner integer type, could be `i8`, `i16`, `i32`, `i64`,
/// or `i128`, with 2, 4, 9, 18 and 38 significant digits respectively.
///
/// `S` is the static scale. Positive means fraction precision. Negative
/// means omitting the low-order digits of integer values.
///
/// For example, `ConstScaleFpdec<i64, 4>` means using `i64` as the underlying
/// integer, and having `4` fraction precision.
///
/// See [the module-level documentation](super) for more information.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct ConstScaleFpdec<I, const S: i32>(I);

impl<I, const S: i32> ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    crate::none_scale_common::define_none_scale_common!();

    /// The static scale.
    pub const SCALE: i32 = S;

    /// Checked multiplication.
    ///
    /// Equivalent to [`Self::checked_mul_ext`] with `Rounding::Round`.
    pub fn checked_mul<J, const S2: i32, const SR: i32>(
        self,
        rhs: ConstScaleFpdec<J, S2>,
    ) -> Option<ConstScaleFpdec<I, SR>>
    where
        J: FpdecInner,
    {
        self.checked_mul_ext(rhs, Rounding::Round, None)
    }

    /// Checked multiplication. Computes `self * rhs`, returning `None` if
    /// overflow occurred or the scale difference `S + S2 - SR` is out of range
    /// `[-Self::DIGITS, Self::DIGITS]`.
    ///
    /// The type of `rhs` can have different inner integer `J` and scale `S2`
    /// with `self`. The type of result must have the same inner integer `I`
    /// while have different scale `SR`.
    ///
    /// If the scale of the result's type `SR` is less than the sum of
    /// scales of the two multiplicands `S + S2`, then rounding operations
    /// are required and precision may be lost.
    /// You can specify the rounding type and cumulative error.
    ///
    /// See the [cumulative error section](index.html#cumulative-error)
    /// for more information and examples.
    ///
    /// # Examples
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{ConstScaleFpdec, Rounding, fpdec};
    /// type Balance = ConstScaleFpdec<i64, 2>;
    /// type FeeRate = ConstScaleFpdec<i16, 4>; // different types
    ///
    /// let balance: Balance = fpdec!(12.60);
    /// let rate: FeeRate = fpdec!(0.01);
    ///
    /// // calculate fee 3 times with same arguments, with `cum_error`.
    /// // but have different results: 0.13, 0.13 and 0.12
    /// let mut cum_error: i64 = 0;
    ///
    /// let fee: Balance = balance.checked_mul_ext(rate, Rounding::Ceiling, Some(&mut cum_error)).unwrap();
    /// assert_eq!(fee, fpdec!(0.13));
    ///
    /// let fee: Balance = balance.checked_mul_ext(rate, Rounding::Ceiling, Some(&mut cum_error)).unwrap();
    /// assert_eq!(fee, fpdec!(0.13));
    ///
    /// let fee: Balance = balance.checked_mul_ext(rate, Rounding::Ceiling, Some(&mut cum_error)).unwrap();
    /// assert_eq!(fee, fpdec!(0.12)); // here, different
    /// ```
    pub fn checked_mul_ext<J, const S2: i32, const SR: i32>(
        self,
        rhs: ConstScaleFpdec<J, S2>,
        rounding: Rounding,
        cum_error: Option<&mut CumErr<I>>,
    ) -> Option<ConstScaleFpdec<I, SR>>
    where
        J: FpdecInner,
    {
        self.0
            .checked_mul_ext(I::from(rhs.0)?, S + S2 - SR, rounding, cum_error)
            .map(ConstScaleFpdec)
    }

    /// Checked division.
    ///
    /// Equivalent to [`Self::checked_div_ext`] with `Rounding::Round`.
    pub fn checked_div<J, const S2: i32, const SR: i32>(
        self,
        rhs: ConstScaleFpdec<J, S2>,
    ) -> Option<ConstScaleFpdec<I, SR>>
    where
        J: FpdecInner,
    {
        self.checked_div_ext(rhs, Rounding::Round, None)
    }

    /// Checked division. Computes `self / rhs`, returning `None` if
    /// division by 0, or overflow occurred, or the scale difference
    /// `S - S2 - SR` is out of range `[-Self::DIGITS, Self::DIGITS]`.
    ///
    /// The type of `rhs` can have different inner integer `J` and scale `S2`
    /// with `self`. The type of result must have the same inner integer `I`
    /// while have different scale `SR`.
    ///
    /// You can specify the rounding type and cumulative error.
    /// See the [cumulative error section](index.html#cumulative-error)
    /// for more information and examples.
    ///
    /// # Examples
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{ConstScaleFpdec, Rounding, fpdec};
    /// type Balance = ConstScaleFpdec<i64, 2>;
    /// type FeeRate = ConstScaleFpdec<i16, 4>; // different types
    ///
    /// let rate: FeeRate = fpdec!(0.03);
    /// let fee: Balance = fpdec!(0.13);
    ///
    /// let balance: Balance = fee.checked_div_ext(rate, Rounding::Ceiling, None).unwrap();
    /// assert_eq!(balance, fpdec!(4.34));
    /// ```
    pub fn checked_div_ext<J, const S2: i32, const SR: i32>(
        self,
        rhs: ConstScaleFpdec<J, S2>,
        rounding: Rounding,
        cum_error: Option<&mut CumErr<I>>,
    ) -> Option<ConstScaleFpdec<I, SR>>
    where
        J: FpdecInner,
    {
        self.0
            .checked_div_ext(I::from(rhs.0)?, S - S2 - SR, rounding, cum_error)
            .map(ConstScaleFpdec)
    }

    /// Round the decimal at the specified scale.
    ///
    /// Equivalent to [`Self::round_with_rounding`] with `Rounding::Round`.
    pub fn round(self, scale: i32) -> Self {
        self.round_with_rounding(scale, Rounding::Round)
    }

    /// Round the decimal at the specified scale with rounding type.
    ///
    /// Return the original decimal if `scale >= S`.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{ConstScaleFpdec, Rounding, fpdec};
    /// type Price = ConstScaleFpdec<i64, 8>;
    ///
    /// let price: Price = fpdec!(12.12345678);
    ///
    /// assert_eq!(price.round(6), fpdec!(12.123457)); // `Rounding::Round` as default
    ///
    /// assert_eq!(price.round_with_rounding(6, Rounding::Floor), fpdec!(12.123456));
    /// ```
    pub fn round_with_rounding(self, scale: i32, rounding: Rounding) -> Self {
        Self(self.0.round_diff_with_rounding(S - scale, rounding))
    }
}

impl<I, const S: i32> ConstScaleFpdec<I, S>
where
    I: FpdecInner + Signed,
{
    crate::none_scale_common::define_none_scale_common_signed!();
}

impl<I, const S: i32> fmt::Debug for ConstScaleFpdec<I, S>
where
    I: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Fpdec({},{})", self.0, S)
    }
}

/// Format the decimal.
///
/// Examples:
///
/// ```
/// use primitive_fixed_point_decimal::{ConstScaleFpdec, ParseError};
/// type Decimal = ConstScaleFpdec<i16, 4>;
/// type BigPrec = ConstScaleFpdec<i16, 8>;
/// type NegPrec = ConstScaleFpdec<i16, -2>;
///
/// assert_eq!(format!("{}", Decimal::try_from(1.230).unwrap()), String::from("1.23"));
/// assert_eq!(format!("{}", Decimal::try_from(-1.230).unwrap()), String::from("-1.23")); // negative
/// assert_eq!(format!("{}", Decimal::try_from(-3.2768).unwrap()), String::from("-3.2768")); // i16::MIN
///
/// assert_eq!(format!("{}", BigPrec::try_from(0.00001230).unwrap()), String::from("0.0000123"));
/// assert_eq!(format!("{}", BigPrec::try_from(-0.00001230).unwrap()), String::from("-0.0000123"));
/// assert_eq!(format!("{}", BigPrec::try_from(-0.00032768).unwrap()), String::from("-0.00032768")); // i16::MIN
///
/// assert_eq!(format!("{}", NegPrec::try_from(12300).unwrap()), String::from("12300"));
/// assert_eq!(format!("{}", NegPrec::try_from(-12300).unwrap()), String::from("-12300"));
/// assert_eq!(format!("{}", NegPrec::try_from(-3276800).unwrap()), String::from("-3276800"));
/// ```
impl<I, const S: i32> fmt::Display for ConstScaleFpdec<I, S>
where
    I: FpdecInner + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.display_fmt(S, f)
    }
}

/// Read decimal from string.
///
/// This method has 2 limitations:
/// 1. Support decimal format only but not scientific notation;
/// 2. Return `ParseError::Precision` if the string has more precision than `S`.
///
/// If you want to skip these limitations, you can parse the string
/// to float number first and then convert the number to this decimal.
///
/// Examples:
///
/// ```
/// use core::str::FromStr;
/// use primitive_fixed_point_decimal::{ConstScaleFpdec, ParseError};
/// type Decimal = ConstScaleFpdec<i16, 4>;
///
/// assert_eq!(Decimal::from_str("1.23"), Decimal::try_from(1.23));
/// assert_eq!(Decimal::from_str("9999"), Err(ParseError::Overflow));
/// assert_eq!(Decimal::from_str("1.23456"), Err(ParseError::Precision));
/// ```
impl<I, const S: i32> FromStr for ConstScaleFpdec<I, S>
where
    I: FpdecInner + Num<FromStrRadixErr = ParseIntError>,
{
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, ParseError> {
        I::try_from_str(s, S).map(Self)
    }
}

impl<I, const S: i32> From<OobScaleFpdec<I>> for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    /// Convert from `OobScaleFpdec` with scale `S` to `ConstScaleFpdec`.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{ConstScaleFpdec, OobScaleFpdec, fpdec};
    /// type ConstDec = ConstScaleFpdec<i32, 6>;
    /// type OobDec = OobScaleFpdec<i32>; // the OOB scale is 6 too
    ///
    /// let od: OobDec = fpdec!(123.45, 6); // make sure that `od` has the same scale=6
    /// let sd: ConstDec = od.into();
    /// assert_eq!(sd, fpdec!(123.45));
    /// ```
    fn from(od: OobScaleFpdec<I>) -> Self {
        Self(od.mantissa())
    }
}

macro_rules! convert_from_int {
    ($from_int_type:ty) => {
        impl<I, const S: i32> TryFrom<$from_int_type> for ConstScaleFpdec<I, S>
        where
            I: FpdecInner,
        {
            type Error = ParseError;

            /// Convert from integer. Returning error if overflow occurred
            /// or lossing precision under `scale < 0`.
            ///
            /// Examples:
            ///
            /// ```
            /// use core::str::FromStr;
            /// use primitive_fixed_point_decimal::{ConstScaleFpdec, ParseError};
            /// type Decimal = ConstScaleFpdec<i32, 6>;
            /// type NegPrec = ConstScaleFpdec<i16, -6>;
            ///
            /// assert_eq!(Decimal::try_from(123).unwrap(), Decimal::from_str("123").unwrap());
            /// assert_eq!(Decimal::try_from(123_i8).unwrap(), Decimal::from_str("123").unwrap());
            /// assert_eq!(NegPrec::try_from(12000000).unwrap(), NegPrec::from_str("12000000").unwrap());
            /// assert_eq!(Decimal::try_from(9999999), Err(ParseError::Overflow));
            /// assert_eq!(NegPrec::try_from(123), Err(ParseError::Precision));
            /// ```
            fn try_from(i: $from_int_type) -> Result<Self, Self::Error> {
                if S > 0 {
                    // convert from type i to I first
                    let i2 = I::from(i).ok_or(ParseError::Overflow)?;
                    I::checked_from_int(i2, S).map(Self)
                } else {
                    // convert to fpdec inner first
                    let i2 = i.checked_from_int(S)?;
                    I::from(i2).ok_or(ParseError::Overflow).map(Self)
                }
            }
        }
    };
}
convert_from_int!(i8);
convert_from_int!(i16);
convert_from_int!(i32);
convert_from_int!(i64);
convert_from_int!(i128);

macro_rules! convert_from_float {
    ($float_type:ty, $from_fn:ident, $to_fn:ident) => {
        impl<I, const S: i32> TryFrom<$float_type> for ConstScaleFpdec<I, S>
        where
            I: FromPrimitive + FpdecInner,
        {
            type Error = ParseError;

            /// Convert from float type. Returning error if overflow occurred.
            ///
            /// Since it's hard for the float types to represent decimal fraction
            /// exactly, so this method always rounds the float number into
            /// ConstScaleFpdec.
            ///
            /// Examples:
            ///
            /// ```
            /// use core::str::FromStr;
            /// use primitive_fixed_point_decimal::{ConstScaleFpdec, ParseError};
            /// type Decimal = ConstScaleFpdec<i32, 4>;
            ///
            /// assert_eq!(Decimal::try_from(1.23).unwrap(), Decimal::from_str("1.23").unwrap());
            /// assert_eq!(Decimal::try_from(1.23456789).unwrap(), Decimal::from_str("1.2346").unwrap());
            /// ```
            fn try_from(f: $float_type) -> Result<Self, Self::Error> {
                let inner_f = if S > 0 {
                    f * 10.0.powi(S)
                } else if S < 0 {
                    f / 10.0.powi(-S)
                } else {
                    f
                };
                I::$from_fn(inner_f.round())
                    .map(Self)
                    .ok_or(ParseError::Overflow)
            }
        }

        impl<I, const S: i32> From<ConstScaleFpdec<I, S>> for $float_type
        where
            I: FpdecInner,
        {
            /// Convert into float type.
            ///
            /// Examples:
            ///
            /// ```
            /// use core::str::FromStr;
            /// use primitive_fixed_point_decimal::{ConstScaleFpdec, ParseError, fpdec};
            /// type Decimal = ConstScaleFpdec<i32, 4>;
            ///
            /// let dec: Decimal = fpdec!(1.23);
            /// let f: f32 = dec.into();
            /// assert_eq!(f, 1.23);
            ///
            /// type Decimal2 = ConstScaleFpdec<i32, -3>;
            /// let dec: Decimal2 = fpdec!(123000);
            /// let f: f32 = dec.into();
            /// assert_eq!(f, 123000.0);
            /// ```
            fn from(dec: ConstScaleFpdec<I, S>) -> Self {
                let f = dec.0.$to_fn().unwrap();
                if S > 0 {
                    f / 10.0.powi(S)
                } else if S < 0 {
                    f * 10.0.powi(-S)
                } else {
                    f
                }
            }
        }
    };
}

convert_from_float!(f32, from_f32, to_f32);
convert_from_float!(f64, from_f64, to_f64);

impl<I, const S: i32> ops::Neg for ConstScaleFpdec<I, S>
where
    I: FpdecInner + Signed,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<I, const S: i32> ops::Add for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<I, const S: i32> ops::Sub for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

/// Performs the `*` operation bewteen 2 decimals.
///
/// The right operand may have different types (both underlying integer
/// and scale), but the result inherits the left operand's type. If you
/// want specify result's scale, please use [`Self::checked_mul`] directly.
///
/// # Panics
///
/// If [`Self::checked_mul`] returns `None`.
///
/// # Examples
///
/// ```
/// use primitive_fixed_point_decimal::{ConstScaleFpdec, fpdec};
/// type Balance = ConstScaleFpdec<i64, 4>;
/// type FeeRate = ConstScaleFpdec<i16, 6>; // different types
///
/// let balance: Balance = fpdec!(12.60);
/// let rate: FeeRate = fpdec!(0.01);
///
/// assert_eq!(balance * rate, fpdec!(0.126));
/// ```
impl<I, J, const S: i32, const S2: i32> ops::Mul<ConstScaleFpdec<J, S2>> for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
    J: FpdecInner,
{
    type Output = Self;
    fn mul(self, rhs: ConstScaleFpdec<J, S2>) -> Self::Output {
        self.checked_mul(rhs)
            .expect("overflow in decimal multiplication")
    }
}

/// Performs the `*` operation with an integer.
///
/// # Panics
///
/// If [`Self::checked_mul_int`] returns `None`.
impl<I, J, const S: i32> ops::Mul<J> for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
    J: Into<I> + Num, // the `Num` to avoid conflicting implementations only
{
    type Output = Self;
    fn mul(self, rhs: J) -> Self::Output {
        self.checked_mul_int(rhs)
            .expect("overflow in decimal multiplication")
    }
}

/// Performs the `/` operation bewteen 2 decimals.
///
/// The right operand may have different types (both underlying integer
/// and scale), but the result inherits the left operand's type. If you
/// want specify result's scale, please use [`Self::checked_div`] directly.
///
/// # Panics
///
/// If [`Self::checked_div`] returns `None`.
///
/// # Examples
///
/// ```
/// use primitive_fixed_point_decimal::{ConstScaleFpdec, fpdec};
/// type Balance = ConstScaleFpdec<i64, 4>;
/// type FeeRate = ConstScaleFpdec<i16, 6>; // different types
///
/// let fee: Balance = fpdec!(0.1260);
/// let rate: FeeRate = fpdec!(0.01);
///
/// assert_eq!(fee / rate, fpdec!(12.6));
/// ```
impl<I, J, const S: i32, const S2: i32> ops::Div<ConstScaleFpdec<J, S2>> for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
    J: FpdecInner,
{
    type Output = Self;
    fn div(self, rhs: ConstScaleFpdec<J, S2>) -> Self::Output {
        self.checked_div(rhs).expect("fail in decimal division")
    }
}

/// Performs the `/` operation with an integer.
///
/// # Panics
///
/// If [`Self::checked_div_int`] returns `None`.
impl<I, J, const S: i32> ops::Div<J> for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
    J: Into<I> + Num,
{
    type Output = Self;
    fn div(self, rhs: J) -> Self::Output {
        self.checked_div_int(rhs).expect("fail in decimal division")
    }
}

impl<I, const S: i32> ops::AddAssign for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<I, const S: i32> ops::SubAssign for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl<I, J, const S: i32, const S2: i32> ops::MulAssign<ConstScaleFpdec<J, S2>>
    for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
    J: FpdecInner,
{
    fn mul_assign(&mut self, rhs: ConstScaleFpdec<J, S2>) {
        *self = *self * rhs;
    }
}

impl<I, J, const S: i32> ops::MulAssign<J> for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
    J: Into<I> + Num,
{
    fn mul_assign(&mut self, rhs: J) {
        *self = *self * rhs;
    }
}

impl<I, J, const S: i32, const S2: i32> ops::DivAssign<ConstScaleFpdec<J, S2>>
    for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
    J: FpdecInner,
{
    fn div_assign(&mut self, rhs: ConstScaleFpdec<J, S2>) {
        *self = *self / rhs;
    }
}

impl<I, J, const S: i32> ops::DivAssign<J> for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
    J: Into<I> + Num,
{
    fn div_assign(&mut self, rhs: J) {
        *self = *self / rhs;
    }
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "serde")]
impl<I, const S0: i32> Serialize for ConstScaleFpdec<I, S0>
where
    I: FpdecInner + fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // XXX how to selete dump type?
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            Into::<f64>::into(*self).serialize(serializer)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, I, const S: i32> Deserialize<'de> for ConstScaleFpdec<I, S>
where
    I: FromPrimitive + FpdecInner + Num<FromStrRadixErr = ParseIntError>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use core::marker::PhantomData;
        use core::str::FromStr;
        use serde::de::{self, Visitor};

        struct ConstScaleFpdecVistor<I, const S: i32>(PhantomData<I>);

        macro_rules! visit_num {
            ($func_name:ident, $num_type:ty) => {
                fn $func_name<E: de::Error>(self, n: $num_type) -> Result<Self::Value, E> {
                    ConstScaleFpdec::try_from(n).map_err(|_| E::custom("decimal overflow"))
                }
            };
        }

        impl<'de, I, const S: i32> Visitor<'de> for ConstScaleFpdecVistor<I, S>
        where
            I: FromPrimitive + FpdecInner + Num<FromStrRadixErr = ParseIntError>,
        {
            type Value = ConstScaleFpdec<I, S>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "decimal")
            }

            fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                ConstScaleFpdec::from_str(s).map_err(E::custom)
            }

            visit_num!(visit_f32, f32);
            visit_num!(visit_f64, f64);
            visit_num!(visit_i8, i8);
            visit_num!(visit_i16, i16);
            visit_num!(visit_i32, i32);
            visit_num!(visit_i64, i64);
            visit_num!(visit_i128, i128);
        }

        deserializer.deserialize_any(ConstScaleFpdecVistor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as primitive_fixed_point_decimal;
    use crate::fpdec;

    #[test]
    fn test_mul() {
        let two_p12: ConstScaleFpdec<i32, 12> = fpdec!(2e-12);
        let two_n12: ConstScaleFpdec<i32, -12> = fpdec!(2e+12);
        let two_p6: ConstScaleFpdec<i32, 6> = fpdec!(2e-6);
        let two_n6: ConstScaleFpdec<i32, -6> = fpdec!(2e+6);
        let two_p3: ConstScaleFpdec<i32, 3> = fpdec!(2e-3);
        let two_n3: ConstScaleFpdec<i32, -3> = fpdec!(2e+3);
        let two_0: ConstScaleFpdec<i32, 0> = fpdec!(2);

        let zero_p6 = ConstScaleFpdec::<i32, 6>::ZERO;
        let zero_n6 = ConstScaleFpdec::<i32, -6>::ZERO;

        let four_p12: ConstScaleFpdec<i32, 12> = fpdec!(4e-12);
        let four_n12: ConstScaleFpdec<i32, -12> = fpdec!(4e+12);
        let four_p6: ConstScaleFpdec<i32, 6> = fpdec!(4e-6);
        let four_n6: ConstScaleFpdec<i32, -6> = fpdec!(4e+6);
        let four_p3: ConstScaleFpdec<i32, 3> = fpdec!(4e-3);
        let four_n3: ConstScaleFpdec<i32, -3> = fpdec!(4e+3);
        let four_0: ConstScaleFpdec<i32, 0> = fpdec!(4);

        assert_eq!(two_p12.mantissa(), 2);
        assert_eq!(two_n12.mantissa(), 2);
        assert_eq!(two_p6.mantissa(), 2);
        assert_eq!(two_n6.mantissa(), 2);
        assert_eq!(two_p3.mantissa(), 2);
        assert_eq!(two_n3.mantissa(), 2);
        assert_eq!(two_0.mantissa(), 2);
        assert_eq!(zero_p6.mantissa(), 0);
        assert_eq!(zero_n6.mantissa(), 0);

        // S + S2 = SR
        assert_eq!(two_p3.checked_mul(two_p3), Some(four_p6));
        assert_eq!(two_n3.checked_mul(two_n3), Some(four_n6));
        assert_eq!(two_p6.checked_mul(two_p6), Some(four_p12));
        assert_eq!(two_n6.checked_mul(two_n6), Some(four_n12));
        assert_eq!(two_0.checked_mul(two_p6), Some(four_p6));
        assert_eq!(two_0.checked_mul(two_n6), Some(four_n6));
        assert_eq!(two_n6.checked_mul(two_p6), Some(four_0));

        // S + S2 > SR
        assert_eq!(two_p6.checked_mul(two_p6), Some(zero_p6));
        assert_eq!(two_p6.checked_mul(two_p3), Some(zero_p6));
        assert_eq!(two_n6.checked_mul(two_p3), Some(zero_n6));
        assert_eq!(two_n12.checked_mul(two_p12), Some(zero_n6));

        // S + S2 < SR
        assert_eq!(two_p6.checked_mul(two_p3), four_p12.checked_mul_int(1000));
        assert_eq!(two_p6.checked_mul(two_0), four_p12.checked_mul_int(1000000));
        assert_eq!(two_n6.checked_mul(two_n3), four_n3.checked_mul_int(1000000));
        assert_eq!(two_n6.checked_mul(two_p3), four_p3.checked_mul_int(1000000));
        assert_eq!(two_p6.checked_mul(two_n3), four_p6.checked_mul_int(1000));
        assert_eq!(two_n12.checked_mul(two_p12), four_p3.checked_mul_int(1000));

        // S + S2 - SR > 9
        assert_eq!(two_p6.checked_mul::<_, 6, 0>(two_p6), None);
        assert_eq!(two_p12.checked_mul::<_, 6, 6>(two_p6), None);
        assert_eq!(two_p6.checked_mul::<_, -6, -10>(two_n6), None);

        // S + S2 - SR < -9
        assert_eq!(two_n6.checked_mul::<_, -6, 0>(two_n6), None);
        assert_eq!(two_n12.checked_mul::<_, -6, -6>(two_n6), None);
        assert_eq!(two_n6.checked_mul::<_, 6, 10>(two_p6), None);
    }

    #[test]
    fn test_mul_overflow() {
        let max_p6 = ConstScaleFpdec::<i32, 6>::MAX;
        let min_p6 = ConstScaleFpdec::<i32, 6>::MIN;
        let ten_p6: ConstScaleFpdec<i32, 6> = fpdec!(10);
        let half_min_p6 = ConstScaleFpdec::<i32, 6>::MIN.checked_div_int(2).unwrap();
        let half_max_p6 = ConstScaleFpdec::<i32, 6>::MAX
            .checked_div_int_ext(2, Rounding::Floor, None)
            .unwrap();

        let max_p5 = ConstScaleFpdec::<i32, 5>::MAX;
        let min_p5 = ConstScaleFpdec::<i32, 5>::MIN;

        assert_eq!(max_p6.checked_mul_int(2), None);
        assert_eq!(min_p6.checked_mul_int(2), None);
        assert_eq!(half_min_p6.checked_mul_int(2), Some(min_p6));
        assert_eq!(
            half_max_p6.checked_mul_int(2),
            max_p6.checked_sub(fpdec!(1e-6))
        );

        assert_eq!(max_p6.checked_mul(ten_p6), Some(max_p5));
        assert_eq!(min_p6.checked_mul(ten_p6), Some(min_p5));

        // mantissa overflow
        assert_eq!(max_p6.checked_mul::<_, 6, 6>(max_p6), None);
        assert_eq!(max_p6.checked_mul::<_, 6, 6>(ten_p6), None);
        assert_eq!(half_max_p6.checked_mul::<_, 6, 6>(ten_p6), None);
        assert_eq!(min_p6.checked_mul::<_, 6, 6>(min_p6), None);
        assert_eq!(min_p6.checked_mul::<_, 6, 6>(ten_p6), None);
        assert_eq!(half_min_p6.checked_mul::<_, 6, 6>(ten_p6), None);

        // diff_scale out of range [-9, 9]
        assert_eq!(max_p6.checked_mul::<_, 6, 2>(max_p6), None);
        assert_eq!(ten_p6.checked_mul::<_, 6, 2>(ten_p6), None);
        assert_eq!(max_p6.checked_mul::<_, 6, -22>(max_p6), None);
        assert_eq!(ten_p6.checked_mul::<_, 6, -22>(ten_p6), None);
    }

    #[test]
    fn test_div() {
        let two_p12: ConstScaleFpdec<i32, 12> = fpdec!(2e-12);
        let two_n12: ConstScaleFpdec<i32, -12> = fpdec!(2e+12);
        let two_p6: ConstScaleFpdec<i32, 6> = fpdec!(2e-6);
        let two_n6: ConstScaleFpdec<i32, -6> = fpdec!(2e+6);
        let two_p3: ConstScaleFpdec<i32, 3> = fpdec!(2e-3);
        let two_n3: ConstScaleFpdec<i32, -3> = fpdec!(2e+3);
        let two_0: ConstScaleFpdec<i32, 0> = fpdec!(2);

        let zero_p6 = ConstScaleFpdec::<i32, 6>::ZERO;
        let zero_n6 = ConstScaleFpdec::<i32, -6>::ZERO;

        let four_p12: ConstScaleFpdec<i32, 12> = fpdec!(4e-12);
        let four_n12: ConstScaleFpdec<i32, -12> = fpdec!(4e+12);
        let four_p6: ConstScaleFpdec<i32, 6> = fpdec!(4e-6);
        let four_n6: ConstScaleFpdec<i32, -6> = fpdec!(4e+6);
        let four_p3: ConstScaleFpdec<i32, 3> = fpdec!(4e-3);
        let four_n3: ConstScaleFpdec<i32, -3> = fpdec!(4e+3);
        let four_0: ConstScaleFpdec<i32, 0> = fpdec!(4);

        // S - S2 = SR
        assert_eq!(four_p3.checked_div(two_p3), Some(two_0));
        assert_eq!(four_n3.checked_div(two_n3), Some(two_0));
        assert_eq!(four_p12.checked_div(two_p6), Some(two_p6));
        assert_eq!(four_n6.checked_div(two_n12), Some(two_p6));
        assert_eq!(four_0.checked_div(two_p6), Some(two_n6));
        assert_eq!(four_0.checked_div(two_n6), Some(two_p6));
        assert_eq!(four_n6.checked_div(two_p6), Some(two_n12));

        // S - S2 > SR
        assert_eq!(four_p6.checked_div(two_p6), Some(zero_n6));
        assert_eq!(four_p12.checked_div(two_p3), Some(zero_p6));
        assert_eq!(four_p6.checked_div(two_n3), Some(zero_p6));

        // S - S2 < SR
        assert_eq!(four_p6.checked_div(two_p3), two_p6.checked_mul_int(1000));
        assert_eq!(four_p6.checked_div(two_0), two_p12.checked_mul_int(1000000));
        assert_eq!(four_n6.checked_div(two_n3), two_0.checked_mul_int(1000));
        assert_eq!(four_n6.checked_div(two_p3), two_n6.checked_mul_int(1000));
        assert_eq!(four_p6.checked_div(two_n3), two_p12.checked_mul_int(1000));

        // S - S2 - SR > 9
        assert_eq!(four_p6.checked_div::<_, 6, -10>(two_p6), None);
        assert_eq!(four_p12.checked_div::<_, 6, -6>(two_p6), None);
        assert_eq!(four_p6.checked_div::<_, -6, 0>(two_n6), None);

        // S - S2 - SR < -9
        assert_eq!(four_n6.checked_div::<_, -6, 10>(two_n6), None);
        assert_eq!(four_n12.checked_div::<_, -6, 6>(two_n6), None);
        assert_eq!(four_n6.checked_div::<_, 6, 0>(two_p6), None);
    }

    #[test]
    fn test_div_overflow() {
        let max_p6 = ConstScaleFpdec::<i32, 6>::MAX;
        let min_p6 = ConstScaleFpdec::<i32, 6>::MIN;
        let cent_p6: ConstScaleFpdec<i32, 6> = fpdec!(0.1);
        let half_min_p6 = ConstScaleFpdec::<i32, 6>::MIN.checked_div_int(2).unwrap();
        let half_max_p6 = ConstScaleFpdec::<i32, 6>::MAX
            .checked_div_int_ext(2, Rounding::Floor, None)
            .unwrap();

        let max_p5 = ConstScaleFpdec::<i32, 5>::MAX;
        let min_p5 = ConstScaleFpdec::<i32, 5>::MIN;

        assert_eq!(max_p6.checked_div(cent_p6), Some(max_p5));
        assert_eq!(min_p6.checked_div(cent_p6), Some(min_p5));

        // mantissa overflow
        assert_eq!(max_p6.checked_div::<_, 6, 6>(cent_p6), None);
        assert_eq!(half_max_p6.checked_div::<_, 6, 6>(cent_p6), None);
        assert_eq!(min_p6.checked_div::<_, 6, 6>(cent_p6), None);
        assert_eq!(half_min_p6.checked_div::<_, 6, 6>(cent_p6), None);

        // diff_scale out of range [-9, 9]
        assert_eq!(max_p6.checked_div::<_, 6, 10>(max_p6), None);
        assert_eq!(cent_p6.checked_div::<_, 6, 10>(cent_p6), None);
        assert_eq!(max_p6.checked_div::<_, 6, -10>(max_p6), None);
        assert_eq!(cent_p6.checked_div::<_, 6, -10>(cent_p6), None);
    }

    type Dec32p2 = ConstScaleFpdec<i32, 2>;
    type Dec32n2 = ConstScaleFpdec<i32, -2>;
    #[test]
    fn test_from_int() {
        assert_eq!(Dec32p2::try_from(1_i16).unwrap().mantissa(), 100);
        assert_eq!(Dec32p2::try_from(i32::MAX), Err(ParseError::Overflow));

        // avoid overflow for: i16::MAX * 100
        assert_eq!(
            Dec32p2::try_from(i16::MAX).unwrap().mantissa(),
            i16::MAX as i32 * 100
        );

        // avoid overflow for: i32::MAX * 100
        assert_eq!(
            Dec32n2::try_from(i32::MAX as i64 * 100).unwrap().mantissa(),
            i32::MAX
        );

        // overflow
        assert_eq!(Dec32p2::try_from(i32::MAX), Err(ParseError::Overflow));
        assert_eq!(
            Dec32n2::try_from(i32::MAX as i64 * 1000),
            Err(ParseError::Overflow)
        );
    }

    #[test]
    fn test_from_float() {
        assert_eq!(Dec32p2::try_from(3.1415).unwrap().mantissa(), 314);
        assert_eq!(Dec32n2::try_from(31415.16).unwrap().mantissa(), 314);

        assert_eq!(Dec32p2::try_from(3.14e10), Err(ParseError::Overflow));
        assert_eq!(Dec32n2::try_from(3.14e16), Err(ParseError::Overflow));
    }

    #[test]
    fn test_fmt() {
        // FromStr
        assert_eq!(Dec32p2::from_str("0"), Ok(fpdec!(0)));
        assert_eq!(Dec32p2::from_str("1000"), Ok(fpdec!(1000)));
        assert_eq!(Dec32p2::from_str("-1000"), Ok(fpdec!(-1000)));
        assert_eq!(Dec32p2::from_str("0.12"), Ok(fpdec!(0.12)));
        assert_eq!(Dec32p2::from_str("-0.12"), Ok(fpdec!(-0.12)));
        assert_eq!(Dec32p2::from_str("3.14"), Ok(fpdec!(3.14)));
        assert_eq!(Dec32p2::from_str("-3.14"), Ok(fpdec!(-3.14)));
        assert_eq!(Dec32p2::from_str("3.1415"), Err(ParseError::Precision));

        assert_eq!(Dec32n2::from_str("1000"), Ok(fpdec!(1000)));
        assert_eq!(Dec32n2::from_str("-1000"), Ok(fpdec!(-1000)));
        assert_eq!(Dec32n2::from_str("1000.00"), Err(ParseError::Precision));
        assert_eq!(Dec32n2::from_str("1001"), Err(ParseError::Precision));
    }
}
