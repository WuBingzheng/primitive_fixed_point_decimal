//! Differential tests: every arithmetic entry point against an arbitrary-
//! precision oracle (`tests/common`), for ALL ten inner integer types, ALL five
//! rounding modes, signs, and the magnitudes where the fast paths hand over to
//! the slow ones.
//!
//! Why this exists: the in-crate tests compare `calc_mul_div` with
//! `calc_mul_div_exp` -- two implementations by the same author that share
//! operand magnitudes (`10^i - k` against `u128::MAX - j*113`), so a flaw in the
//! boundary of the "happy path" guard (`leading_zeros(a) + leading_zeros(b) >= N`)
//! is invisible to them. The oracle shares nothing with the implementation.
//!
//! Scale the sampling up with `PFPD_ORACLE_SCALE=10 cargo test --release --test oracle`.
mod common;
use common::*;

use num_bigint::BigInt;
use num_traits::Signed;
use primitive_fixed_point_decimal::{OobScaleFpdec as D, Rounding};

fn scale() -> usize {
    std::env::var("PFPD_ORACLE_SCALE").ok().and_then(|s| s.parse().ok()).unwrap_or(1)
}

// ---------------------------------------------------------------------------
// The checks
// ---------------------------------------------------------------------------

/// The `diff` values for rounding: `diffs()` plus the first few beyond the type's
/// digits (where `10^diff` stops fitting and the library has a separate branch) and
/// one far beyond.
fn round_diffs<T: Int>() -> Vec<i32> {
    let d = T::DIGITS as i32;
    let mut v = diffs::<T>();
    v.extend([d + 2, d + 3, d + 40]);
    v.sort();
    v.dedup();
    v
}

fn check_round<T: Int>(f: &mut Failures, a: T, ds: &[i32]) {
    let x = D::from_mantissa(a);
    let name = T::NAME;
    for r in ROUNDINGS {
        for &d in ds {
            let want = spec_round(a, d, r);
            f.check(|| format!("{name} checked_round_diff_ext diff={d} {r:?}"), || format!("{a:?}"),
                guarded(|| x.checked_round_diff_ext(d, r).map(|v| v.mantissa())), want);
            // the panicking twin must agree: same value, or a panic exactly when `None`
            f.check(|| format!("{name} round_diff_ext diff={d} {r:?}"), || format!("{a:?}"),
                Ok(guarded(|| x.round_diff_ext(d, r).mantissa()).ok()), want);
        }
        let want = spec_round_huge(a, r);
        f.check(|| format!("{name} checked_round_diff_ext i32::MAX {r:?}"), || format!("{a:?}"),
            guarded(|| x.checked_round_diff_ext(i32::MAX, r).map(|v| v.mantissa())), want);
        // diff = i32::MIN: nothing to drop (this used to overflow `-diff_scale`)
        f.check(|| format!("{name} checked_round_diff_ext i32::MIN {r:?}"), || format!("{a:?}"),
            guarded(|| x.checked_round_diff_ext(i32::MIN, r).map(|v| v.mantissa())), Some(a));
    }
    // `round_diff` is `round_diff_ext(.., Rounding::Round)`
    for &d in ds {
        f.check(|| format!("{name} checked_round_diff diff={d}"), || format!("{a:?}"),
            guarded(|| x.checked_round_diff(d).map(|v| v.mantissa())), spec_round(a, d, Rounding::Round));
    }
}

fn check_pair<T: Int>(f: &mut Failures, a: T, b: T, ds: &[i32]) {
    let (x, y) = (D::from_mantissa(a), D::from_mantissa(b));
    let name = T::NAME;

    // add / sub: exact, `None` exactly when out of range.
    let sum = T::from_big(&(a.big() + b.big()));
    let dif = T::from_big(&(a.big() - b.big()));
    f.check(|| format!("{name} checked_add"), || format!("{a:?} + {b:?}"),
        guarded(|| x.checked_add(y).map(|v| v.mantissa())), sum);
    f.check(|| format!("{name} checked_sub"), || format!("{a:?} - {b:?}"),
        guarded(|| x.checked_sub(y).map(|v| v.mantissa())), dif);
    f.check(|| format!("{name} checked_mul_int"), || format!("{a:?} * {b:?}"),
        guarded(|| x.checked_mul_int(b).map(|v| v.mantissa())), T::from_big(&(a.big() * b.big())));

    for r in ROUNDINGS {
        f.check(|| format!("{name} rounding_div {r:?}"), || format!("{a:?} / {b:?}"),
            guarded(|| a.rounding_div(b, r)), spec_rounding_div(a, b, r));
        f.check(|| format!("{name} checked_div_int_ext {r:?}"), || format!("{a:?} / {b:?}"),
            guarded(|| x.checked_div_int_ext(b, r).map(|v| v.mantissa())), spec_rounding_div(a, b, r));
    }

    for &d in ds {
        // With diff <= 0 the multiply has no rounding step, so one mode is enough.
        let mul_modes: &[Rounding] = if d > 0 { &ROUNDINGS } else { &ROUNDINGS[..1] };
        for &r in mul_modes {
            f.check(|| format!("{name} mul diff={d} {r:?}"), || format!("{a:?} * {b:?}"),
                guarded(|| x.checked_mul_ext(y, d, r).map(|v| v.mantissa())), spec_mul(a, b, d, r));
        }
        for r in ROUNDINGS {
            f.check(|| format!("{name} div diff={d} {r:?}"), || format!("{a:?} / {b:?}"),
                guarded(|| x.checked_div_ext(y, d, r).map(|v| v.mantissa())), spec_div(a, b, d, r));
        }
    }
}

