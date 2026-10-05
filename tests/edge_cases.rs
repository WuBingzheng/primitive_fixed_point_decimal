//! Edge cases that the arithmetic oracle (`tests/oracle.rs`) and the text oracle
//! (`tests/parse_format.rs`) do not reach: the type constants, `%`, integer and
//! float conversions, overflow semantics, `ConstScaleFpdec` against the same
//! spec, documented panics, and one named regression per defect that the
//! review found. Each regression names the value that broke the old code.
//!
//! Debug builds panic on overflow and release builds wrap; the tests that depend
//! on it ask the compiler's setting at run time (`overflow_checks()`), so run the
//! suite both ways: `cargo test` and `cargo test --release`.
mod common;
use common::*;

use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};
use primitive_fixed_point_decimal::{
    ConstScaleFpdec as C, OobFmt, OobScaleFpdec as D, ParseError, Rounding,
};
use std::hint::black_box;

fn overflow_checks() -> bool {
    guarded(|| black_box(255u8) + black_box(1u8)).is_err()
}

fn sample<T: Int>(stride_div: usize) -> Vec<T> {
    let all = interesting::<T>();
    let stride = (all.len() / stride_div).max(1);
    all.into_iter().step_by(stride).collect()
}

// ---------------------------------------------------------------------------
// 1. The constants every algorithm leans on
// ---------------------------------------------------------------------------

fn invariants<T: Int>() {
    let d = T::DIGITS;
    assert_eq!(T::MAX.big(), T::max_big(), "{}", T::NAME);
    assert_eq!(T::MIN.big(), T::min_big(), "{}", T::NAME);
    assert_eq!(T::max_big().to_string().len() as u32, d + 1, "{} DIGITS", T::NAME);
    assert_eq!(T::MAX_POWERS.big(), pow10(d), "{} MAX_POWERS", T::NAME);
    assert_eq!(T::TEN.big(), BigInt::from(10));
    assert_eq!(T::HUNDRED.big(), BigInt::from(100));
    for i in 0..=d + 3 {
        let want = (i <= d).then(|| pow10(i));
        assert_eq!(T::get_exp(i as usize).map(|v| v.big()), want, "{} get_exp({i})", T::NAME);
    }
    assert_eq!(T::get_exp(usize::MAX), None);
    assert_eq!(T::get_exp(usize::MAX / 2), None);
    if T::SIGNED {
        assert_eq!(T::NEG_MIN_STR, (-T::min_big()).to_string(), "{} NEG_MIN_STR", T::NAME);
    }
    // `unsigned_abs` is exact at both ends, including |MIN|
    assert_eq!(T::MIN.unsigned_abs().to_u128().unwrap(), T::min_big().abs().to_u128().unwrap());
    assert_eq!(T::MAX.unsigned_abs().to_u128().unwrap(), T::max_big().to_u128().unwrap());
    // widening is lossless, and narrowing refuses what does not fit
    // (i128/u128 have no wider type; their `as_wider` is `unreachable!()` by design)
    if T::BITS < 128 {
        for v in sample::<T>(200) {
            assert_eq!(T::from_wider(v.as_wider()), Some(v), "{} wider roundtrip {v:?}", T::NAME);
        }
        assert_eq!(T::from_wider(T::MAX.as_wider() + <T::Wider as num_traits::ConstOne>::ONE), None);
        if T::SIGNED {
            assert_eq!(T::from_wider(T::MIN.as_wider() - <T::Wider as num_traits::ConstOne>::ONE), None);
        }
    }
    // the decimal types expose the same limits
    assert_eq!(C::<T, 3>::MAX.mantissa(), T::MAX);
    assert_eq!(C::<T, 3>::MIN.mantissa(), T::MIN);
    assert_eq!(C::<T, 3>::EPSILON.mantissa().big(), BigInt::from(1));
    assert_eq!(C::<T, 3>::ZERO.mantissa().big(), BigInt::from(0));
    assert_eq!(C::<T, 3>::MAX_POWERS.mantissa(), T::MAX_POWERS);
    assert_eq!(C::<T, 3>::DIGITS, d);
    assert_eq!(C::<T, 3>::SCALE, 3);
    assert_eq!(D::<T>::MAX.mantissa(), T::MAX);
    assert_eq!(D::<T>::DIGITS, d);
}

// ---------------------------------------------------------------------------
// 2. `%`
// ---------------------------------------------------------------------------

fn rem_oracle<T: Int>() {
    let mut f = Failures::default();
    let all = interesting::<T>();
    let mut rng = Rng(0x5EED ^ T::BITS as u64);
    let mut pairs: Vec<(T, T)> = Vec::new();
    for &a in &sample::<T>(120) {
        for &b in &all {
            pairs.push((a, b));
        }
    }
    for _ in 0..3000 {
        pairs.push((rng.int(), rng.int()));
    }
    // `MIN % -1` is the one pair a hardware remainder cannot do; make sure it is in
    if T::SIGNED {
        pairs.push((T::MIN, T::ZERO - T::ONE));
    }
    for (a, b) in pairs {
        let want = if b.big().is_zero() { None } else { T::from_big(&(a.big() % b.big())) };
        f.check(|| format!("{} checked_rem", T::NAME), || format!("{a:?} % {b:?}"),
            guarded(|| C::<T, 2>::from_mantissa(a).checked_rem(C::from_mantissa(b)).map(|v| v.mantissa())), want);
        if let Some(w) = want {
            f.check(|| format!("{} % operator", T::NAME), || format!("{a:?} % {b:?}"),
                guarded(|| (C::<T, 2>::from_mantissa(a) % C::from_mantissa(b)).mantissa()), w);
        }
    }
    f.finish(&format!("rem {}", T::NAME));
}

#[test]
#[should_panic(expected = "divisor of zero")]
fn rem_operator_by_zero_panics() {
    let _ = C::<i32, 2>::from_mantissa(5) % C::from_mantissa(0);
}

// ---------------------------------------------------------------------------
// 3. Integer -> decimal (`TryFrom<int>`)
// ---------------------------------------------------------------------------

fn check_from_int<Src: Int, I: Int, const S: i32>(f: &mut Failures)
where
    C<I, S>: TryFrom<Src, Error = ParseError>,
{
    for v in interesting::<Src>() {
        f.check(
            || format!("{} -> {}<{S}>", Src::NAME, I::NAME),
            || format!("{v:?}"),
            guarded(|| C::<I, S>::try_from(v).map(|d| d.mantissa().big()).map_err(|e| format!("{e:?}"))),
            spec_from_int::<I>(&v.big(), S).map_err(|e| format!("{e:?}")),
        );
    }
}

