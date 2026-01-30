use crate::const_scale_fpdec::ConstScaleFpdec;
use crate::fpdec_inner::FpdecInner;
use crate::{IntoRatioInt, ParseError, Rounding};

use core::{fmt, num::ParseIntError, ops, str::FromStr};

use num_traits::{cast::FromPrimitive, float::FloatCore, Num, Signed};

/// Out-of-band-scale fixed-point decimal.
///
/// `I` is the inner integer type. It could be signed `i8`, `i16`, `i32`,
/// `i64`, `i128`, or unsigned `u8`, `u16`, `u32`, `u64`, `u128`, with
/// about 2, 4, 9, 18/19 and 38 significant digits respectively.
///
/// For example, `OobScaleFpdec<i64>` means using `i64` as the underlying
/// integer. It's your job to save the out-of-band scale somewhere else.
///
/// The scale can be positive for fraction precision or be negative
/// for omitting the low-order digits of integer values.
///
/// Compared to [`ConstScaleFpdec`], this `OobScaleFpdec` has more verbose APIs:
///
/// - extra `diff_scale` argument for most operations such as `*` and `/`, but no need for `+` and `-`,
/// - use `try_from_str()` to convert from string with scale set,
/// - use `(*, i32)` tuple for converting from integers or floats,
/// - use `to_f32()` or `to_f64()` to convert to floats,
/// - use [`OobFmt`] for `Display` and `FromStr`,
/// - no associate const `SCALE`,
/// - and others.
///
/// See [the module-level documentation](super) for more information.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Debug)]
#[repr(transparent)]
pub struct OobScaleFpdec<I>(I);

