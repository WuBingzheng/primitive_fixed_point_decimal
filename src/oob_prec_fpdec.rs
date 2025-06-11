use crate::fpdec_inner::FpdecInner;
use crate::static_prec_fpdec::StaticPrecFpdec;
use crate::ParseError;
use int_div_cum_error::{checked_divide, Rounding};
use num_traits::{cast::FromPrimitive, float::Float, Num};
use std::fmt;
use std::str::FromStr;

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
where
    I: FpdecInner,
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
    where
        J: FpdecInner,
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
    where
        J: FpdecInner,
    {
        self.0
            .checked_mul_ext(I::from(rhs.0)?, diff_precision, rounding, cum_error)
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
    where
        J: FpdecInner,
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
    where
        J: FpdecInner,
    {
        self.0
            .checked_div_ext(I::from(rhs.0)?, diff_precision, rounding, cum_error)
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
    pub fn shrink_with_rounding(self, reduce_precision: i32, rounding: Rounding) -> Self {
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
    where
        ParseError: From<<I as Num>::FromStrRadixErr>,
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
    where
        F: Float,
    {
        let base = F::from(10.0).unwrap();
        F::from(self.0).unwrap() / base.powi(precision)
    }
}

impl<I, const P: i32> From<StaticPrecFpdec<I, P>> for OobPrecFpdec<I>
where
    I: FpdecInner,
{
    /// Convert from [`StaticPrecFpdec`] to `OobPrecFpdec` with precision `P`.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{StaticPrecFpdec, OobPrecFpdec, fpdec};
    /// type StaticDec = StaticPrecFpdec<i32, 6>;
    /// type OobDec = OobPrecFpdec<i32>; // the OOB precision is 6 too
    ///
    /// let sd: StaticDec = fpdec!(123.45);
    /// let od: OobDec = sd.into(); // `od` has the same precision=6
    /// assert_eq!(od, fpdec!(123.45, 6));
    /// ```
    fn from(sd: StaticPrecFpdec<I, P>) -> Self {
        Self(sd.mantissa())
    }
}

macro_rules! convert_from_int {
    ($from_int_type:ty) => {
        impl<I> TryFrom<($from_int_type, i32)> for OobPrecFpdec<I>
        where
            I: FpdecInner,
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
                    let i2 = <$from_int_type>::checked_from_int(i.0, i.1)?;
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
        impl<I> TryFrom<($float_type, i32)> for OobPrecFpdec<I>
        where
            I: FromPrimitive + FpdecInner,
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
    };
}

convert_from_float!(f32, from_f32, to_f32);
convert_from_float!(f64, from_f64, to_f64);

impl<I> std::ops::Neg for OobPrecFpdec<I>
where
    I: FpdecInner,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<I> std::ops::Add for OobPrecFpdec<I>
where
    I: FpdecInner,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<I> std::ops::Sub for OobPrecFpdec<I>
where
    I: FpdecInner,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<I> std::ops::AddAssign for OobPrecFpdec<I>
where
    I: FpdecInner,
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<I> std::ops::SubAssign for OobPrecFpdec<I>
where
    I: FpdecInner,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

/// Wrapper to display/load OobPrecFpdec.
///
/// Since the precision of OobPrecFpdec is out-of-band, we can not
/// display or load it directly. We have to give the precision.
/// `OobFmt` merges the OobPrecFpdec and precision together to display/load.
///
/// So `OobFmt` is available for `serde`.
///
/// Examples:
///
/// ```
/// use primitive_fixed_point_decimal::{OobPrecFpdec, OobFmt, fpdec};
/// type Decimal = OobPrecFpdec<i32>;
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
pub struct OobFmt<I>(pub OobPrecFpdec<I>, pub i32);

impl<I> fmt::Display for OobFmt<I>
where
    I: FpdecInner + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let precision = self.1;
        self.0 .0.display_fmt(precision, f)
    }
}