macro_rules! from_int_all_sources {
    ($f:ident, $i:ty, $s:literal) => {
        check_from_int::<i8, $i, $s>(&mut $f);
        check_from_int::<i16, $i, $s>(&mut $f);
        check_from_int::<i32, $i, $s>(&mut $f);
        check_from_int::<i64, $i, $s>(&mut $f);
        check_from_int::<i128, $i, $s>(&mut $f);
        check_from_int::<u8, $i, $s>(&mut $f);
        check_from_int::<u16, $i, $s>(&mut $f);
        check_from_int::<u32, $i, $s>(&mut $f);
        check_from_int::<u64, $i, $s>(&mut $f);
        check_from_int::<u128, $i, $s>(&mut $f);
    };
}

macro_rules! from_int_tests {
    ($($name:ident: $i:ty),*) => { $(
        #[test]
        fn $name() {
            let mut f = Failures::default();
            from_int_all_sources!(f, $i, -40);
            from_int_all_sources!(f, $i, -3);
            from_int_all_sources!(f, $i, 0);
            from_int_all_sources!(f, $i, 4);
            from_int_all_sources!(f, $i, 40);
            f.finish(concat!("TryFrom<int> for ", stringify!($i)));
        }
    )* };
}
from_int_tests!(from_int_i8: i8, from_int_i16: i16, from_int_i32: i32, from_int_i64: i64, from_int_i128: i128,
                from_int_u8: u8, from_int_u16: u16, from_int_u32: u32, from_int_u64: u64, from_int_u128: u128);

// ---------------------------------------------------------------------------
// 4. Float conversions
// ---------------------------------------------------------------------------

#[test]
fn float_specials_and_ties() {
    // not a number / infinite: an error, never a garbage value
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(C::<i32, 4>::try_from(bad), Err(ParseError::Overflow), "{bad}");
        assert_eq!(C::<i32, 4>::try_from(bad as f32), Err(ParseError::Overflow), "{bad}");
    }
    assert_eq!(C::<i32, 4>::try_from(-0.0f64), Ok(C::ZERO));
    assert_eq!(C::<i32, 4>::try_from(0.0f32), Ok(C::ZERO));
    // `round()` ties go away from zero
    assert_eq!(C::<i32, 0>::try_from(0.5f64).unwrap().mantissa(), 1);
    assert_eq!(C::<i32, 0>::try_from(-0.5f64).unwrap().mantissa(), -1);
    assert_eq!(C::<i32, 0>::try_from(2.5f64).unwrap().mantissa(), 3);
    assert_eq!(C::<i32, 0>::try_from(-2.5f64).unwrap().mantissa(), -3);
    // unsigned: -0.4 rounds to zero (fine), -0.6 to -1 (overflow)
    assert_eq!(C::<u32, 0>::try_from(-0.4f64).unwrap().mantissa(), 0);
    assert_eq!(C::<u32, 0>::try_from(-0.6f64), Err(ParseError::Overflow));
    // exactly at the edge of the range
    assert_eq!(C::<i32, 0>::try_from(i32::MAX as f64 + 0.4).unwrap().mantissa(), i32::MAX);
    assert_eq!(C::<i32, 0>::try_from(i32::MAX as f64 + 0.6), Err(ParseError::Overflow));
    assert_eq!(C::<i32, 0>::try_from(i32::MIN as f64 - 0.4).unwrap().mantissa(), i32::MIN);
    assert_eq!(C::<i32, 0>::try_from(i32::MIN as f64 - 0.6), Err(ParseError::Overflow));
    assert_eq!(C::<i32, 4>::try_from(1e30f64), Err(ParseError::Overflow));
    // the tuple form for OobScaleFpdec agrees with the const form
    assert_eq!(D::<i32>::try_from((1.23456789f64, 4)).unwrap().mantissa(), 12346);
    assert_eq!(D::<i32>::try_from((1.5f64, -1)).unwrap().mantissa(), 0); // 0.15 tens -> 0
    assert_eq!(D::<i32>::try_from((15.0f64, -1)).unwrap().mantissa(), 2); // 1.5 tens -> 2
    // i32::MIN scale must not overflow while negating it
    assert!(guarded(|| D::<i32>::try_from((1.0f64, i32::MIN))).is_ok());
    assert!(guarded(|| D::<i32>::from_mantissa(7).to_f64(i32::MIN)).is_ok());
    assert!(guarded(|| D::<i32>::from_mantissa(7).to_f32(i32::MIN)).is_ok());
}

/// Float -> decimal -> float is exact whenever the float has the digits to hold it.
macro_rules! float_roundtrip {
    ($name:ident, $i:ty, $float:ty, $bits:expr, [$($s:literal),*]) => {
        #[test]
        fn $name() {
            let mut rng = Rng(0xF10A7 ^ $bits);
            $(
                for _ in 0..2000 {
                    let m = (rng.next() >> (64 - $bits)) as i64 * if rng.next() & 1 == 1 { -1 } else { 1 };
                    #[allow(irrefutable_let_patterns)]
                    let Ok(m) = <$i>::try_from(m) else { continue };
                    let d = C::<$i, $s>::from_mantissa(m);
                    let fl: $float = d.into();
                    assert_eq!(C::<$i, $s>::try_from(fl), Ok(d), "{m} @ {} via {}", $s, stringify!($float));
                }
            )*
        }
    };
}
float_roundtrip!(f64_roundtrip_i64, i64, f64, 50, [-5, -1, 0, 1, 2, 4, 8, 12, 15, 22]);
float_roundtrip!(f64_roundtrip_i32, i32, f64, 31, [-3, 0, 2, 4, 9]);
float_roundtrip!(f32_roundtrip_i32, i32, f32, 20, [-3, 0, 1, 2, 4, 8, 10]);
float_roundtrip!(f32_roundtrip_i16, i16, f32, 15, [-2, 0, 2, 4]);