impl<I> OobScaleFpdec<I>
where
    I: FpdecInner,
{
    crate::none_scale_common::define_none_scale_common!();

    /// Checked multiplication.
    ///
    /// Equivalent to [`Self::checked_mul_ext`] with `Rounding::Round`.
    #[must_use]
    pub fn checked_mul<J>(
        self,
        rhs: OobScaleFpdec<J>,
        diff_scale: i32, // scale (self + rhs - result)
    ) -> Option<OobScaleFpdec<I>>
    where
        J: FpdecInner,
    {
        self.checked_mul_ext(rhs, diff_scale, Rounding::Round)
    }

    /// Checked multiplication. Computes `self * rhs`, returning `None` if
    /// overflow occurred or argument `diff_scale` is out of range
    /// `[-Self::DIGITS, Self::DIGITS]`.
    ///
    /// The type of `rhs` can have different inner integer `J`,
    /// while the type of result must have the same `I`.
    ///
    /// Argument: `diff_scale = scale(self) + scale(rhs) - scale(result)`.
    ///
    /// If the diff_scale < 0, then rounding operations are required and
    /// precision may be lost.
    /// You can specify the rounding type.
    ///
    /// # Examples
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobScaleFpdec, Rounding, fpdec};
    /// type Balance = OobScaleFpdec<i64>;
    /// type FeeRate = OobScaleFpdec<i16>; // different types
    ///
    /// let balance: Balance = fpdec!(12.30, 2); // scale=2
    /// let rate: FeeRate = fpdec!(0.01, 4); // scale=4
    ///
    /// let fee: Balance = balance.checked_mul(rate, 4).unwrap();
    /// assert_eq!(fee, fpdec!(0.12, 2));
    ///
    /// let fee: Balance = balance.checked_mul_ext(rate, 4, Rounding::Ceiling).unwrap();
    /// assert_eq!(fee, fpdec!(0.13, 2));
    /// ```
    #[must_use]
    pub fn checked_mul_ext<J>(
        self,
        rhs: OobScaleFpdec<J>,
        diff_scale: i32, // scale (self + rhs - result)
        rounding: Rounding,
    ) -> Option<OobScaleFpdec<I>>
    where
        J: FpdecInner,
    {
        self.0
            .checked_mul_ext(I::from(rhs.0)?, diff_scale, rounding)
            .map(Self)
    }

    /// Checked multiplication with `ConstScaleFpdec`.
    ///
    /// Equivalent to [`Self::checked_mul_const_scale_ext`] with `Rounding::Round`.
    ///
    /// If you make sure no overflow error, you can use `*` and `*=` instead for convenience.
    #[must_use]
    pub fn checked_mul_const_scale<J, const S: i32>(
        self,
        rhs: ConstScaleFpdec<J, S>,
    ) -> Option<OobScaleFpdec<I>>
    where
        J: FpdecInner,
    {
        self.checked_mul_const_scale_ext(rhs, Rounding::Round)
    }

    /// Checked multiplication with `ConstScaleFpdec`. Computes `self * rhs`,
    /// returning `None` if overflow occurred.
    ///
    /// Compared to [`Self::checked_mul_ext`], the result of this method inherits
    /// the scale of `self`. So the `diff_scale` (which is equal to `S`) is no
    /// more need.
    ///
    /// # Examples
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{ConstScaleFpdec, OobScaleFpdec, Rounding, fpdec};
    /// type Balance = OobScaleFpdec<i64>;
    /// type FeeRate = ConstScaleFpdec<i16, 4>;
    ///
    /// let balance: Balance = fpdec!(12.30, 2); // scale=2
    /// let rate: FeeRate = fpdec!(0.01);
    ///
    /// let fee: Balance = balance.checked_mul_const_scale(rate).unwrap();
    /// assert_eq!(fee, fpdec!(0.12, 2));
    ///
    /// let fee: Balance = balance.checked_mul_const_scale_ext(rate, Rounding::Ceiling).unwrap();
    /// assert_eq!(fee, fpdec!(0.13, 2));
    /// ```
    #[must_use]
    pub fn checked_mul_const_scale_ext<J, const S: i32>(
        self,
        rhs: ConstScaleFpdec<J, S>,
        rounding: Rounding,
    ) -> Option<OobScaleFpdec<I>>
    where
        J: FpdecInner,
    {
        self.0
            .checked_mul_ext(I::from(rhs.mantissa())?, S, rounding)
            .map(Self)
    }

    /// Checked division.
    ///
    /// Equivalent to [`Self::checked_div_ext`] with `Rounding::Round`.
    #[must_use]
    pub fn checked_div<J>(
        self,
        rhs: OobScaleFpdec<J>,
        diff_scale: i32, // scale (self - rhs - result)
    ) -> Option<OobScaleFpdec<I>>
    where
        J: FpdecInner,
    {
        self.checked_div_ext(rhs, diff_scale, Rounding::Round)
    }

    /// Checked division. Computes `self / rhs`, returning `None` if
    /// division by 0, or overflow occurred, or argument `diff_scale`
    /// is out of range `[-Self::DIGITS, Self::DIGITS]`.
    ///
    /// The type of `rhs` can have different inner integer `J`,
    /// while the type of result must have the same `I`.
    ///
    /// Argument: `diff_scale = scale(self) - scale(rhs) - scale(result)`.
    ///
    /// You can specify the rounding type.
    ///
    /// # Examples
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobScaleFpdec, Rounding, fpdec};
    /// type Balance = OobScaleFpdec<i64>;
    /// type FeeRate = OobScaleFpdec<i16>; // different types
    ///
    /// let fee: Balance = fpdec!(0.13, 2); // scale=2
    /// let rate: FeeRate = fpdec!(0.03, 4); // scale=4
    ///
    /// let balance: Balance = fee.checked_div_ext(rate, -4, Rounding::Ceiling).unwrap();
    /// assert_eq!(balance, fpdec!(4.34, 2));
    /// ```
    #[must_use]
    pub fn checked_div_ext<J>(
        self,
        rhs: OobScaleFpdec<J>,
        diff_scale: i32, // scale (self - rhs - result)
        rounding: Rounding,
    ) -> Option<OobScaleFpdec<I>>
    where
        J: FpdecInner,
    {
        self.0
            .checked_div_ext(I::from(rhs.0)?, diff_scale, rounding)
            .map(Self)
    }

    /// Checked division with `ConstScaleFpdec`.
    ///
    /// Equivalent to [`Self::checked_div_const_scale_ext`] with `Rounding::Round`.
    ///
    /// If you make sure no overflow or 0-division error, you can use `/` and `/=`
    /// instead for convenience.
    #[must_use]
    pub fn checked_div_const_scale<J, const S: i32>(
        self,
        rhs: ConstScaleFpdec<J, S>,
    ) -> Option<OobScaleFpdec<I>>
    where
        J: FpdecInner,
    {
        self.checked_div_const_scale_ext(rhs, Rounding::Round)
    }

    /// Checked division with `ConstScaleFpdec`. Computes `self / rhs`,
    /// returning `None` if division by 0 or overflow occurred.
    ///
    /// Compared to [`Self::checked_div_ext`], the result of this method inherits
    /// the scale of `self`. So the `diff_scale` (which is equal to `-S`) is no
    /// more need.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{ConstScaleFpdec, OobScaleFpdec, Rounding, fpdec};
    /// type Balance = OobScaleFpdec<i64>;
    /// type FeeRate = ConstScaleFpdec<i16, 4>;
    ///
    /// let fee: Balance = fpdec!(0.12, 2); // scale=2
    /// let rate: FeeRate = fpdec!(0.01);
    ///
    /// let balance: Balance = fee.checked_div_const_scale(rate).unwrap();
    /// assert_eq!(balance, fpdec!(12.0, 2));
    /// ```
    #[must_use]
    pub fn checked_div_const_scale_ext<J, const S: i32>(
        self,
        rhs: ConstScaleFpdec<J, S>,
        rounding: Rounding,
    ) -> Option<OobScaleFpdec<I>>
    where
        J: FpdecInner,
    {
        self.0
            .checked_div_ext(I::from(rhs.mantissa())?, -S, rounding)
            .map(Self)
    }

    /// Round the decimal.
    ///
    /// Equivalent to [`Self::round_diff_ext`] with `Rounding::Round`.
    #[must_use]
    pub fn round_diff(self, diff_scale: i32) -> Self {
        self.round_diff_ext(diff_scale, Rounding::Round)
    }

    /// Round the decimal with rounding type.
    ///
    /// The argument `diff_scale` is `original_scale - round_scale`.
    /// Return the original decimal if `diff_scale <= 0`.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobScaleFpdec, Rounding, fpdec};
    /// type Price = OobScaleFpdec<i64>;
    ///
    /// let price: Price = fpdec!(12.12345678, 8); // scale=8
    ///
    /// assert_eq!(price.round_diff(8 - 6), // reduce 2 scale
    ///     fpdec!(12.123457, 8)); // `Rounding::Round` as default
    ///
    /// assert_eq!(price.round_diff_ext(8 - 6, Rounding::Floor),
    ///     fpdec!(12.123456, 8));
    /// ```
    #[must_use]
    pub fn round_diff_ext(self, diff_scale: i32, rounding: Rounding) -> Self {
        Self(self.0.round_diff_with_rounding(diff_scale, rounding))
    }

    /// Read decimal from string.
    ///
    /// This method has 2 limitations:
    /// 1. Support decimal format only but not scientific notation;
    /// 2. Return `ParseError::Precision` if the string has more precision.
    ///
    /// If you want to skip these limitations, you can parse the string
    /// to float number first and then convert the number to this decimal.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobScaleFpdec, ParseError, fpdec};
    /// type Decimal = OobScaleFpdec<i16>;
    ///
    /// assert_eq!(Decimal::try_from_str("1.23", 4).unwrap(), fpdec!(1.23, 4));
    /// assert_eq!(Decimal::try_from_str("9999", 4), Err(ParseError::Overflow));
    /// assert_eq!(Decimal::try_from_str("1.23456", 4), Err(ParseError::Precision));
    /// ```
    #[must_use]
    pub fn try_from_str(s: &str, scale: i32) -> Result<Self, ParseError>
    where
        I: Num<FromStrRadixErr = ParseIntError>,
    {
        I::try_from_str(s, scale).map(Self)
    }

    /// Convert into `f32`.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobScaleFpdec, fpdec};
    /// type Decimal = OobScaleFpdec<i32>;
    ///
    /// let dec: Decimal = fpdec!(1.234, 4);
    /// assert_eq!(dec.to_f32(4), 1.234);
    ///
    /// let dec: Decimal = fpdec!(1234000, -3);
    /// assert_eq!(dec.to_f32(-3), 1234000.0);
    /// ```
    #[must_use]
    pub fn to_f32(self, scale: i32) -> f32 {
        let f = self.0.to_f32().unwrap();
        if scale > 0 {
            f / 10.0.powi(scale)
        } else if scale < 0 {
            f * 10.0.powi(-scale)
        } else {
            f
        }
    }

    /// Convert into `f64`.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobScaleFpdec, fpdec};
    /// type Decimal = OobScaleFpdec<i32>;
    ///
    /// let dec: Decimal = fpdec!(1.234, 4);
    /// assert_eq!(dec.to_f64(4), 1.234);
    ///
    /// let dec: Decimal = fpdec!(1234000, -3);
    /// assert_eq!(dec.to_f64(-3), 1234000.0);
    /// ```
    #[must_use]
    pub fn to_f64(self, scale: i32) -> f64 {
        let f = self.0.to_f64().unwrap();
        if scale > 0 {
            f / 10.0.powi(scale)
        } else if scale < 0 {
            f * 10.0.powi(-scale)
        } else {
            f
        }
    }
}

