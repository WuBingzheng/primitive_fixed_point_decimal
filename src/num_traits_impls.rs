//! `num-traits` implementations for [`ConstScaleFpdec`].
//!
//! Enabled by the `num-traits-impls` feature. (The feature is spelled with a
//! suffix because `num-traits` itself is an unconditional dependency of this
//! crate -- these impls are the optional part, not the dependency.)
//!
//! # Which traits, and why not the others
//!
//! Implemented: [`Zero`], [`One`], [`Num`], [`Bounded`], [`Signed`], the
//! `Checked*` family, [`ToPrimitive`] and [`FromPrimitive`]. Together these are
//! what generic numeric code actually bounds on.
//!
//! NOT implemented: `NumCast` and `PrimInt`. `ConstScaleFpdec` is not an
//! integer, and `NumCast::from` would have to silently pick a rounding mode --
//! this crate makes rounding an explicit argument everywhere else and should not
//! quietly stop doing so here.
//!
//! # A note on method resolution
//!
//! [`CheckedMul::checked_mul`] takes `&self, &Self` and fixes the result scale
//! to `S`. The *inherent* [`ConstScaleFpdec::checked_mul`] is more general
//! (`<J, S2, SR>`) and takes `self` by value. Inherent methods win resolution,
//! so `a.checked_mul(b)` keeps calling the inherent one; the trait method is
//! reached through the trait, e.g. `CheckedMul::checked_mul(&a, &b)`.
//!
//! Like the inherent operators, the trait `Mul`/`Div` used by [`Num`] round with
//! [`Rounding::Round`] and panic on overflow.

use crate::const_scale_fpdec::ConstScaleFpdec;
use crate::fpdec_inner::FpdecInner;
use crate::ParseError;

use core::num::ParseIntError;
use core::str::FromStr;
use num_traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedSub, FromPrimitive,
    Num, One, Signed, ToPrimitive, Zero,
};

impl<I, const S: i32> Zero for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

impl<I, const S: i32> One for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    /// The decimal `1`, i.e. mantissa `10^S`.
    ///
    /// # Panics
    ///
    /// If scale `S` is negative or too large for `I` to represent `1`, e.g.
    /// `ConstScaleFpdec<i8, 4>` (which would need a mantissa of 10000).
    fn one() -> Self {
        let exp = usize::try_from(S)
            .ok()
            .and_then(I::get_exp)
            .expect("scale out of range for One::one()");
        Self::from_mantissa(exp)
    }
}

impl<I, const S: i32> Bounded for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    fn min_value() -> Self {
        Self::MIN
    }
    fn max_value() -> Self {
        Self::MAX
    }
}

impl<I, const S: i32> Num for ConstScaleFpdec<I, S>
where
    I: FpdecInner + FromPrimitive + Num<FromStrRadixErr = ParseIntError>,
{
    type FromStrRadixErr = ParseError;

    /// Only radix 10 is meaningful for a *decimal*; anything else is
    /// [`ParseError::Invalid`] rather than a silently wrong answer.
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        if radix != 10 {
            return Err(ParseError::Invalid);
        }
        Self::from_str(s)
    }
}

/// Only for signed inner integers -- `ConstScaleFpdec` also supports `u8`..`u128`,
/// and `abs`/`signum`/`Neg` are meaningless there.
impl<I, const S: i32> Signed for ConstScaleFpdec<I, S>
where
    I: FpdecInner + FromPrimitive + Signed + Num<FromStrRadixErr = ParseIntError>,
{
    fn abs(&self) -> Self {
        ConstScaleFpdec::abs(*self)
    }
    /// `self - other`, floored at zero.
    fn abs_sub(&self, other: &Self) -> Self {
        if self <= other {
            Self::ZERO
        } else {
            *self - *other
        }
    }
    fn signum(&self) -> Self {
        if self.is_pos() {
            Self::one()
        } else if self.is_neg() {
            -Self::one()
        } else {
            Self::ZERO
        }
    }
    fn is_positive(&self) -> bool {
        self.is_pos()
    }
    fn is_negative(&self) -> bool {
        self.is_neg()
    }
}

macro_rules! checked_binop {
    ($trait:ident, $method:ident) => {
        impl<I, const S: i32> $trait for ConstScaleFpdec<I, S>
        where
            I: FpdecInner,
        {
            fn $method(&self, rhs: &Self) -> Option<Self> {
                ConstScaleFpdec::$method(*self, *rhs)
            }
        }
    };
}
checked_binop!(CheckedAdd, checked_add);
checked_binop!(CheckedSub, checked_sub);
checked_binop!(CheckedRem, checked_rem);

// `checked_mul`/`checked_div` are generic over the result scale; pin it to `S`.
impl<I, const S: i32> CheckedMul for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    fn checked_mul(&self, rhs: &Self) -> Option<Self> {
        ConstScaleFpdec::checked_mul::<I, S, S>(*self, *rhs)
    }
}

impl<I, const S: i32> CheckedDiv for ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    fn checked_div(&self, rhs: &Self) -> Option<Self> {
        ConstScaleFpdec::checked_div::<I, S, S>(*self, *rhs)
    }
}

/// Signed inner integers only (see [`Signed`]).
impl<I, const S: i32> CheckedNeg for ConstScaleFpdec<I, S>
where
    I: FpdecInner + Signed,
{
    fn checked_neg(&self) -> Option<Self> {
        // `-MIN` is not representable in two's complement.
        if *self == Self::MIN {
            None
        } else {
            Some(-*self)
        }
    }
}

/// Conversions to primitives **truncate toward zero** on the integer methods --
/// `to_i64` of `2.9` is `2` -- matching `as` casts and `f64 as i64`.
impl<I, const S: i32> ToPrimitive for ConstScaleFpdec<I, S>
where
    I: FpdecInner + ToPrimitive,
    f64: From<ConstScaleFpdec<I, S>>,
{
    fn to_i64(&self) -> Option<i64> {
        self.trunc_to_int()?.to_i64()
    }
    fn to_u64(&self) -> Option<u64> {
        self.trunc_to_int()?.to_u64()
    }
    fn to_i128(&self) -> Option<i128> {
        self.trunc_to_int()?.to_i128()
    }
    fn to_u128(&self) -> Option<u128> {
        self.trunc_to_int()?.to_u128()
    }
    /// Lossy above ~15.9 significant digits, as any `f64` conversion is.
    fn to_f64(&self) -> Option<f64> {
        Some(f64::from(*self))
    }
}

impl<I, const S: i32> ConstScaleFpdec<I, S>
where
    I: FpdecInner,
{
    /// The integer part, truncated toward zero, as the inner integer type.
    fn trunc_to_int(&self) -> Option<I> {
        match S {
            0 => Some(self.mantissa()),
            s if s > 0 => Some(self.mantissa() / I::get_exp(s as usize)?),
            s => self.mantissa().checked_mul(&I::get_exp((-s) as usize)?),
        }
    }
}

/// Conversions from primitives are **exact or nothing**: they return `None`
/// rather than rounding, so a value that does not fit the scale is rejected
/// instead of silently changed.
impl<I, const S: i32> FromPrimitive for ConstScaleFpdec<I, S>
where
    I: FpdecInner + FromPrimitive,
    ConstScaleFpdec<I, S>: TryFrom<i64> + TryFrom<u64> + TryFrom<f64>,
{
    fn from_i64(n: i64) -> Option<Self> {
        Self::try_from(n).ok()
    }
    fn from_u64(n: u64) -> Option<Self> {
        Self::try_from(n).ok()
    }
    fn from_f64(n: f64) -> Option<Self> {
        Self::try_from(n).ok()
    }
}