/// Float -> decimal at the edge of the integer range, where a float has fewer digits than
/// the integer (the spacing is 1024..2048 near 2^63). The conversion scales by `10^S`,
/// rounds, and must say Overflow exactly when the rounded value is outside the integer's
/// range -- never wrap, saturate, or accept the first double past the limit. (Brought over
/// from fixdec, whose `from_f64` had one defect on this seam.)
#[test]
fn float_conversion_at_the_integer_limits() {
    const P63: f64 = 9_223_372_036_854_775_808.0;
    fn ovf<T>() -> Result<T, ParseError> {
        Err(ParseError::Overflow)
    }
    // S = 0: the float IS the integer. The largest double below 2^63 is 2^63 - 1024.
    assert_eq!(C::<i64, 0>::try_from(P63), ovf());
    assert_eq!(C::<i64, 0>::try_from(-P63).unwrap().mantissa(), i64::MIN);
    assert_eq!(C::<i64, 0>::try_from(P63 - 1024.0).unwrap().mantissa(), i64::MAX - 1023);
    assert_eq!(C::<i64, 0>::try_from(-P63 - 2048.0), ovf());
    // unsigned: [2^63, 2^64) has spacing 2048
    assert_eq!(C::<u64, 0>::try_from(2.0 * P63), ovf());
    assert_eq!(C::<u64, 0>::try_from(2.0 * P63 - 2048.0).unwrap().mantissa(), u64::MAX - 2047);
    // 128-bit: spacing 2^74 below 2^127, 2^75 below 2^128
    let (p127, p128) = (2f64.powi(127), 2f64.powi(128));
    assert_eq!(C::<i128, 0>::try_from(p127), ovf());
    assert_eq!(C::<i128, 0>::try_from(-p127).unwrap().mantissa(), i128::MIN);
    assert_eq!(C::<i128, 0>::try_from(p127 - 2f64.powi(74)).unwrap().mantissa(), i128::MAX - ((1 << 74) - 1));
    assert_eq!(C::<u128, 0>::try_from(p128), ovf());
    assert_eq!(C::<u128, 0>::try_from(p128 - 2f64.powi(75)).unwrap().mantissa(), u128::MAX - ((1 << 75) - 1));
    // f32: 2^31 and 2^32
    let (p31, p32) = (2147483648.0f32, 4294967296.0f32);
    assert_eq!(C::<i32, 0>::try_from(p31), ovf());
    assert_eq!(C::<i32, 0>::try_from(-p31).unwrap().mantissa(), i32::MIN);
    assert_eq!(C::<i32, 0>::try_from(p31 - 128.0).unwrap().mantissa(), i32::MAX - 127);
    assert_eq!(C::<u32, 0>::try_from(p32), ovf());
    assert_eq!(C::<u32, 0>::try_from(p32 - 256.0).unwrap().mantissa(), u32::MAX - 255);

    // S > 0: walk the doubles on either side of the limit; Ok exactly when the scaled,
    // rounded value is in [-2^63, 2^63), with that value as the mantissa.
    for (scale_edge, scale) in [(P63 / 1e8, 8), (P63 / 1e2, 2)] {
        let edge = scale_edge.to_bits() as i64;
        for k in -40i64..=40 {
            for sign in [1.0, -1.0] {
                let x = sign * f64::from_bits((edge + k) as u64);
                let scaled = (x * 10f64.powi(scale)).round();
                let want = if scaled >= P63 || scaled < -P63 { ovf() } else { Ok(scaled as i64) };
                let got = if scale == 8 { C::<i64, 8>::try_from(x).map(|d| d.mantissa()) } else { C::<i64, 2>::try_from(x).map(|d| d.mantissa()) };
                assert_eq!(got, want, "{x:e} @ {scale}");
            }
        }
    }
}

/// Regression: the float was scaled by `10^S` formed as a float, which is `inf` beyond
/// `f32` 10^38 / `f64` 10^308 -- so `0.0` became `0 * inf = NaN` (Overflow) and decimal -> float
/// divided by infinity and returned `0`.
#[test]
fn float_conversion_when_the_power_of_ten_does_not_fit_the_float() {
    let rel = |got: f64, want: f64| (got / want - 1.0).abs() < 1e-5;
    // f32, scale 40 (10^40 > f32::MAX)
    assert_eq!(C::<i128, 40>::try_from(0.0f32), Ok(C::ZERO));
    assert_eq!(C::<i128, 40>::try_from(-0.0f32), Ok(C::ZERO));
    let tiny = 2f32.powi(-100); // 7.9e-31
    let d = C::<i128, 40>::try_from(tiny).unwrap();
    assert!(rel(d.mantissa() as f64, tiny as f64 * 1e40), "{}", d.mantissa());
    let back: f32 = C::<i128, 40>::from_mantissa(10_000_000_000).into();
    assert!(rel(back as f64, 1e-30), "{back:e}");
    // f64, scale 310 (10^310 > f64::MAX)
    assert_eq!(C::<i128, 310>::try_from(0.0f64), Ok(C::ZERO));
    let d = C::<i128, 310>::try_from(1e-300f64).unwrap();
    assert!(rel(d.mantissa() as f64, 1e10), "{}", d.mantissa());
    let back: f64 = C::<i128, 310>::from_mantissa(1_000_000_000).into(); // 1e-301
    assert!(rel(back, 1e-301), "{back:e}");
    // negative scales, beyond the chunk size
    assert_eq!(C::<i64, -37>::try_from(3e38f32).unwrap().mantissa(), 30);
    let f: f32 = C::<i64, -37>::from_mantissa(30).into();
    assert!(rel(f as f64, 3e38), "{f:e}");
    // the tuple form, and `to_f32`/`to_f64` with a runtime scale
    assert_eq!(D::<i128>::try_from((0.0f32, 40)).unwrap().mantissa(), 0);
    assert!(rel(D::<i128>::from_mantissa(10_000_000_000).to_f32(40) as f64, 1e-30));
    assert!(rel(D::<i128>::from_mantissa(1_000_000_000).to_f64(310), 1e-301));
    // the extreme scales must neither hang nor panic
    assert!(guarded(|| D::<i32>::try_from((1.0f64, i32::MAX))).is_ok());
    assert!(guarded(|| D::<i32>::try_from((1.0f64, i32::MIN))).is_ok());
    assert!(guarded(|| D::<i32>::from_mantissa(7).to_f64(i32::MAX)).is_ok());
}

/// Regression: `S + S2 - SR` was computed in `i32`, which overflows for scales near the limits
/// (debug: panic; release: a wrapped, plausible-looking difference).
#[test]
fn const_mul_div_scale_difference_does_not_overflow_i32() {
    type A = C<i64, { i32::MAX }>;
    let one = A::from_mantissa(1);
    // MAX + MAX - MIN, and MAX - MIN - MAX: far outside every type's range -> None, not a panic
    assert_eq!(guarded(|| one.checked_mul::<i64, { i32::MAX }, { i32::MIN }>(one)), Ok(None));
    assert_eq!(guarded(|| one.checked_div::<i64, { i32::MIN }, { i32::MAX }>(C::<i64, { i32::MIN }>::from_mantissa(1))), Ok(None));
    assert_eq!(guarded(|| one.checked_mul_ext::<i64, { i32::MAX }, { i32::MIN }>(one, Rounding::Floor)), Ok(None));
    // and an ordinary difference still works
    let x = C::<i64, 4>::from_mantissa(12_345);
    let y = C::<i64, 2>::from_mantissa(250);
    assert_eq!(x.checked_mul::<i64, 2, 4>(y).unwrap().mantissa(), 30_863); // 1.2345 * 2.50 = 3.08625 -> 3.0863
}