impl<I> OobScaleFpdec<I>
where
    I: FpdecInner + Signed,
{
    crate::none_scale_common::define_none_scale_common_signed!();
}

impl<I, const S: i32> From<ConstScaleFpdec<I, S>> for OobScaleFpdec<I>
where
    I: FpdecInner,
{
    /// Convert from `ConstScaleFpdec` to `OobScaleFpdec` with scale `S`.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{ConstScaleFpdec, OobScaleFpdec, fpdec};
    /// type ConstDec = ConstScaleFpdec<i32, 6>;
    /// type OobDec = OobScaleFpdec<i32>; // the OOB scale is 6 too
    ///
    /// let sd: ConstDec = fpdec!(123.45);
    /// let od: OobDec = sd.into(); // `od` has the same scale=6
    /// assert_eq!(od, fpdec!(123.45, 6));
    /// ```
    fn from(sd: ConstScaleFpdec<I, S>) -> Self {
        Self(sd.mantissa())
    }
}

macro_rules! convert_from_int {
    ($from_int_type:ty) => {
        impl<I> TryFrom<($from_int_type, i32)> for OobScaleFpdec<I>
        where
            I: FpdecInner,
        {
            type Error = ParseError;

            /// Convert from integer with scale. Returning error if
            /// overflow occurred or lossing precision under `scale < 0`.
            ///
            /// Examples:
            ///
            /// ```
            /// use core::str::FromStr;
            /// use primitive_fixed_point_decimal::{OobScaleFpdec, ParseError};
            /// type Decimal = OobScaleFpdec<i32>;
            ///
            /// assert_eq!(Decimal::try_from((123, 4)).unwrap(), Decimal::try_from_str("123", 4).unwrap());
            /// assert_eq!(Decimal::try_from((123_i8, 4)).unwrap(), Decimal::try_from_str("123", 4).unwrap());
            /// assert_eq!(Decimal::try_from((120000000000_i64, -10)).unwrap(), Decimal::try_from_str("120000000000", -10).unwrap());
            /// assert_eq!(Decimal::try_from((9999999, 4)), Err(ParseError::Overflow));
            /// assert_eq!(Decimal::try_from((123, -4)), Err(ParseError::Precision));
            /// ```
            fn try_from(i: ($from_int_type, i32)) -> Result<Self, Self::Error> {
                if i.1 > 0 {
                    // convert from type i to I first
                    let i2 = I::from(i.0).ok_or(ParseError::Overflow)?;
                    I::checked_from_int(i2, i.1).map(Self)
                } else {
                    // convert to fpdec inner first
                    let i2 = i.0.checked_from_int(i.1)?;
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
convert_from_int!(u8);
convert_from_int!(u16);
convert_from_int!(u32);
convert_from_int!(u64);
convert_from_int!(u128);

macro_rules! convert_from_float {
    ($float_type:ty, $from_fn:ident) => {
        impl<I> TryFrom<($float_type, i32)> for OobScaleFpdec<I>
        where
            I: FromPrimitive + FpdecInner,
        {
            type Error = ParseError;

            /// Convert from float and scale. Returning error if overflow occurred.
            ///
            /// Since it's hard for the float types to represent decimal fraction
            /// exactly, so this method always rounds the float number into
            /// OobScaleFpdec.
            ///
            /// Examples:
            ///
            /// ```
            /// use core::str::FromStr;
            /// use primitive_fixed_point_decimal::{OobScaleFpdec, ParseError};
            /// type Decimal = OobScaleFpdec<i32>;
            ///
            /// assert_eq!(Decimal::try_from((1.23, 4)).unwrap(), Decimal::try_from_str("1.23", 4).unwrap());
            /// assert_eq!(Decimal::try_from((1.23456789, 4)).unwrap(), Decimal::try_from_str("1.2346", 4).unwrap());
            /// ```
            fn try_from(t: ($float_type, i32)) -> Result<Self, Self::Error> {
                let (f, scale) = t;
                let inner_f = if scale > 0 {
                    f * 10.0.powi(scale)
                } else if scale < 0 {
                    f / 10.0.powi(-scale)
                } else {
                    f
                };
                I::$from_fn(inner_f.round())
                    .map(Self)
                    .ok_or(ParseError::Overflow)
            }
        }
    };
}

convert_from_float!(f32, from_f32);
convert_from_float!(f64, from_f64);

impl<I> ops::Neg for OobScaleFpdec<I>
where
    I: FpdecInner + Signed,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<I> ops::Add for OobScaleFpdec<I>
where
    I: FpdecInner,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<I> ops::Sub for OobScaleFpdec<I>
where
    I: FpdecInner,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

/// Performs the `*` operation with an integer.
///
/// # Panics
///
/// If [`Self::checked_mul_int`] returns `None`.
impl<I, J> ops::Mul<J> for OobScaleFpdec<I>
where
    I: FpdecInner,
    J: Into<I> + Num,
{
    type Output = Self;
    fn mul(self, rhs: J) -> Self::Output {
        self.checked_mul_int(rhs)
            .expect("overflow in decimal multiplication")
    }
}

/// Performs the `*` operation with `ConstScaleFpdec`.
///
/// # Panics
///
/// If [`Self::checked_mul_const_scale`] returns `None`.
impl<I, J, const S: i32> ops::Mul<ConstScaleFpdec<J, S>> for OobScaleFpdec<I>
where
    I: FpdecInner,
    J: FpdecInner,
{
    type Output = Self;
    fn mul(self, rhs: ConstScaleFpdec<J, S>) -> Self::Output {
        self.checked_mul_const_scale(rhs)
            .expect("overflow in decimal multiplication")
    }
}

/// Performs the `/` operation with an integer.
///
/// # Panics
///
/// If [`Self::checked_div_int`] returns `None`.
impl<I, J> ops::Div<J> for OobScaleFpdec<I>
where
    I: FpdecInner,
    J: Into<I> + Num,
{
    type Output = Self;
    fn div(self, rhs: J) -> Self::Output {
        self.checked_div_int(rhs).expect("fail in decimal division")
    }
}

/// Performs the `/` operation with `ConstScaleFpdec`.
///
/// # Panics
///
/// If [`Self::checked_div_const_scale`] returns `None`.
impl<I, J, const S: i32> ops::Div<ConstScaleFpdec<J, S>> for OobScaleFpdec<I>
where
    I: FpdecInner,
    J: FpdecInner,
{
    type Output = Self;
    fn div(self, rhs: ConstScaleFpdec<J, S>) -> Self::Output {
        self.checked_div_const_scale(rhs)
            .expect("overflow in decimal multiplication")
    }
}

impl<I> ops::AddAssign for OobScaleFpdec<I>
where
    I: FpdecInner,
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<I> ops::SubAssign for OobScaleFpdec<I>
where
    I: FpdecInner,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl<I, J> ops::MulAssign<J> for OobScaleFpdec<I>
where
    I: FpdecInner,
    J: Into<I> + Num,
{
    fn mul_assign(&mut self, rhs: J) {
        *self = *self * rhs;
    }
}

impl<I, J, const S: i32> ops::MulAssign<ConstScaleFpdec<J, S>> for OobScaleFpdec<I>
where
    I: FpdecInner,
    J: FpdecInner,
{
    fn mul_assign(&mut self, rhs: ConstScaleFpdec<J, S>) {
        *self = *self * rhs;
    }
}

impl<I, J> ops::DivAssign<J> for OobScaleFpdec<I>
where
    I: FpdecInner,
    J: Into<I> + Num,
{
    fn div_assign(&mut self, rhs: J) {
        *self = *self / rhs;
    }
}

impl<I, J, const S: i32> ops::DivAssign<ConstScaleFpdec<J, S>> for OobScaleFpdec<I>
where
    I: FpdecInner,
    J: FpdecInner,
{
    fn div_assign(&mut self, rhs: ConstScaleFpdec<J, S>) {
        *self = *self / rhs;
    }
}

impl<I> core::iter::Sum for OobScaleFpdec<I>
where
    I: FpdecInner,
{
    fn sum<Iter: Iterator<Item = Self>>(iter: Iter) -> Self {
        iter.fold(Self::ZERO, |acc, d| acc + d)
    }
}

impl<'a, I> core::iter::Sum<&'a OobScaleFpdec<I>> for OobScaleFpdec<I>
where
    I: FpdecInner,
{
    fn sum<Iter: Iterator<Item = &'a Self>>(iter: Iter) -> Self {
        iter.fold(Self::ZERO, |acc, d| acc + *d)
    }
}

/// Wrapper to display/load OobScaleFpdec.
///
/// Since the scale of OobScaleFpdec is out-of-band, we can not
/// display or load it directly. We have to give the scale.
/// `OobFmt` merges the OobScaleFpdec and scale together to display/load.
///
/// So `OobFmt` is available for `serde`.
///
/// Examples:
///
/// ```
/// use primitive_fixed_point_decimal::{OobScaleFpdec, OobFmt, fpdec};
/// type Decimal = OobScaleFpdec<i32>;
///
/// let d: Decimal = fpdec!(3.14, 4);
///
/// // display
/// assert_eq!(format!("pi is {}", OobFmt(d, 4)), String::from("pi is 3.14"));
///
/// // load from string
/// let of: OobFmt<i32> = "3.14".parse().unwrap();
/// let d2: Decimal = of.rescale(4).unwrap();
/// assert_eq!(d, d2);
/// ```
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Debug)]
pub struct OobFmt<I>(pub OobScaleFpdec<I>, pub i32);

/// Display the decimal.
///
/// It supports some [formatting options](https://doc.rust-lang.org/std/fmt/index.html#formatting-parameters):
/// width, fill, alignment, precision, sign and 0-fill.
///
/// **Panic**: if the scale is too big (>200 or <-200) or the specified
/// precision is too big (>200).
///
/// Examples:
///
/// ```
/// use primitive_fixed_point_decimal::{OobScaleFpdec, OobFmt, fpdec};
/// type Decimal = OobScaleFpdec<i32>;
///
/// let d: Decimal = fpdec!(12.3470, 4);
/// let f = OobFmt(d, 4);
///
/// assert_eq!(format!("{}", f), "12.347"); // no option set: omit tailing 0
/// assert_eq!(format!("{:.4}", f), "12.3470"); // set precision: pad 0
/// assert_eq!(format!("{:.2}", f), "12.35"); // set smaller precision: round the number
/// assert_eq!(format!("{:x>10}", f), "xxxx12.347"); // set width, fill, alignment
/// assert_eq!(format!("{:+}", f), "+12.347"); // set sign
impl<I> fmt::Display for OobFmt<I>
where
    I: FpdecInner + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let scale = self.1;
        self.0 .0.display_fmt(scale, f)
    }
}

