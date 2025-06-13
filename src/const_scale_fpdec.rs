use crate::fpdec_inner::FpdecInner;
use crate::oob_scale_fpdec::OobScaleFpdec;
use crate::ParseError;
use core::{fmt, ops, str::FromStr};
use int_div_cum_error::{checked_divide, Rounding};
#[allow(unused_imports)]
use num_traits::float::FloatCore; // used only for `no_std`
use num_traits::{cast::FromPrimitive, Num};

/// Const-scale fixed-point decimal.
///
/// `I` is the inner integer type, could be `i8`, `i16`, `i32`, `i64`,
/// or `i128`, with 2, 4, 9, 18 and 38 significant digits respectively.
///
/// `P` is the static scale.
///
/// For example, `ConstScaleFpdec<i64, 4>` means using `i64` as the underlying
/// integer, and having `4` fraction precision.
///
/// See [the module-level documentation](super) for more information.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct ConstScaleFpdec<I, const P: i32>(I);

impl<I, const P: i32> ConstScaleFpdec<I, P>
where
    I: FpdecInner,
{
    crate::none_scale_common::define_none_scale_common!();

    /// Precision.
    pub const PRECISION: i32 = P;

    /// Checked multiplication. Computes `self * rhs`, returning `None` if
    /// overflow occurred.
    ///
    /// Equivalent to [`Self::checked_mul_ext`] with `Rounding::Round`.
    pub fn checked_mul<J, const Q: i32, const R: i32>(
        self,
        rhs: ConstScaleFpdec<J, Q>,
    ) -> Option<ConstScaleFpdec<I, R>>
    where
        J: FpdecInner,
    {
        self.checked_mul_ext(rhs, Rounding::Round, None)
    }

    /// Checked multiplication. Computes `self * rhs`, returning `None` if
    /// overflow occurred.
    ///
    /// The type of `rhs` can have different inner integer `J` and scale `Q`
    /// with `self`. The type of result must have the same inner integer `I`
    /// while have different scale `R`.
    ///
    /// If the scale of the result's type `R` is less than the sum of
    /// scales of the two multiplicands `P + Q`, then rounding operations
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
    pub fn checked_mul_ext<J, const Q: i32, const R: i32>(
        self,
        rhs: ConstScaleFpdec<J, Q>,
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<ConstScaleFpdec<I, R>>
    where
        J: FpdecInner,
    {
        self.0
            .checked_mul_ext(I::from(rhs.0)?, P + Q - R, rounding, cum_error)
            .map(ConstScaleFpdec)
    }

    /// Checked division. Computes `self / rhs`, returning `None` if
    /// division by 0 or overflow occurred.
    ///
    /// Equivalent to [`Self::checked_div_ext`] with `Rounding::Round`.
    pub fn checked_div<J, const Q: i32, const R: i32>(
        self,
        rhs: ConstScaleFpdec<J, Q>,
    ) -> Option<ConstScaleFpdec<I, R>>
    where
        J: FpdecInner,
    {
        self.checked_div_ext(rhs, Rounding::Round, None)
    }

    /// Checked division. Computes `self / rhs`, returning `None` if
    /// division by 0 or overflow occurred.
    ///
    /// The type of `rhs` can have different inner integer `J` and scale `Q`
    /// with `self`. The type of result must have the same inner integer `I`
    /// while have different scale `R`.
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
    pub fn checked_div_ext<J, const Q: i32, const R: i32>(
        self,
        rhs: ConstScaleFpdec<J, Q>,
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<ConstScaleFpdec<I, R>>
    where
        J: FpdecInner,
    {
        self.0
            .checked_div_ext(I::from(rhs.0)?, P - Q - R, rounding, cum_error)
            .map(ConstScaleFpdec)
    }

    /// Shrink to a lower scale.
    ///
    /// Equivalent to [`Self::shrink_to_with_rounding`] with `Rounding::Round`.
    pub fn shrink_to(self, retain_scale: i32) -> Self {
        self.shrink_to_with_rounding(retain_scale, Rounding::Round)
    }

    /// Shrink to a lower scale.
    ///
    /// The `retain_scale` argument specifies the number of scale to be
    /// retained, rather than the number to be reduced.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{ConstScaleFpdec, Rounding, fpdec};
    /// type Price = ConstScaleFpdec<i64, 8>;
    ///
    /// let price: Price = fpdec!(12.12345678);
    ///
    /// assert_eq!(price.shrink_to(6), fpdec!(12.123457)); // Rounding::Round as default
    ///
    /// assert_eq!(price.shrink_to_with_rounding(6, Rounding::Floor), fpdec!(12.123456));
    /// ```
    pub fn shrink_to_with_rounding(self, retain_scale: i32, rounding: Rounding) -> Self {
        Self(self.0.shrink_with_rounding(P - retain_scale, rounding))
    }
}

impl<I, const P: i32> fmt::Debug for ConstScaleFpdec<I, P>
where
    I: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Fpdec({},{})", self.0, P)
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
impl<I, const P: i32> fmt::Display for ConstScaleFpdec<I, P>
where
    I: FpdecInner + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.display_fmt(P, f)
    }
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
/// use core::str::FromStr;
/// use primitive_fixed_point_decimal::{ConstScaleFpdec, ParseError};
/// type Decimal = ConstScaleFpdec<i16, 4>;
///
/// assert_eq!(Decimal::from_str("1.23"), Decimal::try_from(1.23));
/// assert_eq!(Decimal::from_str("9999"), Err(ParseError::Overflow));
/// assert_eq!(Decimal::from_str("1.23456"), Err(ParseError::Precision));
/// ```
impl<I, const P: i32> FromStr for ConstScaleFpdec<I, P>
where
    I: FpdecInner,
    ParseError: From<<I as Num>::FromStrRadixErr>,
{
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, ParseError> {
        I::try_from_str(s, P).map(Self)
    }
}

impl<I, const P: i32> From<OobScaleFpdec<I>> for ConstScaleFpdec<I, P>
where
    I: FpdecInner,
{
    /// Convert from `OobScaleFpdec` with scale `P` to `ConstScaleFpdec`.
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
        impl<I, const P: i32> TryFrom<$from_int_type> for ConstScaleFpdec<I, P>
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
                if P > 0 {
                    // convert from type i to I first
                    let i2 = I::from(i).ok_or(ParseError::Overflow)?;
                    I::checked_from_int(i2, P).map(Self)
                } else {
                    // convert to fpdec inner first
                    let i2 = i.checked_from_int(P)?;
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
        impl<I, const P: i32> TryFrom<$float_type> for ConstScaleFpdec<I, P>
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
                let base: $float_type = 10.0;
                let inner_f = f * base.powi(P) as $float_type;
                I::$from_fn(inner_f.round())
                    .map(Self)
                    .ok_or(ParseError::Overflow)
            }
        }

        impl<I, const P: i32> From<ConstScaleFpdec<I, P>> for $float_type
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
            /// ```
            fn from(dec: ConstScaleFpdec<I, P>) -> Self {
                let base: $float_type = 10.0;
                dec.0.$to_fn().unwrap() / base.powi(P)
            }
        }
    };
}

