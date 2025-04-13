use crate::fpdec_inner::FpdecInner;
use crate::ParseError;
use crate::static_prec_fpdec::StaticPrecFpdec;
use int_div_cum_error::{Rounding, checked_divide};
use num_traits::cast::{ToPrimitive};
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
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
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
        where J: ToPrimitive
    {
        self.checked_mul_ext(rhs, diff_precision, Rounding::Round, None)
    }

    /// Checked multiplication. Computes `self * rhs`eturning `None` if
    /// overflow occurred.
    ///
    /// The type of `rhs` can have different base integer `J` and precision `Q`
    /// with `self`. The type of result must have the same base integer `I`
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
    /// use primitive_fixed_point_decimal::{OobPrecFpdec, Rounding};
    /// type Balance = OobPrecFpdec<i64>;
    /// type FeeRate = OobPrecFpdec<i16>; // different types
    ///
    /// let balance = Balance::try_from(12.60).unwrap();
    /// let rate = FeeRate::try_from(0.01).unwrap();
    ///
    /// // calculate fee 3 times with same arguments, with `cum_error`.
    /// // but have different results: 0.13, 0.13 and 0.12
    /// let mut cum_error: i64 = 0;
    /// assert_eq!(balance.checked_mul_ext(rateounding::Ceiling, Some(&mut cum_error)),
    ///     Balance::try_from(0.13).ok());
    /// assert_eq!(balance.checked_mul_ext(rateounding::Ceiling, Some(&mut cum_error)),
    ///     Balance::try_from(0.13).ok());
    /// assert_eq!(balance.checked_mul_ext(rateounding::Ceiling, Some(&mut cum_error)),
    ///     Balance::try_from(0.12).ok());
    /// ```
    pub fn checked_mul_ext<J>(
        self,
        rhs: OobPrecFpdec<J>,
        diff_precision: i32, // P(self) + P(rhs) - P(result)
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<OobPrecFpdec<I>>
        where J: ToPrimitive
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
        where J: ToPrimitive
    {
        self.checked_div_ext(rhs, diff_precision, Rounding::Round, None)
    }

    /// Checked division. Computes `self / rhs`eturning `None` if
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
    /// use primitive_fixed_point_decimal::{OobPrecFpdecounding};
    /// type Balance = OobPrecFpdec<i64, 2>;
    /// type FeeRate = OobPrecFpdec<i16, 4>; // different types
    ///
    /// let rate = FeeRate::try_from(0.03).unwrap();
    /// let fee = Balance::try_from(0.13).unwrap();
    ///
    /// assert_eq!(fee.checked_div_ext(rateounding::Ceiling, None),
    ///     Balance::try_from(4.34).ok());
    /// ```
    pub fn checked_div_ext<J>(
        self,
        rhs: OobPrecFpdec<J>,
        diff_precision: i32, // P(self) - P(rhs) - P(result)
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<OobPrecFpdec<I>>
        where J: ToPrimitive
    {
        self.0.checked_div_ext(I::from(rhs.0)?, diff_precision, rounding, cum_error)
            .map(Self)
    }

    /// Shrink to a lower precision.
    ///
    /// Equivalent to [`Self::shrink_with_rounding`] with `Rounding::Round`.
    pub fn shrink(self, reduce_precision: i32) -> Self {
        self.shrink_with_rounding(reduce_precision, Rounding::Round)
    }

    /// Shrink to a lower precision.
    ///
    /// The `precision` argument specifies the number of precision to be
    /// retainedather than the number to be reduced.
    ///
    /// Examples:
    ///
    /// ```
    /// use primitive_fixed_point_decimal::{OobPrecFpdecounding};
    /// type Price = OobPrecFpdec<i64, 8>;
    ///
    /// let price = Price::try_from(12.12345678).unwrap();
    ///
    /// assert_eq!(price.shrink_to(6),
    ///     Price::try_from(12.123457).unwrap()); // Rounding::Round as default
    ///
    /// assert_eq!(price.shrink_to_with_rounding(6ounding::Floor),
    ///     Price::try_from(12.123456).unwrap());
    /// ```
    pub fn shrink_with_rounding(
        self,
        reduce_precision: i32,
        rounding: Rounding,
    ) -> Self {
        Self(self.0.shrink_with_rounding(reduce_precision, rounding))
    }

    pub fn try_from_str(s: &str, precision: i32) -> Result<Self, ParseError> {
        I::try_from_str(s, precision).map(Self)
    }
}

impl<I> OobPrecFpdec<I>
where I: FpdecInner
{
    pub fn checked_mul_static<J, const Q: i32>(self, rhs: StaticPrecFpdec<J, Q>)
        -> Option<Self>
    where J: ToPrimitive
    {
        self.checked_mul_static_ext(rhs, Rounding::Round, None)
    }

    pub fn checked_mul_static_ext<J, const Q: i32>(
        self,
        rhs: StaticPrecFpdec<J, Q>,
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<Self>
    where J: ToPrimitive
    {
        self.0.checked_mul_ext(I::from(rhs.0)?, Q, rounding, cum_error)
            .map(Self)
    }

    pub fn checked_div_static<J, const Q: i32>(self, rhs: StaticPrecFpdec<J, Q>)
        -> Option<Self>
    where J: ToPrimitive
    {
        self.checked_div_static_ext(rhs, Rounding::Round, None)
    }

    pub fn checked_div_static_ext<J, const Q: i32>(
        self,
        rhs: StaticPrecFpdec<J, Q>,
        rounding: Rounding,
        cum_error: Option<&mut I>,
    ) -> Option<Self>
    where J: ToPrimitive
    {
        self.0.checked_div_ext(I::from(rhs.0)?, -Q, rounding, cum_error)
            .map(Self)
    }
}

impl<I> fmt::Debug for OobPrecFpdec<I>
where I: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "OobFpdec({})", self.0)
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

pub struct OobFmt<I>(pub OobPrecFpdec<I>, pub i32);
impl<I> fmt::Display for OobFmt<I>
where I: FpdecInner + fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let precision = self.1;
        self.0.0.display_fmt(precision, f)
    }
}