/// Load from string and guess the scale by counting the fraction part.
///
/// Generally you should then call [`OobFmt::rescale()`] to convert to the target
/// scale.
///
/// You can also use [`OobScaleFpdec::try_from_str()`] instead with scale set, to avoid
/// the guessing and rescaling.
///
/// Examples:
///
/// ```
/// use primitive_fixed_point_decimal::{OobScaleFpdec, OobFmt, fpdec, ParseError};
/// type DecFmt = OobFmt<i16>;
///
/// // normal cases
/// assert_eq!("3.14".parse::<DecFmt>(), Ok(OobFmt(fpdec!(3.14, 2), 2)));
/// assert_eq!("-3.14".parse::<DecFmt>(), Ok(OobFmt(fpdec!(-3.14, 2), 2)));
///
/// // call rescale() if you want 3 scale
/// assert_eq!("3.14".parse::<DecFmt>().unwrap().rescale(3), Ok(fpdec!(3.14, 3)));
///
/// // large scale
/// assert_eq!("0.000000000314".parse::<DecFmt>(), Ok(OobFmt(fpdec!(3.14e-10, 12), 12)));
///
/// // negative scale
/// assert_eq!("314000000000".parse::<DecFmt>(), Ok(OobFmt(fpdec!(3.14e11, -9), -9)));
///
/// // too large scale
/// assert_eq!("1.000000000314".parse::<DecFmt>(), Err(ParseError::Precision));
///
/// // overflow
/// assert_eq!("31415.926".parse::<DecFmt>(), Err(ParseError::Overflow));
/// ```
impl<I> FromStr for OobFmt<I>
where
    I: FpdecInner + Num<FromStrRadixErr = ParseIntError>,
{
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (inner, scale) = I::try_from_str_only(s)?;
        Ok(OobFmt(OobScaleFpdec(inner), scale))
    }
}