fn check_ratio<T: Int>(f: &mut Failures, a: T, n: T, d: T) {
    let x = D::from_mantissa(a);
    let name = T::NAME;
    for r in ROUNDINGS {
        f.check(|| format!("{name} mul_ratio {r:?}"), || format!("{a:?} * {n:?} / {d:?}"),
            guarded(|| x.checked_mul_ratio_ext(n, d, r).map(|v| v.mantissa())), spec_ratio(a, n, d, r));
    }
}

fn run_type<T: Int>() {
    let mut f = Failures::default();
    let k = scale();
    let ds = diffs::<T>();
    let all = interesting::<T>();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ T::BITS as u64 ^ ((T::SIGNED as u64) << 40));

    // 8-bit types: EXHAUSTIVE. Every pair, every diff, every rounding is a proof.
    if T::BITS == 8 {
        let lo = T::min_big();
        let hi = T::max_big();
        let mut vals = Vec::new();
        let mut v = lo.clone();
        while v <= hi {
            vals.push(T::from_big(&v).unwrap());
            v += 1;
        }
        let rds = round_diffs::<T>();
        for &a in &vals {
            check_round(&mut f, a, &rds);
            for &b in &vals {
                check_pair(&mut f, a, b, &ds);
            }
        }
        for &a in vals.iter().step_by(7) {
            for &n in vals.iter().step_by(5) {
                for &d in vals.iter().step_by(3) {
                    check_ratio(&mut f, a, n, d);
                }
            }
        }
        f.finish(&format!("oracle {} (exhaustive)", T::NAME));
        return;
    }

    // 1. A deterministic grid over the boundary values.
    let stride = (all.len() / (60 * k)).max(1);
    let core: Vec<T> = all.iter().step_by(stride).copied().collect();
    for &a in &core {
        for &b in &core {
            check_pair(&mut f, a, b, &ds);
        }
    }
    // 2. Every boundary value against a rotating sample of the others.
    for (i, &a) in all.iter().enumerate() {
        for j in 0..(8 * k) {
            let b = all[(i * 7 + j * 13 + 1) % all.len()];
            check_pair(&mut f, a, b, &ds);
        }
    }
    // 3. Products that land on word boundaries; 4. exact half-way ties.
    for (a, b) in product_edge_pairs::<T>(&mut rng, 3 * k) {
        check_pair(&mut f, a, b, &ds);
    }
    for (a, b) in tie_pairs::<T>(&mut rng, 2 * k) {
        check_pair(&mut f, a, b, &ds);
    }
    // 5. Uniform-bit-length random pairs.
    for _ in 0..(3000 * k) {
        let (a, b): (T, T) = (rng.int(), rng.int());
        check_pair(&mut f, a, b, &ds);
    }
    // 6. rounding: every boundary value, plus uniform random magnitudes.
    let rds = round_diffs::<T>();
    for &a in &all {
        check_round(&mut f, a, &rds);
    }
    for _ in 0..(1500 * k) {
        check_round(&mut f, rng.int::<T>(), &rds);
    }
    // 7. a*n/d ratios.
    for _ in 0..(2500 * k) {
        let (a, n, d): (T, T, T) = (rng.int(), rng.int(), rng.int());
        check_ratio(&mut f, a, n, d);
    }
    for _ in 0..(1500 * k) {
        let pick = |rng: &mut Rng| all[rng.below(all.len() as u64) as usize];
        let (a, n, d) = (pick(&mut rng), pick(&mut rng), pick(&mut rng));
        check_ratio(&mut f, a, n, d);
    }
    f.finish(&format!("oracle {}", T::NAME));
}