// ---------------------------------------------------------------------------
// 5. Overflow semantics (profile-aware)
// ---------------------------------------------------------------------------

/// `checked_*` is `None` in every profile; the operators panic where overflow
/// checks are on and WRAP where they are off -- but never saturate or lie.
#[test]
fn add_sub_neg_abs_overflow() {
    type X = C<i16, 2>;
    let (max, min, one) = (X::MAX, X::MIN, X::EPSILON);
    assert_eq!(max.checked_add(one), None);
    assert_eq!(min.checked_sub(one), None);
    assert_eq!(max.checked_add(X::ZERO), Some(max));
    assert_eq!(min.checked_abs(), None);
    assert_eq!(max.checked_abs(), Some(max));
    assert_eq!((min + one).checked_abs(), Some(max));
    assert_eq!(min.checked_mul_int(-1i16), None);
    assert_eq!(min.checked_div_int(-1i16), None);
    assert_eq!(min.checked_div_int(0i16), None);
    assert_eq!(max.checked_mul_int(2i16), None);
    assert_eq!(max.checked_mul_int(1i16), Some(max));

    let on = overflow_checks();
    for (name, got, wrapped) in [
        ("MAX + 1", guarded(|| black_box(max) + one), min),
        ("MIN - 1", guarded(|| black_box(min) - one), max),
        ("-MIN", guarded(|| -black_box(min)), min),
        ("abs(MIN)", guarded(|| black_box(min).abs()), min),
    ] {
        if on {
            assert!(got.is_err(), "{name}: expected a panic with overflow checks on");
        } else {
            assert_eq!(got, Ok(wrapped), "{name}: expected wrap-around with overflow checks off");
        }
    }
    let mut acc = max;
    let r = guarded(|| {
        acc += black_box(one);
        acc
    });
    assert_eq!(r.is_err(), on);
    let sum = guarded(|| [max, one].into_iter().sum::<X>());
    assert_eq!(sum.is_err(), on);
}

/// `*` and `/` are checked in every profile: they panic with a message.
#[test]
#[should_panic(expected = "overflow in decimal multiplication")]
fn mul_operator_overflow_panics_in_every_profile() {
    let _ = black_box(C::<i32, 2>::MAX) * black_box(C::<i32, 2>::from_mantissa(200));
}
#[test]
#[should_panic(expected = "overflow in decimal multiplication")]
fn mul_int_operator_overflow_panics_in_every_profile() {
    let _ = black_box(C::<i32, 2>::MAX) * black_box(2i32);
}
#[test]
#[should_panic(expected = "fail in decimal division")]
fn div_operator_by_zero_panics_in_every_profile() {
    let _ = black_box(C::<i32, 2>::from_mantissa(5)) / black_box(C::<i32, 2>::ZERO);
}
#[test]
#[should_panic(expected = "fail in decimal division")]
fn div_int_operator_by_zero_panics_in_every_profile() {
    let _ = black_box(C::<i32, 2>::from_mantissa(5)) / black_box(0i32);
}
#[test]
#[should_panic(expected = "fail in decimal division")]
fn oob_div_by_zero_panics_in_every_profile() {
    let _ = black_box(D::<i32>::from_mantissa(5)) / black_box(0i32);
}
#[test]
#[should_panic(expected = "fail in decimal division")]
fn oob_div_by_const_scale_zero_has_a_division_message() {
    // used to say "overflow in decimal multiplication"
    let _ = black_box(D::<i32>::from_mantissa(5)) / black_box(C::<i32, 2>::ZERO);
}
#[test]
#[should_panic(expected = "overflow in decimal multiplication")]
fn oob_mul_by_const_scale_overflow_panics() {
    let _ = black_box(D::<i32>::from_mantissa(i32::MAX)) * black_box(C::<i32, 0>::from_mantissa(2));
}

// ---------------------------------------------------------------------------
// 6. ConstScaleFpdec against the same spec
// ---------------------------------------------------------------------------

fn const_vs_spec<I: Int, const S: i32, const S2: i32, const SR: i32>(f: &mut Failures) {
    let vals = sample::<I>(40);
    for &a in &vals {
        for &b in &vals {
            for r in ROUNDINGS {
                let x = C::<I, S>::from_mantissa(a);
                let y = C::<I, S2>::from_mantissa(b);
                f.check(|| format!("{} const mul {S}+{S2}->{SR} {r:?}", I::NAME), || format!("{a:?} {b:?}"),
                    guarded(|| x.checked_mul_ext::<I, S2, SR>(y, r).map(|v| v.mantissa())), spec_mul(a, b, S + S2 - SR, r));
                f.check(|| format!("{} const div {S}-{S2}->{SR} {r:?}", I::NAME), || format!("{a:?} {b:?}"),
                    guarded(|| x.checked_div_ext::<I, S2, SR>(y, r).map(|v| v.mantissa())), spec_div(a, b, S - S2 - SR, r));
            }
        }
    }
}

/// A right-hand side of ANOTHER inner type: the product / quotient is exact, in the left's type
/// and scale; `None` only if THAT does not fit (a narrow left times a wide right is fine when the
/// result fits -- `0.0001 * 1000.00` -- and a negative result is `None` for an unsigned left).
fn mixed_vs_spec<I: Int, J: Int, const S: i32, const S2: i32, const SR: i32>(f: &mut Failures) {
    for &a in &sample::<I>(30) {
        for &b in &sample::<J>(30) {
            for r in [Rounding::Round, Rounding::Ceiling, Rounding::Floor] {
                let x = C::<I, S>::from_mantissa(a);
                let y = C::<J, S2>::from_mantissa(b);
                f.check(|| format!("{}x{} mixed mul", I::NAME, J::NAME), || format!("{a:?} {b:?}"),
                    guarded(|| x.checked_mul_ext::<J, S2, SR>(y, r).map(|v| v.mantissa())), spec_mul_mixed(a, b, S + S2 - SR, r));
                f.check(|| format!("{}x{} mixed div", I::NAME, J::NAME), || format!("{a:?} {b:?}"),
                    guarded(|| x.checked_div_ext::<J, S2, SR>(y, r).map(|v| v.mantissa())), spec_div_mixed(a, b, S - S2 - SR, r));
            }
        }
    }
}