impl<I> OobFmt<I>
where
    I: FpdecInner,
{
    /// Convert to OobScaleFpdec with scale specified.
    ///
    /// Return error if overflow occurred (to bigger scale) or precision
    /// lost (to smaller scale).
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobScaleFpdec, OobFmt, fpdec, ParseError};
    /// type DecFmt = OobFmt<i16>;
    ///
    /// let df = "3.14".parse::<DecFmt>().unwrap();
    /// assert_eq!(df.rescale(4), Ok(fpdec!(3.14, 4)));
    /// assert_eq!(df.rescale(1), Err(ParseError::Precision));
    /// assert_eq!(df.rescale(10), Err(ParseError::Overflow));
    /// ```
    #[must_use]
    pub fn rescale(self, scale2: i32) -> Result<OobScaleFpdec<I>, ParseError> {
        let OobFmt(dec, scale0) = self;

        if scale2 == scale0 {
            Ok(dec)
        } else if scale2 > scale0 {
            let inner = I::get_exp((scale2 - scale0) as usize)
                .ok_or(ParseError::Overflow)?
                .checked_mul(&dec.0)
                .ok_or(ParseError::Overflow)?;
            Ok(OobScaleFpdec(inner))
        } else {
            let diff_exp = I::get_exp((scale0 - scale2) as usize).ok_or(ParseError::Precision)?;
            let inner = dec.0 / diff_exp;
            if (dec.0 % diff_exp).is_zero() {
                Ok(OobScaleFpdec(inner))
            } else {
                Err(ParseError::Precision)
            }
        }
    }
}

