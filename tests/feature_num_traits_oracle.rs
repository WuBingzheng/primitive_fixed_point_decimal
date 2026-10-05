//! The `num-traits-impls` feature against the oracle: `ToPrimitive`,
//! `FromPrimitive` and the `Checked*` family, for all ten inner types.
//!
//! `ToPrimitive` truncates toward zero (documented) and `FromPrimitive` is exact
//! or `None`; both are specified here from `BigInt` arithmetic.
mod common;
use common::*;

use num_bigint::BigInt;
use num_traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedSub, FromPrimitive, Num, One,
    Signed, ToPrimitive, Zero,
};
use primitive_fixed_point_decimal::{ConstScaleFpdec as C, ParseError};

fn trunc(m: &BigInt, s: i32) -> BigInt {
    if s > 0 { m / pow10(s as u32) } else { m * pow10(s.unsigned_abs()) }
}

fn check_to_primitive<I: Int, const S: i32>(f: &mut Failures) {
    for m in interesting::<I>() {
        let d = C::<I, S>::from_mantissa(m);
        let v = trunc(&m.big(), S);
        let what = || format!("{m:?} @{S}");
        f.check(|| format!("{} to_i64 S={S}", I::NAME), what, guarded(|| d.to_i64()), v.to_i64());
        f.check(|| format!("{} to_u64 S={S}", I::NAME), what, guarded(|| d.to_u64()), v.to_u64());
        f.check(|| format!("{} to_i128 S={S}", I::NAME), what, guarded(|| d.to_i128()), v.to_i128());
        f.check(|| format!("{} to_u128 S={S}", I::NAME), what, guarded(|| d.to_u128()), v.to_u128());
        // the narrower ones are derived from to_i64/to_u64
        f.check(|| format!("{} to_i32 S={S}", I::NAME), what, guarded(|| d.to_i32()), v.to_i32());
        f.check(|| format!("{} to_u8 S={S}", I::NAME), what, guarded(|| d.to_u8()), v.to_u8());
        f.check(|| format!("{} to_f64 finite S={S}", I::NAME), what, guarded(|| d.to_f64().is_some_and(f64::is_finite)), true);
    }
}

fn check_from_primitive<I: Int + FromPrimitive, const S: i32>(f: &mut Failures) {
    let w = |r: Result<BigInt, ParseError>| r.ok().map(|m| m);
    for v in interesting::<i64>() {
        let want = w(spec_from_int::<I>(&v.big(), S));
        f.check(|| format!("{} from_i64 S={S}", I::NAME), || format!("{v}"),
            guarded(|| <C<I, S>>::from_i64(v).map(|d| d.mantissa().big())), want);
    }
    for v in interesting::<u64>() {
        let want = w(spec_from_int::<I>(&v.big(), S));
        f.check(|| format!("{} from_u64 S={S}", I::NAME), || format!("{v}"),
            guarded(|| <C<I, S>>::from_u64(v).map(|d| d.mantissa().big())), want);
    }
    for v in interesting::<i128>() {
        let want = w(spec_from_int::<I>(&v.big(), S));
        f.check(|| format!("{} from_i128 S={S}", I::NAME), || format!("{v}"),
            guarded(|| <C<I, S>>::from_i128(v).map(|d| d.mantissa().big())), want);
    }
    for v in interesting::<u128>() {
        let want = w(spec_from_int::<I>(&v.big(), S));
        f.check(|| format!("{} from_u128 S={S}", I::NAME), || format!("{v}"),
            guarded(|| <C<I, S>>::from_u128(v).map(|d| d.mantissa().big())), want);
    }
}

macro_rules! per_type {
    ($($t:ty: $name:ident),*) => { $(
        #[test]
        fn $name() {
            let mut f = Failures::default();
            check_to_primitive::<$t, -40>(&mut f);
            check_to_primitive::<$t, -3>(&mut f);
            check_to_primitive::<$t, 0>(&mut f);
            check_to_primitive::<$t, 2>(&mut f);
            check_to_primitive::<$t, 12>(&mut f);
            check_to_primitive::<$t, 40>(&mut f);
            check_from_primitive::<$t, -40>(&mut f);
            check_from_primitive::<$t, -3>(&mut f);
            check_from_primitive::<$t, 0>(&mut f);
            check_from_primitive::<$t, 2>(&mut f);
            check_from_primitive::<$t, 40>(&mut f);
            f.finish(concat!("num-traits conversions ", stringify!($t)));
        }
    )* };
}
per_type!(i8: conv_i8, i16: conv_i16, i32: conv_i32, i64: conv_i64, i128: conv_i128,
          u8: conv_u8, u16: conv_u16, u32: conv_u32, u64: conv_u64, u128: conv_u128);

