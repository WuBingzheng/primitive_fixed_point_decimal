//! `checked_mul_ext` / `checked_div_ext` with an `rhs` of a DIFFERENT inner integer.
//!
//! The result keeps `self`'s type, so `rhs` was converted to it with `I::from(rhs)?` -- which
//! is `None` whenever `rhs` does not fit, even if the RESULT does: a small `i16` times a large
//! `i64` (`0.0001 * 1000.00`) is `0.1`, not an overflow. Checked against the same BigInt
//! oracle as `tests/oracle.rs`, over every pair of the ten inner types.
mod common;
use common::*;

use primitive_fixed_point_decimal::OobScaleFpdec as D;

fn sample<T: Int>(n: usize) -> Vec<T> {
    let all = interesting::<T>();
    let stride = (all.len() / n).max(1);
    all.into_iter().step_by(stride).collect()
}

fn run<I: Int, J: Int>(f: &mut Failures) {
    let (xs, ys) = (sample::<I>(14), sample::<J>(14));
    let diffs = [-40, -20, -9, -3, -1, 0, 1, 2, 5, 9, 20, 38, 39, 40];
    for &a in &xs {
        for &b in &ys {
            for diff in diffs {
                for r in ROUNDINGS {
                    let at = || format!("{a} {b} diff={diff} {r:?}");
                    let (da, db) = (D::<I>::from_mantissa(a), D::<J>::from_mantissa(b));
                    f.check(|| format!("{} x {} mul", I::NAME, J::NAME), at,
                        guarded(|| da.checked_mul_ext(db, diff, r).map(|d| d.mantissa())), spec_mul_mixed(a, b, diff, r));
                    f.check(|| format!("{} / {} div", I::NAME, J::NAME), at,
                        guarded(|| da.checked_div_ext(db, diff, r).map(|d| d.mantissa())), spec_div_mixed(a, b, diff, r));
                }
            }
        }
    }
}

macro_rules! per_self {
    ($($i:ty: $name:ident),*) => { $(
        #[test]
        fn $name() {
            let mut f = Failures::default();
            run::<$i, i8>(&mut f); run::<$i, i16>(&mut f); run::<$i, i32>(&mut f); run::<$i, i64>(&mut f); run::<$i, i128>(&mut f);
            run::<$i, u8>(&mut f); run::<$i, u16>(&mut f); run::<$i, u32>(&mut f); run::<$i, u64>(&mut f); run::<$i, u128>(&mut f);
            f.finish(concat!("mixed rhs, self = ", stringify!($i)));
        }
    )* };
}
per_self!(i8: self_i8, i16: self_i16, i32: self_i32, i64: self_i64, i128: self_i128,
          u8: self_u8, u16: self_u16, u32: self_u32, u64: self_u64, u128: self_u128);

/// The case that was reported, through the typed (`ConstScaleFpdec`) API.
#[test]
fn small_times_large_is_not_an_overflow() {
    use primitive_fixed_point_decimal::ConstScaleFpdec as C;
    let a = C::<i16, 4>::from_mantissa(1); // 0.0001
    let b = C::<i64, 2>::from_mantissa(100_000); // 1000.00
    assert_eq!(a.checked_mul::<i64, 2, 4>(b), Some(C::<i16, 4>::from_mantissa(1000))); // 0.1000
    assert_eq!(a.checked_div::<i64, 2, 4>(b), Some(C::<i16, 4>::from_mantissa(0)));
    // the result itself must still fit
    assert_eq!(C::<i16, 4>::from_mantissa(30_000).checked_mul::<i64, 2, 4>(b), None);
    // zero is representable whatever the other operand
    assert_eq!(C::<u32, 2>::from_mantissa(0).checked_mul::<i64, 2, 2>(C::<i64, 2>::from_mantissa(-5_000_000_000)), Some(C::<u32, 2>::from_mantissa(0)));
    // ... and a negative result is not, for an unsigned self
    assert_eq!(C::<u32, 2>::from_mantissa(5).checked_mul::<i64, 2, 2>(C::<i64, 2>::from_mantissa(-5_000_000_000)), None);
}
