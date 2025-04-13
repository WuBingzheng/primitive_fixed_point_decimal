use crate::fpdec_inner::FpdecInner;
use crate::ParseError;
use int_div_cum_error::{Rounding, checked_divide};
use num_traits::{Num, cast::{FromPrimitive, ToPrimitive}};
use std::fmt;


/// Static-precision fixed-point decimal.
///
/// `I` is the inner integer type, could be `i8`, `i16`, `i32`, `i64`, or `i128`.
///
/// `P` is the static precision.
///
/// For example, `StaticPrecFpdec<i64, 4>` means using `i64` as the inner
/// integer with about 18 significant digits, and having `4` fraction precision.
///
/// See [the module-level documentation](super) for more information.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct StaticPrecFpdec<I, const P: i32>(pub(crate) I);

impl<I, const P: i32> StaticPrecFpdec<I, P>
where I: FpdecInner
{
    crate::none_prec_common::define_none_prec_common!();

    /// Checked multiplication. Computes `self * rhs`, returning `None` if
    /// overflow occurred.
    ///
    /// Equivalent to [`Self::checked_mul_ext`] with `Rounding::Round`.
    pub fn checked_mul<J, const Q: i32, const R: i32>(
        self,
        rhs: StaticPrecFpdec<J, Q>,
    ) -> Option<StaticPrecFpdec<I, R>>
        where J: ToPrimitive
    {
        self.checked_mul_ext(rhs, Rounding::Round, None)
    }

    /// Checked multiplication. Computes `self * rhs`, returning `None` if
    /// overflow occurred.
    ///
    /// The type of `rhs` can have different inner integer `J` and precision `Q`
    /// with `self`. The type of result must have the same inner integer `I`
    /// while have different precision `R`.
    ///
    /// If the precision of the result's type `R` is less than the sum of
    /// precisions of the two multiplicands `P + Q`, then rounding operations
    /// are required and precision may be lost.
    /// You can specify the rounding type and cumulative error.
    ///
    /// See [the module-level documentation](super) for more information and
    /// examples about cumulative error.
    ///
    /// # Examples
    /// 
    /// ```
    /// use primitive_fixed_point_decimal::{StaticPrecFpdec, Rounding};
    /// type Balance = StaticPrecFpdec<i64, 2>;
    /// type FeeRate = StaticPrecFpdec<i16, 4>; // different types
    ///
    /// let balance = Balance::try_from(12.60).unwrap();
    /// let rate = FeeRate::try_from(0.01).unwrap();
    ///
    /// // calculate fee 3 times with same arguments, with `cum_error`.
    /// // but have different results: 0.13, 0.13 and 0.12
    /// let mut cum_error: i64 = 0;
    /// assert_eq!(balance.checked_mul_ext(rate, Rounding::Ceiling, Some(&mut cum_error)),
    ///     Balance::try_from(0.13).ok());
    /// assert_eq!(balance.checked_mul_ext(rate, Rounding::Ceiling, Some(&mut cum_error)),
    ///     Balance::try_from(0.13).ok());
    /// assert_eq!(balance.checked_mul_ext(rate, Rounding::Ceiling, Some(&mut cum_error)),
    ///     Balance::try_from(0.12).ok());
    /// ```
    pub fn checked_mul_ext<J, const Q: i32, const R: i32>(
        self,
        rhs: StaticPrecFpdec<J, Q>,
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<StaticPrecFpdec<I, R>>
        where J: ToPrimitive
    {
        self.0.checked_mul_ext(I::from(rhs.0)?, P + Q - R, rounding, cum_error)
            .map(StaticPrecFpdec::<I, R>)
    }

    /// Checked division. Computes `self / rhs`, returning `None` if
    /// division by 0 or overflow occurred.
    ///
    /// Equivalent to [`Self::checked_div_ext`] with `Rounding::Round`.
    pub fn checked_div<J, const Q: i32, const R: i32>(
        self,
        rhs: StaticPrecFpdec<J, Q>,
    ) -> Option<StaticPrecFpdec<I, R>>
        where J: ToPrimitive
    {
        self.checked_div_ext(rhs, Rounding::Round, None)
    }

    /// Checked division. Computes `self / rhs`, returning `None` if
    /// division by 0 or overflow occurred.
    ///
    /// The type of `rhs` can have different inner integer `J` and precision `Q`
    /// with `self`. The type of result must have the same inner integer `I`
    /// while have different precision `R`.
    ///
    /// You can specify the rounding type and cumulative error.
    /// See [the module-level documentation](super) for more information and
    /// examples about cumulative error.
    ///
    /// # Examples
    /// 
    /// ```
    /// use primitive_fixed_point_decimal::{StaticPrecFpdec, Rounding};
    /// type Balance = StaticPrecFpdec<i64, 2>;
    /// type FeeRate = StaticPrecFpdec<i16, 4>; // different types
    ///
    /// let rate = FeeRate::try_from(0.03).unwrap();
    /// let fee = Balance::try_from(0.13).unwrap();
    ///
    /// assert_eq!(fee.checked_div_ext(rate, Rounding::Ceiling, None),
    ///     Balance::try_from(4.34).ok());
    /// ```
    pub fn checked_div_ext<J, const Q: i32, const R: i32>(
        self,
        rhs: StaticPrecFpdec<J, Q>,
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<StaticPrecFpdec<I, R>>
        where J: ToPrimitive
    {
        self.0.checked_div_ext(I::from(rhs.0)?, P - Q - R, rounding, cum_error)
            .map(StaticPrecFpdec::<I, R>)
    }

    /// Shrink to a lower precision.
    ///
    /// Equivalent to [`Self::shrink_to_with_rounding`] with `Rounding::Round`.
    pub fn shrink_to(self, retain_precision: i32) -> Self {
        self.shrink_to_with_rounding(retain_precision, Rounding::Round)
    }

    /// Shrink to a lower precision.
    ///
    /// The `retain_precision` argument specifies the number of precision to be
    /// retained, rather than the number to be reduced.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{StaticPrecFpdec, Rounding};
    /// type Price = StaticPrecFpdec<i64, 8>;
    ///
    /// let price = Price::try_from(12.12345678).unwrap();
    ///
    /// assert_eq!(price.shrink_to(6),
    ///     Price::try_from(12.123457).unwrap()); // Rounding::Round as default
    ///
    /// assert_eq!(price.shrink_to_with_rounding(6, Rounding::Floor),
    ///     Price::try_from(12.123456).unwrap());
    /// ```
    pub fn shrink_to_with_rounding(
        self,
        retain_precision: i32,
        rounding: Rounding,
    ) -> Self {
        Self(self.0.shrink_with_rounding(P - retain_precision, rounding))
    }
}

impl<I, const P: i32> fmt::Debug for StaticPrecFpdec<I, P>
where I: fmt::Display
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
/// use primitive_fixed_point_decimal::{StaticPrecFpdec, ParseError};
/// type Decimal = StaticPrecFpdec<i16, 4>;
/// type BigPrec = StaticPrecFpdec<i16, 8>;
/// type NegPrec = StaticPrecFpdec<i16, -2>;
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
/// assert_eq!(format!("{}", NegPrec::try_from(-3276800.0_f64).unwrap()), String::from("-3276800"));
/// ```
impl<I, const P: i32> fmt::Display for StaticPrecFpdec<I, P>
where I: FpdecInner + fmt::Display
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
/// use std::str::FromStr;
/// use primitive_fixed_point_decimal::{StaticPrecFpdec, ParseError};
/// type Decimal = StaticPrecFpdec<i16, 4>;
///
/// assert_eq!(Decimal::from_str("1.23"), Decimal::try_from(1.23));
/// assert_eq!(Decimal::from_str("9999"), Err(ParseError::Overflow));
/// assert_eq!(Decimal::from_str("1.23456"), Err(ParseError::Precision));
/// ```
impl<I, const P: i32> std::str::FromStr for StaticPrecFpdec<I, P>
where I: FpdecInner,
      ParseError: From<<I as Num>::FromStrRadixErr>
{
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, ParseError> {
        I::try_from_str(s, P).map(Self)
    }
}

macro_rules! convert_static_from_int {
    ($from_int_type:ty, $from_int_fn:ident) => {
        impl<I, const P: i32> TryFrom<$from_int_type> for StaticPrecFpdec<I, P>
            where I: FromPrimitive + FpdecInner
        {
            type Error = ParseError;

            /// Convert from integer. Returning error if overflow occurred
            /// or lossing precision under `precision < 0`.
            fn try_from(i: $from_int_type) -> Result<Self, Self::Error> {
                let i2 = I::$from_int_fn(i).ok_or(ParseError::Overflow)?;
                I::checked_from_int(i2, P).map(Self)
            }
        }
    }
}
convert_static_from_int!(i8, from_i8);
convert_static_from_int!(u8, from_u8);
convert_static_from_int!(i16, from_i16);
convert_static_from_int!(u16, from_u16);
convert_static_from_int!(i32, from_i32);
convert_static_from_int!(u32, from_u32);
convert_static_from_int!(i64, from_i64);
convert_static_from_int!(u64, from_u64);
convert_static_from_int!(i128, from_i128);
convert_static_from_int!(u128, from_u128);

macro_rules! convert_static_from_float {
    ($float_type:ty, $from_fn:ident, $to_fn:ident) => {
        impl<I, const P: i32> TryFrom<$float_type> for StaticPrecFpdec<I, P>
            where I: FromPrimitive + FpdecInner
        {
            type Error = ParseError;

            /// Convert from float type. Returning error if overflow occurred.
            ///
            /// Since it's hard for the float types to represent decimal fraction
            /// exactly, so this method always rounds the float number into
            /// StaticPrecFpdec.
            fn try_from(f: $float_type) -> Result<Self, Self::Error> {
                let base: $float_type = 10.0;
                let inner_f = f * base.powi(P) as $float_type;
                I::$from_fn(inner_f.round())
                    .map(Self)
                    .ok_or(ParseError::Overflow)
            }
        }

        impl<I, const P: i32> From<StaticPrecFpdec<I, P>> for $float_type
            where I: ToPrimitive + FpdecInner
        {
            /// Convert into float type.
            fn from(dec: StaticPrecFpdec<I, P>) -> Self {
                let base: $float_type = 10.0;
                dec.0.$to_fn().unwrap() / base.powi(P)
            }
        }
    }
}

convert_static_from_float!(f32, from_f32, to_f32);
convert_static_from_float!(f64, from_f64, to_f64);

impl<I, const P: i32> std::ops::Neg for StaticPrecFpdec<I, P>
where I: FpdecInner
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<I, const P: i32> std::ops::Add for StaticPrecFpdec<I, P>
where I: FpdecInner
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<I, const P: i32> std::ops::Sub for StaticPrecFpdec<I, P>
where I: FpdecInner
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<I, const P: i32> std::ops::AddAssign for StaticPrecFpdec<I, P>
where I: FpdecInner
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<I, const P: i32> std::ops::SubAssign for StaticPrecFpdec<I, P>
where I: FpdecInner
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}


#[cfg(feature="serde")]
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[cfg(feature="serde")]
impl<I, const P: i32> Serialize for StaticPrecFpdec<I, P>
where I: ToPrimitive + FpdecInner + fmt::Display
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        // XXX how to selete dump type?
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            Into::<f64>::into(*self).serialize(serializer)
        }
    }
}