/// `to_i64` of a value below 1 used to be `None` when `10^S` did not fit the inner
/// type (`u16` at scale 5), and so was zero at a scale whose power did not fit.
#[test]
fn regress_to_primitive_below_one_and_zero() {
    assert_eq!(C::<u16, 5>::from_mantissa(65535).to_i64(), Some(0)); // 0.65535
    assert_eq!(C::<u16, 9>::from_mantissa(7).to_u64(), Some(0));
    assert_eq!(C::<i8, 40>::from_mantissa(-5).to_i128(), Some(0));
    assert_eq!(C::<i8, -3>::from_mantissa(0).to_i64(), Some(0)); // 10^3 does not fit i8
    assert_eq!(C::<i8, -3>::from_mantissa(1).to_i64(), Some(1000)); // ...but the result does
    assert_eq!(C::<i8, -3>::from_mantissa(1).to_i128(), Some(1000));
    assert_eq!(C::<i128, -3>::from_mantissa(i128::MAX).to_i128(), None);
    assert_eq!(C::<i32, 2>::from_mantissa(-299).to_i64(), Some(-2)); // truncates toward zero
}

/// The provided `from_i128`/`from_u128` go through `i64`/`u64`, which refused every
/// value above 2^63 even for a 128-bit inner type.
#[test]
fn regress_from_128bit_primitives() {
    assert_eq!(C::<i128, 0>::from_i128(i128::MAX).map(|d| d.mantissa()), Some(i128::MAX));
    assert_eq!(C::<i128, 0>::from_i128(i128::MIN).map(|d| d.mantissa()), Some(i128::MIN));
    assert_eq!(C::<u128, 0>::from_u128(u128::MAX).map(|d| d.mantissa()), Some(u128::MAX));
    assert_eq!(C::<i128, 2>::from_i128(1 << 100).map(|d| d.mantissa()), Some(100 << 100));
    assert_eq!(C::<i128, 2>::from_i128(i128::MAX), None);
    assert_eq!(C::<i64, 0>::from_i128(1 << 63), None);
    assert_eq!(C::<i64, 0>::from_u128(u64::MAX as u128 + 1), None);
}

#[test]
fn checked_traits_agree_with_the_inherent_methods() {
    type X = C<i16, 2>;
    let (max, min, one) = (X::MAX, X::MIN, X::from_mantissa(1));
    assert_eq!(CheckedAdd::checked_add(&max, &one), None);
    assert_eq!(CheckedSub::checked_sub(&min, &one), None);
    assert_eq!(CheckedAdd::checked_add(&max, &X::ZERO), Some(max));
    assert_eq!(CheckedNeg::checked_neg(&min), None);
    assert_eq!(CheckedNeg::checked_neg(&max).map(|v| v.mantissa()), Some(-i16::MAX));
    assert_eq!(CheckedDiv::checked_div(&one, &X::ZERO), None);
    assert_eq!(CheckedMul::checked_mul(&X::from_mantissa(150), &X::from_mantissa(200)).map(|v| v.mantissa()), Some(300));
    assert_eq!(CheckedMul::checked_mul(&max, &X::from_mantissa(200)), None);
    // `MIN % -1` is 0, through the trait too (this used to trap)
    assert_eq!(CheckedRem::checked_rem(&min, &X::from_mantissa(-1)), Some(X::ZERO));
    assert_eq!(CheckedRem::checked_rem(&min, &X::ZERO), None);
    assert_eq!(CheckedRem::checked_rem(&X::from_mantissa(7), &X::from_mantissa(-3)).map(|v| v.mantissa()), Some(1));
}

#[test]
fn identities_bounds_and_radix() {
    type X = C<i32, 3>;
    assert_eq!(X::zero(), X::ZERO);
    assert!(X::zero().is_zero() && !X::one().is_zero());
    assert_eq!(X::one().mantissa(), 1000);
    assert_eq!(X::min_value(), X::MIN);
    assert_eq!(X::max_value(), X::MAX);
    assert_eq!(X::from_str_radix("1.5", 10).map(|v| v.mantissa()), Ok(1500));
    for radix in [2, 8, 16, 36] {
        assert_eq!(X::from_str_radix("1", radix), Err(ParseError::Invalid), "radix {radix}");
    }
    assert_eq!(C::<u8, 1>::from_str_radix("0", 10).map(|v| v.mantissa()), Ok(0));
}

#[test]
fn signed_methods() {
    type X = C<i32, 2>;
    let v = |m: i32| X::from_mantissa(m);
    assert_eq!(v(-5).signum(), -X::one());
    assert_eq!(v(0).signum(), X::ZERO);
    assert_eq!(v(7).signum(), X::one());
    assert_eq!(Signed::abs(&v(-5)), v(5));
    assert_eq!(v(5).abs_sub(&v(3)), v(2));
    assert_eq!(v(3).abs_sub(&v(5)), X::ZERO);
    assert_eq!(v(3).abs_sub(&v(3)), X::ZERO);
    assert!(v(5).is_positive() && !v(5).is_negative() && v(-5).is_negative() && !v(0).is_positive());
}

/// `One::one()` needs `10^S` to fit; otherwise it is a loud panic, not a wrong value.
#[test]
#[should_panic(expected = "scale out of range")]
fn one_with_unrepresentable_unit_panics() {
    let _ = C::<i8, 4>::one(); // 10000 does not fit i8
}
#[test]
#[should_panic(expected = "scale out of range")]
fn one_with_negative_scale_panics() {
    let _ = C::<i32, -2>::one(); // the unit 1 is finer than the scale 100
}