macro_rules! const_spec_tests {
    ($($name:ident: $i:ty),*) => { $(
        #[test]
        fn $name() {
            let mut f = Failures::default();
            const_vs_spec::<$i, 2, 2, 2>(&mut f);
            const_vs_spec::<$i, 4, 4, 0>(&mut f);
            const_vs_spec::<$i, 0, 0, 5>(&mut f);
            const_vs_spec::<$i, -2, 3, 1>(&mut f);
            const_vs_spec::<$i, 5, -3, 0>(&mut f);
            const_vs_spec::<$i, 2, 2, 4>(&mut f);
            const_vs_spec::<$i, 0, 0, 0>(&mut f);
            const_vs_spec::<$i, 9, 9, 0>(&mut f);
            const_vs_spec::<$i, 20, 20, 2>(&mut f);
            f.finish(concat!("const_vs_spec ", stringify!($i)));
        }
    )* };
}
const_spec_tests!(const_i8: i8, const_i16: i16, const_i32: i32, const_i64: i64, const_i128: i128,
                  const_u8: u8, const_u16: u16, const_u32: u32, const_u64: u64, const_u128: u128);

#[test]
fn mixed_inner_types_match_spec() {
    let mut f = Failures::default();
    mixed_vs_spec::<i32, i16, 2, 3, 2>(&mut f);
    mixed_vs_spec::<i32, i64, 2, 3, 2>(&mut f);   // rhs wider than the left
    mixed_vs_spec::<i64, u32, 4, 6, 4>(&mut f);
    mixed_vs_spec::<u64, i32, 4, 2, 4>(&mut f);   // negative rhs -> None for an unsigned lhs
    mixed_vs_spec::<i128, u64, 6, 6, 6>(&mut f);
    mixed_vs_spec::<u32, u8, 2, 2, 2>(&mut f);
    mixed_vs_spec::<i16, i128, 2, 2, 2>(&mut f);
    mixed_vs_spec::<u128, i8, 3, 3, 3>(&mut f);
    f.finish("mixed inner types");
}

/// `ConstScaleFpdec::checked_round_ext(scale, r)` rounds to `scale` fraction digits.
fn const_round_vs_spec<I: Int, const S: i32>(f: &mut Failures) {
    let mut scales: Vec<i32> = vec![i32::MIN, -1_000, -50, -1, 0, 1, S - 1, S, S + 1, S + 2, 40, 1_000, i32::MAX];
    scales.sort();
    scales.dedup();
    for &a in &sample::<I>(80) {
        let x = C::<I, S>::from_mantissa(a);
        for r in ROUNDINGS {
            for &sc in &scales {
                let diff = (S as i64 - sc as i64).clamp(i32::MIN as i64, i32::MAX as i64);
                f.check(|| format!("{} const round S={S} {r:?}", I::NAME), || format!("{a:?} to {sc}"),
                    guarded(|| x.checked_round_ext(sc, r).map(|v| v.mantissa())), spec_round_any(a, diff, r));
                f.check(|| format!("{} const round-panics S={S} {r:?}", I::NAME), || format!("{a:?} to {sc}"),
                    Ok(guarded(|| x.round_ext(sc, r).mantissa()).ok()), spec_round_any(a, diff, r));
            }
        }
        for &sc in &scales {
            let diff = (S as i64 - sc as i64).clamp(i32::MIN as i64, i32::MAX as i64);
            f.check(|| format!("{} const checked_round S={S}", I::NAME), || format!("{a:?} to {sc}"),
                guarded(|| x.checked_round(sc).map(|v| v.mantissa())), spec_round_any(a, diff, Rounding::Round));
        }
    }
}

macro_rules! const_round_tests {
    ($($name:ident: $i:ty),*) => { $(
        #[test]
        fn $name() {
            let mut f = Failures::default();
            const_round_vs_spec::<$i, -3>(&mut f);
            const_round_vs_spec::<$i, 0>(&mut f);
            const_round_vs_spec::<$i, 2>(&mut f);
            const_round_vs_spec::<$i, 5>(&mut f);
            const_round_vs_spec::<$i, 12>(&mut f);
            const_round_vs_spec::<$i, 39>(&mut f);
            f.finish(concat!("const round ", stringify!($i)));
        }
    )* };
}
const_round_tests!(const_round_i8: i8, const_round_i16: i16, const_round_i32: i32, const_round_i64: i64,
                   const_round_i128: i128, const_round_u8: u8, const_round_u16: u16, const_round_u32: u32,
                   const_round_u64: u64, const_round_u128: u128);

/// The ratio argument can be an integer, a `ConstScaleFpdec` or an `OobScaleFpdec`;
/// all three must give the same answer as the plain integers.
#[test]
fn mul_ratio_accepts_decimals_and_narrower_ints() {
    let x = C::<i64, 2>::from_mantissa(3_000_000);
    let want = x.checked_mul_ratio(200i64, 300i64);
    assert_eq!(want, Some(C::from_mantissa(2_000_000)));
    assert_eq!(x.checked_mul_ratio(C::<i32, 4>::from_mantissa(200), C::<i32, 4>::from_mantissa(300)), want);
    assert_eq!(x.checked_mul_ratio(D::<i32>::from_mantissa(200), D::<i32>::from_mantissa(300)), want);
    assert_eq!(x.checked_mul_ratio(200i16, 300i16), want);
    assert_eq!(x.checked_mul_ratio(200u8, 255u8).map(|v| v.mantissa()), Some(2_352_941)); // 3_000_000*200/255
    assert_eq!(x.checked_mul_ratio(1i32, 0i32), None);
    assert_eq!(C::<i64, 2>::MAX.checked_mul_ratio(3i64, 2i64), None);
    assert_eq!(C::<i64, 2>::MAX.checked_mul_ratio(3i64, 3i64), Some(C::MAX)); // no spurious overflow
}

#[test]
fn const_and_oob_convert_losslessly() {
    let c = C::<i32, 6>::from_mantissa(123_450_000);
    let o: D<i32> = c.into();
    assert_eq!(o.mantissa(), 123_450_000);
    let back: C<i32, 6> = o.into();
    assert_eq!(back, c);
    assert_eq!(D::<i32>::from_mantissa(5).to_f64(1), 0.5);
    assert_eq!(D::<i32>::from_mantissa(5).to_f64(0), 5.0);
    assert_eq!(D::<i32>::from_mantissa(5).to_f64(-2), 500.0);
    assert_eq!(D::<i32>::from_mantissa(-5).to_f32(1), -0.5);
}

