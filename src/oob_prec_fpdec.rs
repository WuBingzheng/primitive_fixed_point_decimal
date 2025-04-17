use crate::fpdec_inner::FpdecInner;
use crate::ParseError;
use crate::static_prec_fpdec::StaticPrecFpdec;
use int_div_cum_error::{Rounding, checked_divide};
use num_traits::{Num, cast::FromPrimitive, float::Float};
use std::fmt;


/// Out-of-band-precision fixed-point decimal.
///
/// `I` is the inner integer type, could be `i8`, `i16`, `i32`, `i64`, or `i128`.
///
/// For example, `OobPrecFpdec<i64>` means using `i64` as the inner
/// integer with about 18 significant digits. It's your job to save
/// the out-of-band precision somewhere else.
///
/// See [the module-level documentation](super) for more information.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Debug)]
pub struct OobPrecFpdec<I>(I);

impl<I> OobPrecFpdec<I>
where I: FpdecInner
{
    crate::none_prec_common::define_none_prec_common!();

    /// Checked multiplication. Computes `self * rhs`eturning `None` if
    /// overflow occurred.
    ///
    /// Equivalent to [`Self::checked_mul_ext`] with `Rounding::Round`.
    pub fn checked_mul<J>(
        self,
        rhs: OobPrecFpdec<J>,
        diff_precision: i32, // P(self) + P(rhs) - P(result)
    ) -> Option<OobPrecFpdec<I>>
        where J: FpdecInner
    {
        self.checked_mul_ext(rhs, diff_precision, Rounding::Round, None)
    }

    /// Checked multiplication. Computes `self * rhs`eturning `None` if
    /// overflow occurred.
    ///
    /// The type of `rhs` can have different inner integer `J`,
    /// while the type of result must have the same `I`.
    ///
    /// Argument: `diff_precision = precision(self) + precision(rhs) - precision(result)`.
    ///
    /// If the diff_precision < 0, then rounding operations
    /// are required and precision may be lost.
    /// You can specify the rounding type and cumulative error.
    ///
    /// See the [cumulative error section](index.html#cumulative-error)
    /// for more information and examples.
    ///
    /// # Examples
    /// 
    /// ```
    /// use primitive_fixed_point_decimal::{OobPrecFpdec, Rounding, fpdec};
    /// type Balance = OobPrecFpdec<i64>;
    /// type FeeRate = OobPrecFpdec<i16>; // different types
    ///
    /// let balance: Balance = fpdec!(12.60, 2); // precision=2
    /// let rate: FeeRate = fpdec!(0.01, 4); // precision=4
    ///
    /// // calculate fee 3 times with same arguments, with `cum_error`.
    /// // but have different results: 0.13, 0.13 and 0.12
    /// let mut cum_error: i64 = 0;
    ///
    /// let fee: Balance = balance.checked_mul_ext(rate, 4, Rounding::Ceiling, Some(&mut cum_error)).unwrap();
    /// assert_eq!(fee, fpdec!(0.13, 2));
    ///
    /// let fee: Balance = balance.checked_mul_ext(rate, 4, Rounding::Ceiling, Some(&mut cum_error)).unwrap();
    /// assert_eq!(fee, fpdec!(0.13, 2));
    ///
    /// let fee: Balance = balance.checked_mul_ext(rate, 4, Rounding::Ceiling, Some(&mut cum_error)).unwrap();
    /// assert_eq!(fee, fpdec!(0.12, 2)); // here different
    /// ```
    pub fn checked_mul_ext<J>(
        self,
        rhs: OobPrecFpdec<J>,
        diff_precision: i32, // P(self) + P(rhs) - P(result)
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<OobPrecFpdec<I>>
        where J: FpdecInner
    {
        self.0.checked_mul_ext(I::from(rhs.0)?, diff_precision, rounding, cum_error)
            .map(Self)
    }

    /// Checked division. Computes `self / rhs`eturning `None` if
    /// division by 0 or overflow occurred.
    ///
    /// Equivalent to [`Self::checked_div_ext`] with `Rounding::Round`.
    pub fn checked_div<J>(
        self,
        rhs: OobPrecFpdec<J>,
        diff_precision: i32, // P(self) - P(rhs) - P(result)
    ) -> Option<OobPrecFpdec<I>>
        where J: FpdecInner
    {
        self.checked_div_ext(rhs, diff_precision, Rounding::Round, None)
    }

    /// Checked division. Computes `self / rhs`eturning `None` if
    /// division by 0 or overflow occurred.
    ///
    /// The type of `rhs` can have different inner integer `J`,
    /// while the type of result must have the same `I`.
    ///
    /// Argument: `diff_precision = precision(self) - precision(rhs) - precision(result)`.
    ///
    /// You can specify the rounding type and cumulative error.
    /// See the [cumulative error section](index.html#cumulative-error)
    /// for more information and examples.
    ///
    /// # Examples
    /// 
    /// ```
    /// use primitive_fixed_point_decimal::{OobPrecFpdec, Rounding, fpdec};
    /// type Balance = OobPrecFpdec<i64>;
    /// type FeeRate = OobPrecFpdec<i16>; // different types
    ///
    /// let fee: Balance = fpdec!(0.13, 2); // precision=2
    /// let rate: FeeRate = fpdec!(0.03, 4); // precision=4
    ///
    /// let balance: Balance = fee.checked_div_ext(rate, -4, Rounding::Ceiling, None).unwrap();
    /// assert_eq!(balance, fpdec!(4.34, 2));
    /// ```
    pub fn checked_div_ext<J>(
        self,
        rhs: OobPrecFpdec<J>,
        diff_precision: i32, // P(self) - P(rhs) - P(result)
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<OobPrecFpdec<I>>
        where J: FpdecInner
    {
        self.0.checked_div_ext(I::from(rhs.0)?, diff_precision, rounding, cum_error)
            .map(Self)
    }

    /// Shrink some precision.
    ///
    /// Equivalent to [`Self::shrink_with_rounding`] with `Rounding::Round`.
    pub fn shrink(self, reduce_precision: i32) -> Self {
        self.shrink_with_rounding(reduce_precision, Rounding::Round)
    }

    /// Shrink some precision.
    ///
    /// The `reduct_precision` argument specifies the number of precision to be
    /// reduced rather than the number to be retained.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobPrecFpdec, Rounding, fpdec};
    /// type Price = OobPrecFpdec<i64>;
    ///
    /// let price: Price = fpdec!(12.12345678, 8);
    ///
    /// assert_eq!(price.shrink(2), // reduce 2 precision
    ///     fpdec!(12.123457, 8)); // Rounding::Round as default
    ///
    /// assert_eq!(price.shrink_with_rounding(2, Rounding::Floor),
    ///     fpdec!(12.123456, 8));
    /// ```
    pub fn shrink_with_rounding(
        self,
        reduce_precision: i32,
        rounding: Rounding,
    ) -> Self {
        Self(self.0.shrink_with_rounding(reduce_precision, rounding))
    }

    /// Read decimal from string.
    ///
    /// This method has 2 limitations:
    /// 1. Support decimal format only but not scientific notation;
    /// 2. Return `ParseError::Precision` if the string has more precision than `P`.
    ///
    /// If you want to skip these limitations, you can parse the string
    /// to float number first and then convert the number to this decimal.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobPrecFpdec, ParseError, fpdec};
    /// type Decimal = OobPrecFpdec<i16>;
    ///
    /// assert_eq!(Decimal::try_from_str("1.23", 4).unwrap(), fpdec!(1.23, 4));
    /// assert_eq!(Decimal::try_from_str("9999", 4), Err(ParseError::Overflow));
    /// assert_eq!(Decimal::try_from_str("1.23456", 4), Err(ParseError::Precision));
    /// ```
    pub fn try_from_str(s: &str, precision: i32) -> Result<Self, ParseError>
        where ParseError: From<<I as Num>::FromStrRadixErr>
    {
        I::try_from_str(s, precision).map(Self)
    }

    /// Convert into float types, `f32` or `f64`.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobPrecFpdec, fpdec};
    /// type Decimal = OobPrecFpdec<i32>;
    ///
    /// let dec: Decimal = fpdec!(1.234, 4);
    /// assert_eq!(dec.into_float::<f32>(4), 1.234);
    /// ```
    pub fn into_float<F>(self, precision: i32) -> F
    where F: Float
    {
        let base = F::from(10.0).unwrap();
        F::from(self.0).unwrap() / base.powi(precision)
    }
}

macro_rules! convert_from_int {
    ($from_int_type:ty) => {
        impl<I> TryFrom<($from_int_type, i32)> for OobPrecFpdec<I>
            where I: FpdecInner
        {
            type Error = ParseError;

            /// Convert from integer with precision. Returning error if
            /// overflow occurred or lossing precision under `precision < 0`.
            ///
            /// Examples:
            ///
            /// ```
            /// use std::str::FromStr;
            /// use primitive_fixed_point_decimal::{OobPrecFpdec, ParseError};
            /// type Decimal = OobPrecFpdec<i32>;
            ///
            /// assert_eq!(Decimal::try_from((123, 4)).unwrap(), Decimal::try_from_str("123", 4).unwrap());
            /// assert_eq!(Decimal::try_from((9999999, 4)), Err(ParseError::Overflow));
            /// assert_eq!(Decimal::try_from((123, -4)), Err(ParseError::Precision));
            /// ```
            fn try_from(i: ($from_int_type, i32)) -> Result<Self, Self::Error> {
                let i2 = <$from_int_type>::checked_from_int(i.0, i.1)?;
                I::from(i2).ok_or(ParseError::Overflow).map(Self)
            }
        }
    }
}
convert_from_int!(i8);
convert_from_int!(i16);
convert_from_int!(i32);
convert_from_int!(i64);
convert_from_int!(i128);

macro_rules! convert_from_float {
    ($float_type:ty, $from_fn:ident, $to_fn:ident) => {
        impl<I> TryFrom<($float_type, i32)> for OobPrecFpdec<I>
            where I: FromPrimitive + FpdecInner
        {
            type Error = ParseError;

            /// Convert from float and precision. Returning error if overflow occurred.
            ///
            /// Since it's hard for the float types to represent decimal fraction
            /// exactly, so this method always rounds the float number into
            /// OobPrecFpdec.
            ///
            /// Examples:
            ///
            /// ```
            /// use std::str::FromStr;
            /// use primitive_fixed_point_decimal::{OobPrecFpdec, ParseError};
            /// type Decimal = OobPrecFpdec<i32>;
            ///
            /// assert_eq!(Decimal::try_from((1.23, 4)).unwrap(), Decimal::try_from_str("1.23", 4).unwrap());
            /// assert_eq!(Decimal::try_from((1.23456789, 4)).unwrap(), Decimal::try_from_str("1.2346", 4).unwrap());
            /// ```
            fn try_from(f: ($float_type, i32)) -> Result<Self, Self::Error> {
                let base: $float_type = 10.0;
                let inner_f = f.0 * base.powi(f.1) as $float_type;
                I::$from_fn(inner_f.round())
                    .map(Self)
                    .ok_or(ParseError::Overflow)
            }
        }
    }
}

convert_from_float!(f32, from_f32, to_f32);
convert_from_float!(f64, from_f64, to_f64);


impl<I> OobPrecFpdec<I>
where I: FpdecInner
{
    /// Checked multiplication with StaticPrecFpdec.
    ///
    /// Equivalent to [`Self::checked_mul_static_ext`] with `Rounding::Round`.
    pub fn checked_mul_static<J, const Q: i32>(self, rhs: StaticPrecFpdec<J, Q>)
        -> Option<Self>
    where J: FpdecInner
    {
        self.checked_mul_static_ext(rhs, Rounding::Round, None)
    }

    /// Checked multiplication with StaticPrecFpdec, with rounding and cumulative
    /// error.
    ///
    /// The result will have same inner integer and precision with self.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobPrecFpdec, StaticPrecFpdec, Rounding, fpdec};
    /// type Balance = OobPrecFpdec<i64>; // oob_prec:2
    /// type FeeRate = StaticPrecFpdec<i16, 4>;
    ///
    /// let balance: Balance = fpdec!(12.60, 2);
    /// let rate: FeeRate = fpdec!(0.01);
    ///
    /// assert_eq!(balance.checked_mul_static_ext(rate, Rounding::Ceiling, None).unwrap(),
    ///     fpdec!(0.13, 2));
    /// ```
    pub fn checked_mul_static_ext<J, const Q: i32>(
        self,
        rhs: StaticPrecFpdec<J, Q>,
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<Self>
    where J: FpdecInner
    {
        self.0.checked_mul_ext(I::from(rhs.0)?, Q, rounding, cum_error)
            .map(Self)
    }

    /// Checked division with StaticPrecFpdec.
    ///
    /// Equivalent to [`Self::checked_div_static_ext`] with `Rounding::Round`.
    pub fn checked_div_static<J, const Q: i32>(self, rhs: StaticPrecFpdec<J, Q>)
        -> Option<Self>
    where J: FpdecInner
    {
        self.checked_div_static_ext(rhs, Rounding::Round, None)
    }

    /// Checked division with StaticPrecFpdec, with rounding and cumulative
    /// error.
    ///
    /// The result will have same inner integer and precision with self.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobPrecFpdec, StaticPrecFpdec, Rounding, fpdec};
    /// type Balance = OobPrecFpdec<i64>; // oob_prec:2
    /// type FeeRate = StaticPrecFpdec<i16, 4>;
    ///
    /// let fee: Balance = fpdec!(0.13, 2);
    /// let rate: FeeRate = fpdec!(0.01);
    ///
    /// assert_eq!(fee.checked_div_static_ext(rate, Rounding::Ceiling, None).unwrap(),
    ///     fpdec!(13.0, 2));
    /// ```
    pub fn checked_div_static_ext<J, const Q: i32>(
        self,
        rhs: StaticPrecFpdec<J, Q>,
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<Self>
    where J: FpdecInner
    {
        self.0.checked_div_ext(I::from(rhs.0)?, -Q, rounding, cum_error)
            .map(Self)
    }
}

impl<I> std::ops::Neg for OobPrecFpdec<I>
where I: FpdecInner
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<I> std::ops::Add for OobPrecFpdec<I>
where I: FpdecInner
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<I> std::ops::Sub for OobPrecFpdec<I>
where I: FpdecInner
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<I> std::ops::AddAssign for OobPrecFpdec<I>
where I: FpdecInner
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<I> std::ops::SubAssign for OobPrecFpdec<I>
where I: FpdecInner
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

/// Wrapper to display OobPrecFpdec.
///
/// Since the precision of OobPrecFpdec is out-of-band, we can not
/// display it directly. We have to give the precision to display.
/// `OobFmt` merge the OobPrecFpdec and precision together to display.
///
/// Examples:
///
/// ```
/// use primitive_fixed_point_decimal::{OobPrecFpdec, OobFmt, fpdec};
/// type Decimal = OobPrecFpdec<i64>;
///
/// let d: Decimal = fpdec!(3.14, 4);
///
/// assert_eq!(format!("pi is {}", OobFmt(d, 4)), String::from("pi is 3.14"));
/// ```
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Debug)]
pub struct OobFmt<I>(pub OobPrecFpdec<I>, pub i32);

impl<I> fmt::Display for OobFmt<I>
where I: FpdecInner + fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let precision = self.1;
        self.0.0.display_fmt(precision, f)
    }
}