/// Load from string and guess the precision by counting the fraction part.
///
/// Examples:
///
/// ```
/// use primitive_fixed_point_decimal::{OobPrecFpdec, OobFmt, fpdec, ParseError};
/// type DecFmt = OobFmt<i16>;
///
/// // normal cases
/// assert_eq!("3.14".parse::<DecFmt>(), Ok(OobFmt(fpdec!(3.14, 2), 2)));
/// assert_eq!("-3.14".parse::<DecFmt>(), Ok(OobFmt(fpdec!(-3.14, 2), 2)));
///
/// // large precision
/// assert_eq!("0.000000000314".parse::<DecFmt>(), Ok(OobFmt(fpdec!(3.14e-10, 12), 12)));
///
/// // negative precison
/// assert_eq!("314000000000".parse::<DecFmt>(), Ok(OobFmt(fpdec!(3.14e11, -9), -9)));
///
/// // too large precision
/// assert_eq!("1.000000000314".parse::<DecFmt>(), Err(ParseError::Precision));
///
/// // overflow
/// assert_eq!("31415.926".parse::<DecFmt>(), Err(ParseError::Overflow));
/// ```
impl<I> FromStr for OobFmt<I>
where
    I: FpdecInner,
    ParseError: From<<I as Num>::FromStrRadixErr>,
{
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (inner, precision) = I::try_from_str_only(s)?;
        Ok(OobFmt(OobPrecFpdec(inner), precision))
    }
}

impl<I> OobFmt<I>
where
    I: FpdecInner,
{
    /// Convert to OobPrecFpdec with precision specified.
    ///
    /// Return error if overflow occurred (to bigger precision) or precision
    /// lost (to smaller precision).
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobPrecFpdec, OobFmt, fpdec, ParseError};
    /// type DecFmt = OobFmt<i16>;
    ///
    /// let df = "3.14".parse::<DecFmt>().unwrap();
    /// assert_eq!(df.rescale(4), Ok(fpdec!(3.14, 4)));
    /// assert_eq!(df.rescale(1), Err(ParseError::Precision));
    /// assert_eq!(df.rescale(10), Err(ParseError::Overflow));
    /// ```
    pub fn rescale(self, precision: i32) -> Result<OobPrecFpdec<I>, ParseError> {
        let OobFmt(dec, prec0) = self;

        if precision == prec0 {
            Ok(dec)
        } else if precision > prec0 {
            let inner = I::get_exp((precision - prec0) as usize)
                .ok_or(ParseError::Overflow)?
                .checked_mul(&dec.0)
                .ok_or(ParseError::Overflow)?;
            Ok(OobPrecFpdec(inner))
        } else {
            let diff_exp = I::get_exp((prec0 - precision) as usize).ok_or(ParseError::Precision)?;
            let inner = dec.0 / diff_exp;
            if (dec.0 % diff_exp).is_zero() {
                Ok(OobPrecFpdec(inner))
            } else {
                Err(ParseError::Precision)
            }
        }
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

/// Because we need to guess the precision, so we can load from
/// string only, but not integer or float numbers.
#[cfg(feature = "serde")]
impl<'de, I> Deserialize<'de> for OobFmt<I>
where
    I: FromPrimitive + FpdecInner,
    ParseError: From<<I as Num>::FromStrRadixErr>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::marker::PhantomData;
        use std::str::FromStr;

        struct OobFmtVistor<I>(PhantomData<I>);

        impl<'de, I> Visitor<'de> for OobFmtVistor<I>
        where
            I: FromPrimitive + FpdecInner,
            ParseError: From<<I as Num>::FromStrRadixErr>,
        {
            type Value = OobFmt<I>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "decimal")
            }

            fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                OobFmt::from_str(s).map_err(|e| E::custom(format!("decimal {:?}", e)))
            }
        }

        // TODO:
        // 1. why deserialize_any() works for StaticPrecFpdec?
        // 2. move to serde.rs?
        // 3. more rescale() to fpdec_inner.rs?
        deserializer.deserialize_str(OobFmtVistor(PhantomData))
    }
}