impl<I, J> IntoRatioInt<J> for OobScaleFpdec<I>
where
    I: FpdecInner + Into<J>,
{
    fn to_int(self) -> J {
        self.mantissa().into()
    }
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "serde")]
impl<I> Serialize for OobFmt<I>
where
    I: FpdecInner + fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

/// Because we need to guess the scale, so we can load from
/// string only, but not integer or float numbers.
#[cfg(feature = "serde")]
impl<'de, I> Deserialize<'de> for OobFmt<I>
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

        struct OobFmtVistor<I>(PhantomData<I>);

        impl<'de, I> Visitor<'de> for OobFmtVistor<I>
        where
            I: FromPrimitive + FpdecInner + Num<FromStrRadixErr = ParseIntError>,
        {
            type Value = OobFmt<I>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "decimal")
            }

            fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                OobFmt::from_str(s).map_err(E::custom)
            }
        }

        // TODO:
        // 1. why deserialize_any() works for ConstScaleFpdec?
        // 2. move to serde.rs?
        // 3. more rescale() to fpdec_inner.rs?
        deserializer.deserialize_str(OobFmtVistor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as primitive_fixed_point_decimal;
    use crate::fpdec;

    type Dec32 = OobScaleFpdec<i32>;
    type Fmt32 = OobFmt<i32>;
    #[allow(non_snake_case)]
    fn Fmt32(d: Dec32, s: i32) -> Fmt32 {
        OobFmt::<i32>(d, s)
    }

    #[test]
    fn test_mul() {
        let two = Dec32::from_mantissa(2);
        let four = Dec32::from_mantissa(4);
        let zero = Dec32::ZERO;

        // S + S2 = SR
        assert_eq!(two.checked_mul(two, 0), Some(four));

        // S + S2 > SR
        assert_eq!(two.checked_mul(two, 3), Some(zero));

        // S + S2 < SR
        assert_eq!(two.checked_mul(two, -3), four.checked_mul_int(1000));

        // S + S2 - SR > 9
        assert_eq!(two.checked_mul(two, 10), None);

        // S + S2 - SR < -9
        assert_eq!(two.checked_mul(two, -10), None);
    }

    #[test]
    fn test_mul_overflow() {
        let max = Dec32::MAX;
        let min = Dec32::MIN;
        let ten_p6: Dec32 = fpdec!(10, 6);
        let half_min = Dec32::MIN.checked_div_int(2).unwrap();
        let half_max = Dec32::MAX.checked_div_int_ext(2, Rounding::Floor).unwrap();

        assert_eq!(max.checked_mul_int(2), None);
        assert_eq!(min.checked_mul_int(2), None);
        assert_eq!(half_min.checked_mul_int(2), Some(min));
        assert_eq!(half_max.checked_mul_int(2), max.checked_sub(fpdec!(1, 0)));

        assert_eq!(max.checked_mul(ten_p6, 7), Some(max));
        assert_eq!(min.checked_mul(ten_p6, 7), Some(min));

        // mantissa overflow
        assert_eq!(max.checked_mul(max, 6), None);
        assert_eq!(max.checked_mul(ten_p6, 6), None);
        assert_eq!(half_max.checked_mul(ten_p6, 6), None);
        assert_eq!(min.checked_mul(min, 6), None);
        assert_eq!(min.checked_mul(ten_p6, 6), None);
        assert_eq!(half_min.checked_mul(ten_p6, 6), None);

        // diff_scale out of range [-9, 9]
        assert_eq!(max.checked_mul(max, 10), None);
        assert_eq!(max.checked_mul(ten_p6, 10), None);
        assert_eq!(max.checked_mul(max, -10), None);
        assert_eq!(max.checked_mul(ten_p6, -10), None);
    }

    #[test]
    fn test_div() {
        let two = Dec32::from_mantissa(2);
        let four = Dec32::from_mantissa(4);
        let zero = Dec32::ZERO;

        // S - S2 = SR
        assert_eq!(four.checked_div(two, 0), Some(two));

        // S - S2 > SR
        assert_eq!(four.checked_div(two, 3), Some(zero));

        // S - S2 < SR
        assert_eq!(four.checked_div(two, -3), two.checked_mul_int(1000));

        // S - S2 - SR > 9
        assert_eq!(four.checked_div(two, 10), None);

        // S - S2 - SR < -9
        assert_eq!(four.checked_div(two, -10), None);
    }

    #[test]
    fn test_div_overflow() {
        let max = Dec32::MAX;
        let min = Dec32::MIN;
        let cent_p6: Dec32 = fpdec!(0.1, 6);
        let half_min = Dec32::MIN.checked_div_int(2).unwrap();
        let half_max = Dec32::MAX.checked_div_int_ext(2, Rounding::Floor).unwrap();

        assert_eq!(max.checked_div(cent_p6, -5), Some(max));
        assert_eq!(min.checked_div(cent_p6, -5), Some(min));

        // mantissa overflow
        assert_eq!(max.checked_div(cent_p6, -6), None);
        assert_eq!(half_max.checked_div(cent_p6, -6), None);
        assert_eq!(min.checked_div(cent_p6, -6), None);
        assert_eq!(half_min.checked_div(cent_p6, -6), None);

        // diff_scale out of range [-9, 9]
        assert_eq!(max.checked_div(max, 10), None);
        assert_eq!(max.checked_div(cent_p6, 10), None);
        assert_eq!(max.checked_div(max, -10), None);
        assert_eq!(max.checked_div(cent_p6, -10), None);
    }

    #[test]
    fn test_from_int() {
        assert_eq!(Dec32::try_from((1_i16, 2)).unwrap().mantissa(), 100);
        assert_eq!(Dec32::try_from((i32::MAX, 2)), Err(ParseError::Overflow));

        // avoid overflow for: i16::MAX * 100
        assert_eq!(
            Dec32::try_from((i16::MAX, 2)).unwrap().mantissa(),
            i16::MAX as i32 * 100
        );

        // avoid overflow for: i32::MAX * 100
        assert_eq!(
            Dec32::try_from((i32::MAX as i64 * 100, -2))
                .unwrap()
                .mantissa(),
            i32::MAX
        );

        // overflow
        assert_eq!(Dec32::try_from((i32::MAX, 2)), Err(ParseError::Overflow));
        assert_eq!(
            Dec32::try_from((i32::MAX as i64 * 1000, -2)),
            Err(ParseError::Overflow)
        );
    }

    #[test]
    fn test_from_float() {
        assert_eq!(Dec32::try_from((3.1415, 2)).unwrap().mantissa(), 314);
        assert_eq!(Dec32::try_from((31415.16, -2)).unwrap().mantissa(), 314);

        assert_eq!(Dec32::try_from((3.14e10, 2)), Err(ParseError::Overflow));
        assert_eq!(Dec32::try_from((3.14e16, -2)), Err(ParseError::Overflow));
    }

    #[test]
    fn test_fmt() {
        // FromStr
        assert_eq!(Fmt32::from_str("0"), Ok(Fmt32(Dec32::ZERO, 0)));
        assert_eq!(Fmt32::from_str("1000"), Ok(Fmt32(fpdec!(1000, -3), -3)));
        assert_eq!(Fmt32::from_str("-1000"), Ok(Fmt32(fpdec!(-1000, -3), -3)));
        assert_eq!(Fmt32::from_str("0.12"), Ok(Fmt32(fpdec!(0.12, 2), 2)));
        assert_eq!(Fmt32::from_str("-0.12"), Ok(Fmt32(fpdec!(-0.12, 2), 2)));
        assert_eq!(Fmt32::from_str("3.14"), Ok(Fmt32(fpdec!(3.14, 2), 2)));
        assert_eq!(Fmt32::from_str("-3.14"), Ok(Fmt32(fpdec!(-3.14, 2), 2)));
        assert_eq!(
            Fmt32::from_str("3.14159265359879"),
            Err(ParseError::Overflow)
        );
    }

    #[test]
    fn test_sum() {
        let v: [Dec32; 3] = [fpdec!(0.1, 2), fpdec!(0.2, 2), fpdec!(0.3, 2)];
        let s: Dec32 = v.iter().sum();
        assert_eq!(s, fpdec!(0.6, 2));
    }
}