// ---------------------------------------------------------------------------
// 7. Documented panics
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn display_scale_201_panics() {
    let _ = format!("{}", OobFmt(D::<i8>::from_mantissa(1), 201));
}
#[test]
#[should_panic]
fn display_scale_minus_201_panics() {
    let _ = format!("{}", OobFmt(D::<i8>::from_mantissa(1), -201));
}
#[test]
#[should_panic]
fn display_precision_201_panics() {
    let _ = format!("{:.201}", OobFmt(D::<i8>::from_mantissa(1), 1));
}
#[test]
#[should_panic]
fn display_negative_scale_plus_precision_over_200_panics() {
    let _ = format!("{:.150}", OobFmt(D::<i8>::from_mantissa(1), -60));
}
#[test]
fn display_limits_are_exactly_200() {
    assert_eq!(format!("{}", OobFmt(D::<i8>::from_mantissa(1), 200)).len(), 202);
    assert_eq!(format!("{}", OobFmt(D::<i8>::from_mantissa(1), -200)).len(), 201);
    assert_eq!(format!("{:.200}", OobFmt(D::<u128>::from_mantissa(u128::MAX), 0)).len(), 39 + 1 + 200);
    assert_eq!(format!("{:.100}", OobFmt(D::<u128>::from_mantissa(u128::MAX), -100)).len(), 139 + 1 + 100);
}

// ---------------------------------------------------------------------------
// 8. Regressions: one per defect found by the review. Each value below is one
//    that broke the code before the fix.
// ---------------------------------------------------------------------------

/// u64 `calc_mul_div_exp`: the fast path was taken when `a*b < 2^64`, but it then
/// added the rounding extra `10^i - 1` in u64, which can overflow: debug panic /
/// release wrap to a tiny number.
#[test]
fn regress_u64_mul_diff19_fast_path_overflowed_adding_the_rounding_extra() {
    let a = D::<u64>::from_mantissa(3_000_000_000); // a*a = 9e18; + (10^19 - 1) > u64::MAX
    let m = |r| a.checked_mul_ext(a, 19, r).map(|v| v.mantissa());
    assert_eq!(m(Rounding::Ceiling), Some(1));
    assert_eq!(m(Rounding::AwayFromZero), Some(1));
    assert_eq!(m(Rounding::Round), Some(1)); // 0.9 -> 1
    assert_eq!(m(Rounding::Floor), Some(0));
    assert_eq!(m(Rounding::TowardsZero), Some(0));
}

/// i128/u128: `div-pow10`'s `unchecked_div_single_r1b` requires `n <= 2^127`
/// (a `debug_assert!`; wrong in release). `n = a*b + extra` exceeds it for
/// `a*b` in `[7e37, 8.5e37)` with `diff = 38`.
#[test]
fn regress_128bit_mul_diff38_dividend_above_2_pow_127() {
    let v = 8_900_000_000_000_000_000u64; // v*v = 7.921e37
    let (i, u) = (D::<i128>::from_mantissa(v as i128), D::<u128>::from_mantissa(v as u128));
    for r in [Rounding::Ceiling, Rounding::AwayFromZero, Rounding::Round] {
        assert_eq!(i.checked_mul_ext(i, 38, r).map(|x| x.mantissa()), Some(1), "i128 {r:?}");
        assert_eq!(u.checked_mul_ext(u, 38, r).map(|x| x.mantissa()), Some(1), "u128 {r:?}");
    }
    for r in [Rounding::Floor, Rounding::TowardsZero] {
        assert_eq!(i.checked_mul_ext(i, 38, r).map(|x| x.mantissa()), Some(0), "i128 {r:?}");
        assert_eq!(u.checked_mul_ext(u, 38, r).map(|x| x.mantissa()), Some(0), "u128 {r:?}");
    }
    // negative product: Floor goes to -1
    let n = D::<i128>::from_mantissa(-(v as i128));
    assert_eq!(n.checked_mul_ext(i, 38, Rounding::Floor).map(|x| x.mantissa()), Some(-1));
    assert_eq!(n.checked_mul_ext(i, 38, Rounding::Ceiling).map(|x| x.mantissa()), Some(0));
}

/// `MIN % -1` is 0 mathematically, but the hardware instruction traps.
#[test]
fn regress_checked_rem_min_by_minus_one() {
    assert_eq!(C::<i8, 2>::MIN.checked_rem(C::from_mantissa(-1)), Some(C::ZERO));
    assert_eq!(C::<i64, 2>::MIN.checked_rem(C::from_mantissa(-1)), Some(C::ZERO));
    assert_eq!(C::<i128, 2>::MIN % C::from_mantissa(-1), C::ZERO);
    assert_eq!(C::<i8, 2>::MIN.checked_rem(C::from_mantissa(0)), None);
    assert_eq!(C::<i8, 2>::MIN.checked_rem(C::from_mantissa(1)), Some(C::ZERO));
    assert_eq!(C::<i8, 2>::MIN.checked_rem(C::MAX), Some(C::from_mantissa(-1)));
}

/// The fraction part used to be parsed by `from_str_radix`, which accepts a sign.
#[test]
fn regress_parse_sign_inside_fraction_is_rejected() {
    for s in ["1.+5", "1.-5", "0.-5", "-0.-5", "-0.+5", "+0.-5", "0.+5", "1.5-", "1.5+", "1.-", "1.+"] {
        assert!(D::<i64>::try_from_str(s, 4).is_err(), "{s:?} must not parse");
        assert!(s.parse::<OobFmt<i64>>().is_err(), "{s:?} must not parse (OobFmt)");
    }
    // used to be 1.05, 0.95 and -0.05
    assert_eq!(D::<i64>::try_from_str("1.+5", 2), Err(ParseError::Invalid));
    assert_eq!(D::<i64>::try_from_str("1.-5", 2), Err(ParseError::Invalid));
}

/// "00" used to be `Empty` (its trailing zeros were trimmed to nothing), and
/// `-0.0`-style zeros failed for unsigned types.
#[test]
fn regress_parse_every_spelling_of_zero() {
    for s in ["0", "00", "000", "-0", "+0", "-00", "+00", "0.0", "00.00", "-0.0", "+0.000", "0.00000000000"] {
        assert_eq!(D::<i32>::try_from_str(s, 3).map(|v| v.mantissa()), Ok(0), "i32 {s:?}");
        assert_eq!(D::<u32>::try_from_str(s, 3).map(|v| v.mantissa()), Ok(0), "u32 {s:?}");
        assert_eq!(D::<u8>::try_from_str(s, -9).map(|v| v.mantissa()), Ok(0), "u8 -9 {s:?}");
        assert_eq!(D::<i8>::try_from_str(s, 99).map(|v| v.mantissa()), Ok(0), "i8 99 {s:?}");
        assert_eq!(s.parse::<OobFmt<u128>>().map(|v| v.0.mantissa()), Ok(0), "OobFmt {s:?}");
    }
    assert_eq!("0.00".parse::<OobFmt<i8>>().map(|o| (o.0.mantissa(), o.1)), Ok((0, 2)));
    assert_eq!("00".parse::<OobFmt<i8>>().map(|o| (o.0.mantissa(), o.1)), Ok((0, 0)));
    // but a negative non-zero is still invalid for unsigned types
    assert_eq!(D::<u8>::try_from_str("-0.5", 2), Err(ParseError::Invalid));
    assert_eq!(D::<u8>::try_from_str("-1", 2), Err(ParseError::Invalid));
}