#[cfg(feature="serde")]
impl<'de, I, const P: i32> Deserialize<'de> for StaticPrecFpdec<I, P>
where I: FromPrimitive + FpdecInner
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        use serde::de::{self, Visitor};
        use std::str::FromStr;
        use std::marker::PhantomData;

        struct StaticPrecFpdecVistor<I, const P: i32>(PhantomData<I>);

        macro_rules! visit_num {
            ($func_name:ident, $num_type:ty) => {
                fn $func_name<E: de::Error>(self, n: $num_type) -> Result<Self::Value, E> {
                    StaticPrecFpdec::<I, P>::try_from(n)
                        .map_err(|_| E::custom("decimal overflow"))
                }
            }
        }

        impl<'de, I, const P: i32> Visitor<'de> for StaticPrecFpdecVistor<I, P>
        where I: FromPrimitive + FpdecInner
        {
            type Value = StaticPrecFpdec<I, P>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "decimal")
            }

            fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                StaticPrecFpdec::<I, P>::from_str(s)
                    .map_err(|e| E::custom(format!("decimal {:?}", e)))
            }

            visit_num!(visit_f32, f32);
            visit_num!(visit_f64, f64);
            visit_num!(visit_i8, i8);
            visit_num!(visit_u8, u8);
            visit_num!(visit_i16, i16);
            visit_num!(visit_u16, u16);
            visit_num!(visit_i32, i32);
            visit_num!(visit_u32, u32);
            visit_num!(visit_i64, i64);
            visit_num!(visit_u64, u64);
            visit_num!(visit_i128, i128);
            visit_num!(visit_u128, u128);
        }

        deserializer.deserialize_any(StaticPrecFpdecVistor::<I, P>(PhantomData::<I>))
    }
}