convert_from_float!(f32, from_f32, to_f32);
convert_from_float!(f64, from_f64, to_f64);

impl<I, const P: i32> ops::Neg for ConstScaleFpdec<I, P>
where
    I: FpdecInner,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<I, const P: i32> ops::Add for ConstScaleFpdec<I, P>
where
    I: FpdecInner,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<I, const P: i32> ops::Sub for ConstScaleFpdec<I, P>
where
    I: FpdecInner,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<I, const P: i32> ops::AddAssign for ConstScaleFpdec<I, P>
where
    I: FpdecInner,
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<I, const P: i32> ops::SubAssign for ConstScaleFpdec<I, P>
where
    I: FpdecInner,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "serde")]
impl<I, const P: i32> Serialize for ConstScaleFpdec<I, P>
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
impl<'de, I, const P: i32> Deserialize<'de> for ConstScaleFpdec<I, P>
where
    I: FromPrimitive + FpdecInner,
    ParseError: From<<I as Num>::FromStrRadixErr>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use core::marker::PhantomData;
        use core::str::FromStr;
        use serde::de::{self, Visitor};

        struct ConstScaleFpdecVistor<I, const P: i32>(PhantomData<I>);

        macro_rules! visit_num {
            ($func_name:ident, $num_type:ty) => {
                fn $func_name<E: de::Error>(self, n: $num_type) -> Result<Self::Value, E> {
                    ConstScaleFpdec::try_from(n).map_err(|_| E::custom("decimal overflow"))
                }
            };
        }

        impl<'de, I, const P: i32> Visitor<'de> for ConstScaleFpdecVistor<I, P>
        where
            I: FromPrimitive + FpdecInner,
            ParseError: From<<I as Num>::FromStrRadixErr>,
        {
            type Value = ConstScaleFpdec<I, P>;

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