/// `{:.0}` of a value below 1 whose `10^scale` does not fit the type used to print
/// "0" even when the value is >= 0.5 (u16: 65535 at scale 5 is 0.65535).
#[test]
fn regress_display_rounds_up_when_the_unit_does_not_fit_the_type() {
    let f = |m: u16, sc: i32, p: usize| format!("{:.p$}", OobFmt(D::<u16>::from_mantissa(m), sc));
    assert_eq!(f(65535, 5, 0), "1");
    assert_eq!(f(50000, 5, 0), "1"); // exactly 0.5 rounds away from zero
    assert_eq!(f(49999, 5, 0), "0");
    assert_eq!(f(65535, 6, 1), "0.1"); // 0.065535 -> 0.1
    assert_eq!(f(50000, 6, 1), "0.1"); // 0.05 -> 0.1
    assert_eq!(f(49999, 6, 1), "0.0");
    assert_eq!(f(65535, 5, 1), "0.7");
    assert_eq!(f(65535, 5, 5), "0.65535");
    assert_eq!(format!("{}", OobFmt(D::<u16>::from_mantissa(65535), 5)), "0.65535"); // no precision: unaffected
}

/// `diff_scale = i32::MIN` used to overflow `-diff_scale`; `rescale` / `try_from_str`
/// / the float conversions had the same overflow with scales near the i32 limits.
#[test]
fn regress_extreme_scales_do_not_overflow_i32_arithmetic() {
    let x = D::<i32>::from_mantissa(12345);
    for r in ROUNDINGS {
        assert_eq!(x.checked_round_diff_ext(i32::MIN, r), Some(x), "{r:?}");
        assert_eq!(x.checked_mul_ext(x, i32::MIN, r), None, "{r:?}");
        assert_eq!(x.checked_div_ext(x, i32::MIN, r), None, "{r:?}");
        assert_eq!(x.checked_mul_ext(x, i32::MAX, r), None, "{r:?}");
        assert_eq!(x.checked_div_ext(x, i32::MAX, r), None, "{r:?}");
    }
    let o = OobFmt(x, 5);
    assert_eq!(o.rescale(i32::MAX), Err(ParseError::Overflow));
    assert_eq!(o.rescale(i32::MIN), Err(ParseError::Precision));
    assert_eq!(OobFmt(x, i32::MIN).rescale(i32::MAX), Err(ParseError::Overflow));
    assert_eq!(OobFmt(x, i32::MAX).rescale(i32::MIN), Err(ParseError::Precision));
    assert_eq!(D::<i8>::try_from_str("100", i32::MAX), Err(ParseError::Overflow));
    assert_eq!(D::<i8>::try_from_str("1", i32::MAX), Err(ParseError::Overflow));
    assert_eq!(D::<i8>::try_from_str("1", i32::MIN), Err(ParseError::Precision));
    assert_eq!(D::<i8>::try_from_str("0", i32::MIN), Ok(D::from_mantissa(0)));
    assert_eq!(D::<i8>::try_from_str("0", i32::MAX), Ok(D::from_mantissa(0)));
    assert_eq!(D::<i8>::try_from_str("0.1", i32::MIN), Err(ParseError::Precision));
}

/// `round`/`round_diff` used to silently return a WRONG VALUE when the unit did not
/// fit (`Ceiling` of 1 at diff 3 for `i8` returned 0 instead of an error) and to
/// overflow-wrap in release. Now `checked_*` is `None` and the plain forms panic.
#[test]
fn regress_round_overflow_is_reported_not_zeroed() {
    let one = D::<i8>::from_mantissa(1);
    assert_eq!(one.checked_round_diff_ext(3, Rounding::Ceiling), None); // 1000 does not fit i8
    assert_eq!(one.checked_round_diff_ext(3, Rounding::AwayFromZero), None);
    assert_eq!(one.checked_round_diff_ext(3, Rounding::Floor).map(|v| v.mantissa()), Some(0));
    assert_eq!(one.checked_round_diff_ext(3, Rounding::Round).map(|v| v.mantissa()), Some(0));
    let neg = D::<i8>::from_mantissa(-1);
    assert_eq!(neg.checked_round_diff_ext(3, Rounding::Floor), None);
    assert_eq!(neg.checked_round_diff_ext(3, Rounding::Ceiling).map(|v| v.mantissa()), Some(0));
    // ties go away from zero, and the rounded multiple of ten must itself fit
    assert_eq!(D::<i8>::from_mantissa(124).checked_round_diff_ext(1, Rounding::Round).map(|v| v.mantissa()), Some(120));
    assert_eq!(D::<i8>::from_mantissa(125).checked_round_diff_ext(1, Rounding::Round), None); // 130
    assert_eq!(D::<i8>::from_mantissa(127).checked_round_diff_ext(1, Rounding::Round), None); // 130
    assert_eq!(D::<i8>::from_mantissa(-128).checked_round_diff_ext(1, Rounding::Round), None); // -130
    assert_eq!(D::<i16>::from_mantissa(1234).checked_round_diff(2).map(|v| v.mantissa()), Some(1200));
    assert_eq!(D::<i16>::from_mantissa(1250).checked_round_diff(2).map(|v| v.mantissa()), Some(1300));
    assert_eq!(D::<i16>::from_mantissa(-1250).checked_round_diff(2).map(|v| v.mantissa()), Some(-1300));
    assert_eq!(D::<i16>::from_mantissa(1234).checked_round_diff(0).map(|v| v.mantissa()), Some(1234));
    assert_eq!(D::<i16>::from_mantissa(1234).checked_round_diff(-3).map(|v| v.mantissa()), Some(1234));
    // the u16 special case: 65535 is 0.65535 at diff 5, which rounds to a unit that does not fit
    assert_eq!(D::<u16>::from_mantissa(65535).checked_round_diff(5), None);
    assert_eq!(D::<u16>::from_mantissa(49999).checked_round_diff(5).map(|v| v.mantissa()), Some(0));
    assert_eq!(D::<u16>::from_mantissa(0).checked_round_diff(5).map(|v| v.mantissa()), Some(0));
    // const flavour
    assert_eq!(C::<i16, 4>::from_mantissa(12_345).checked_round(2).map(|v| v.mantissa()), Some(12_300));
    assert_eq!(C::<i16, 4>::from_mantissa(12_350).checked_round(2).map(|v| v.mantissa()), Some(12_400));
    assert_eq!(C::<i16, 4>::from_mantissa(12_345).checked_round(4).map(|v| v.mantissa()), Some(12_345));
    assert_eq!(C::<i16, 4>::from_mantissa(12_345).checked_round(9).map(|v| v.mantissa()), Some(12_345));
    assert_eq!(C::<i16, 4>::from_mantissa(32_767).checked_round(0).map(|v| v.mantissa()), Some(30_000)); // 3.2767 -> 3
    assert_eq!(C::<i16, 4>::from_mantissa(32_767).checked_round_ext(0, Rounding::Ceiling), None); // 4 -> 40000
    assert_eq!(C::<i32, 4>::from_mantissa(50_000).checked_round(-1).map(|v| v.mantissa()), Some(100_000)); // 5.0 -> 10
    assert_eq!(C::<i32, 4>::from_mantissa(49_999).checked_round(-1).map(|v| v.mantissa()), Some(0));
}