macro_rules! per_type {
    ($($name:ident: $t:ty),*) => { $( #[test] fn $name() { run_type::<$t>(); } )* };
}
per_type!(oracle_i8: i8, oracle_i16: i16, oracle_i32: i32, oracle_i64: i64, oracle_i128: i128,
          oracle_u8: u8, oracle_u16: u16, oracle_u32: u32, oracle_u64: u64, oracle_u128: u128);

// ---------------------------------------------------------------------------
// The oracle itself must be right, or everything above proves nothing.
// ---------------------------------------------------------------------------

#[test]
fn oracle_round_div_matches_hand_computed_table() {
    use Rounding::*;
    // (n, d, [Round, Floor, Ceiling, TowardsZero, AwayFromZero])
    let table: &[(i64, i64, [i64; 5])] = &[
        (7, 2, [4, 3, 4, 3, 4]),        // 3.5   tie rounds AWAY from zero
        (-7, 2, [-4, -4, -3, -3, -4]),  // -3.5
        (7, -2, [-4, -4, -3, -3, -4]),
        (-7, -2, [4, 3, 4, 3, 4]),
        (5, 3, [2, 1, 2, 1, 2]),        // 1.67
        (-5, 3, [-2, -2, -1, -1, -2]),
        (4, 3, [1, 1, 2, 1, 2]),        // 1.33
        (-4, 3, [-1, -2, -1, -1, -2]),
        (6, 3, [2, 2, 2, 2, 2]),        // exact: every mode agrees
        (-6, 3, [-2, -2, -2, -2, -2]),
        (0, 5, [0, 0, 0, 0, 0]),
        (1, 3, [0, 0, 1, 0, 1]),
        (-1, 3, [0, -1, 0, 0, -1]),
        (1, 2, [1, 0, 1, 0, 1]),        // 0.5
        (-1, 2, [-1, -1, 0, 0, -1]),
    ];
    for (n, d, want) in table {
        for (i, r) in [Round, Floor, Ceiling, TowardsZero, AwayFromZero].into_iter().enumerate() {
            let got = round_div(&BigInt::from(*n), &BigInt::from(*d), r);
            assert_eq!(got, BigInt::from(want[i]), "{n}/{d} {r:?}");
        }
    }
}

#[test]
fn oracle_agrees_with_native_i64_arithmetic() {
    let mut rng = Rng(7);
    for _ in 0..20_000 {
        let a = (rng.next() as i64) >> (rng.below(60) as u32);
        let b = loop {
            let b = (rng.next() as i64) >> (rng.below(60) as u32);
            if b != 0 {
                break b;
            }
        };
        let (q, rem) = (a.wrapping_div(b), a.wrapping_rem(b));
        let want_trunc = q;
        let got = round_div(&BigInt::from(a), &BigInt::from(b), Rounding::TowardsZero);
        assert_eq!(got, BigInt::from(want_trunc));
        // floor via Rust's own `div_euclid`/`rem_euclid` relation
        let floor = if rem != 0 && ((rem < 0) != (b < 0)) { q - 1 } else { q };
        assert_eq!(round_div(&BigInt::from(a), &BigInt::from(b), Rounding::Floor), BigInt::from(floor));
        let _ = BigInt::from(a).abs();
    }
}

/// The oracle's `interesting()` set must actually contain the boundary cases the
/// suite is meant to hit; guard against it silently shrinking.
#[test]
fn boundary_generators_cover_the_edges() {
    let v = interesting::<i128>();
    assert!(v.contains(&i128::MIN) && v.contains(&i128::MAX) && v.contains(&0) && v.contains(&-1));
    assert!(v.contains(&10_i128.pow(38)) && v.contains(&(5 * 10_i128.pow(37))));
    assert!(v.iter().any(|x| *x == (1i128 << 63) + 1) && v.iter().any(|x| *x == (1i128 << 126) - 1));
    let u = interesting::<u64>();
    assert!(u.contains(&u64::MAX) && u.contains(&10_u64.pow(19)) && u.contains(&(5 * 10_u64.pow(18))));
    assert!(v.len() > 1000, "i128 boundary set unexpectedly small: {}", v.len());
}

#[test]
fn product_edge_pairs_really_straddle_word_boundaries() {
    let mut rng = Rng(11);
    let pairs = product_edge_pairs::<i128>(&mut rng, 8);
    let near = |p: &BigInt, k: u32| (p.abs() - (BigInt::from(1u8) << k)).abs() <= BigInt::from(1u64 << 40);
    for k in [62u32, 63, 64, 126, 127] {
        assert!(
            pairs.iter().any(|(a, b)| near(&(a.big() * b.big()), k) && a.big().abs() > BigInt::from(1u8) << 8),
            "no pair with |a*b| near 2^{k}"
        );
    }
    // And ties: a*b mod 10^i == 10^i / 2 exactly.
    let ties = tie_pairs::<i64>(&mut rng, 2);
    for (a, b) in ties.iter().take(200) {
        let p = (a.big() * b.big()).abs();
        assert!((1..=18u32).any(|i| (&p % pow10(i)) == pow10(i - 1) * 5), "{a} * {b} is not a tie");
    }
}