macro_rules! per_type {
    ($($t:ty: $inv:ident, $rem:ident);* $(;)?) => { $(
        #[test] fn $inv() { invariants::<$t>(); }
        #[test] fn $rem() { rem_oracle::<$t>(); }
    )* };
}
per_type!(i8: consts_i8, rem_i8; i16: consts_i16, rem_i16; i32: consts_i32, rem_i32;
          i64: consts_i64, rem_i64; i128: consts_i128, rem_i128;
          u8: consts_u8, rem_u8; u16: consts_u16, rem_u16; u32: consts_u32, rem_u32;
          u64: consts_u64, rem_u64; u128: consts_u128, rem_u128);

/// Zero is representable at every scale. Integer conversion rejected it whenever
/// `10^scale` did not fit the type (`Overflow` for a large positive scale,
/// `Precision` for a large negative one), although the string and float
/// conversions accepted it; `OobFmt::rescale` had the same flaw.
#[test]
fn regress_zero_converts_at_every_scale() {
    assert_eq!(C::<i8, 3>::try_from(0i32), Ok(C::ZERO)); // 10^3 does not fit i8
    assert_eq!(C::<i8, 40>::try_from(0u64), Ok(C::ZERO));
    assert_eq!(C::<u16, 9>::try_from(0i8), Ok(C::ZERO));
    assert_eq!(C::<i8, -3>::try_from(0i32), Ok(C::ZERO)); // used to be Precision
    assert_eq!(C::<i16, -40>::try_from(0u128), Ok(C::ZERO));
    assert_eq!(C::<i8, 3>::try_from(0.0f64), Ok(C::ZERO));
    assert_eq!(C::<i8, 3>::try_from("0".parse::<i8>().unwrap()), Ok(C::ZERO));
    // ...but a non-zero value still has to fit
    assert_eq!(C::<i8, 3>::try_from(1i32), Err(ParseError::Overflow));
    assert_eq!(C::<i8, -3>::try_from(1i32), Err(ParseError::Precision));
    assert_eq!(C::<i8, -1>::try_from(10i32).map(|v| v.mantissa()), Ok(1));

    let zero = "0.0".parse::<OobFmt<i8>>().unwrap();
    assert_eq!(zero.rescale(100), Ok(D::from_mantissa(0))); // used to be Overflow
    assert_eq!(zero.rescale(-100), Ok(D::from_mantissa(0))); // used to be Precision
    let one = "1.0".parse::<OobFmt<i8>>().unwrap();
    assert_eq!(one.rescale(100), Err(ParseError::Overflow));
    assert_eq!(one.rescale(-100), Err(ParseError::Precision));
    assert_eq!(one.rescale(0), Ok(D::from_mantissa(1)));
}

/// Error kinds of `try_from_str`: more fraction digits than the scale is `Precision`;
/// a number that does not fit is `Overflow`. A value too big to even hold its
/// own digits used to be reported as `Precision`.
#[test]
fn regress_parse_error_kinds() {
    use ParseError::*;
    let p = |s: &str, sc: i32| D::<i16>::try_from_str(s, sc);
    assert_eq!(p("9999", 4), Err(Overflow)); // documented
    assert_eq!(p("1.23456", 4), Err(Precision)); // documented
    assert_eq!(p("1.23", 4).map(|v| v.mantissa()), Ok(12300));
    assert_eq!(p("1.0000", 4).map(|v| v.mantissa()), Ok(10000));
    assert_eq!(p("1.00001", 5), Err(Overflow)); // 100001 does not fit i16, and the scale allows 5 digits
    assert_eq!(D::<i8>::try_from_str("1.001", 3), Err(Overflow)); // used to be Precision
    assert_eq!(D::<i8>::try_from_str("1000", 0), Err(Overflow));
    assert_eq!(D::<i8>::try_from_str("1000000000000000000000000000000", 0), Err(Overflow)); // used to be Precision
    assert_eq!(D::<i8>::try_from_str("0.000000000001", 3), Err(Precision)); // 12 digits into scale 3
    assert_eq!(D::<i8>::try_from_str("0.001", 3).map(|v| v.mantissa()), Ok(1));
    assert_eq!(D::<i8>::try_from_str("0.0000000000000001", 16).map(|v| v.mantissa()), Ok(1));
    assert_eq!(D::<i8>::try_from_str("0.0000000000000001", 30), Err(Overflow)); // 1e14 does not fit
    assert_eq!(D::<i8>::try_from_str("123456789", -5), Err(Overflow));
    assert_eq!(D::<i8>::try_from_str("123456789", 5), Err(Overflow));
}

/// Pins the existing policy (see the module doc of `tests/parse_format.rs`): trailing
/// zeros count as precision for `try_from_str`, but not for `OobFmt::rescale`.
/// If this is ever changed on purpose, these are the lines to update.
#[test]
fn trailing_zero_policy_is_textual() {
    assert_eq!(D::<i32>::try_from_str("1.50", 1), Err(ParseError::Precision));
    assert_eq!(D::<i32>::try_from_str("1.50", 2).map(|v| v.mantissa()), Ok(150));
    assert_eq!(D::<i32>::try_from_str("150", -1), Ok(D::from_mantissa(15)));
    assert_eq!(D::<i32>::try_from_str("150", -2), Err(ParseError::Precision)); // 1.5 hundreds
    assert_eq!(D::<i32>::try_from_str("0.00", 1), Ok(D::from_mantissa(0))); // zero is exempt
    let o = "1.50".parse::<OobFmt<i32>>().unwrap();
    assert_eq!((o.0.mantissa(), o.1), (150, 2));
    assert_eq!(o.rescale(1), Ok(D::from_mantissa(15))); // rescale is about the value
    assert_eq!(o.rescale(0), Err(ParseError::Precision));
}
