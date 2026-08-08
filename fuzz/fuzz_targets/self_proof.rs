#![no_main]

use libfuzzer_sys::fuzz_target;
use num_traits::Num;
use primitive_types::U256;
use std::fmt::{Debug, Display};
use std::num::ParseIntError;

use primitive_fixed_point_decimal::{FpdecInner, OobFmt, OobScaleFpdec, Rounding};

#[derive(Debug, arbitrary::Arbitrary)]
struct Data {
    a: u128,
    b: u128,
    c: u64,
    d: u64,
}

fuzz_target!(|data: Data| {
    test_decimal(data.a, data.b);
    test_decimal(data.a as i128, data.b as i128);
    test_decimal(data.c, data.d);
    test_decimal(data.c as i64, data.d as i64);
});

fn test_decimal<I>(data_a: I, data_b: I)
where
    I: FpdecInner + Debug + Display + Num<FromStrRadixErr = ParseIntError> + RawMulDiv,
{
    let a = OobScaleFpdec::from_mantissa(data_a);
    let b = OobScaleFpdec::from_mantissa(data_b);

    // check: a + b - a = b
    match a.checked_add(b) {
        Some(sum) => assert_eq!(sum - a, b),
        None => assert!(data_a.checked_add(&data_b).is_none()),
    }

    const DIFF_SCALES: [i32; 14] = [-5, -2, 0, 1, 2, 4, 5, 8, 13, 18, 19, 31, 32, 38];

    // check: a * b
    // check: a / b
    for diff_scale in DIFF_SCALES {
        if diff_scale > I::DIGITS as i32 {
            continue;
        }
        // dbg!(a, b, diff_scale, I::MIN);
        assert_eq!(
            a.checked_mul(b, diff_scale).map(|d| d.mantissa()),
            data_a.raw_mul(data_b, diff_scale)
        );

        assert_eq!(
            a.checked_div(b, diff_scale).map(|d| d.mantissa()),
            data_a.raw_div(data_b, diff_scale)
        );
    }

    // check: a / b * b ~= a
    for diff_scale in DIFF_SCALES {
        if let Some(q) = a.checked_div(b, diff_scale) {
            if let Some(p) = q.checked_mul(b, -diff_scale) {
                // -b/2 <= diff <= b/2
                // -b <= diff * 2 <= b
                // abs(diff) * 2 <= abs(b)

                // dbg!(p, a, b, q);
                let diff = if (b >= OobScaleFpdec::ZERO) == (p > a) {
                    match p.checked_sub(a) {
                        Some(d) => d,
                        None => continue,
                    }
                } else {
                    match a.checked_sub(p) {
                        Some(d) => d,
                        None => continue,
                    }
                };

                if diff_scale > 0 {
                    let diff = diff
                        .checked_mul_ext(
                            OobScaleFpdec::<I>::EPSILON,
                            diff_scale,
                            Rounding::TowardsZero,
                        )
                        .unwrap();
                    // dbg!(a, b, diff, diff_scale);
                    if diff + diff != b {
                        assert_eq!(diff + diff <= b, b >= OobScaleFpdec::ZERO);
                    }
                } else {
                    let b = b
                        .checked_mul_ext(
                            OobScaleFpdec::<I>::EPSILON,
                            -diff_scale,
                            Rounding::AwayFromZero,
                        )
                        .unwrap();
                    // dbg!(a, b, diff, diff_scale);
                    if diff + diff != b {
                        assert_eq!(diff + diff <= b, b >= OobScaleFpdec::ZERO);
                    }
                }
            }
        }
    }

    // check string dump and load
    let s = format!("{}", OobFmt(a, 8));
    let a1 = OobScaleFpdec::try_from_str(&s, 8).unwrap();
    assert_eq!(a, a1);

    let s = format!("{:.4}", OobFmt(b, 8));
    match OobScaleFpdec::try_from_str(&s, 8) {
        Ok(b1) => assert_eq!(b1, b.round_diff(4)),
        Err(_) => assert_bigbig(data_b, 4),
    }
}

trait RawMulDiv: Sized {
    fn raw_mul(self, b: Self, i: i32) -> Option<Self>;
    fn raw_div(self, b: Self, i: i32) -> Option<Self>;
}

impl RawMulDiv for i64 {
    fn raw_mul(self, b: i64, i: i32) -> Option<i64> {
        let a = self as i128;
        let b = b as i128;

        let p = a * b;
        if i > 0 {
            rounding_div(p, 10i128.pow(i as u32))
        } else if i < 0 {
            p.checked_mul(10i128.pow((-i) as u32))?
        } else {
            p
        }
        .try_into()
        .ok()
    }
    fn raw_div(self, b: i64, i: i32) -> Option<i64> {
        let a = self as i128;
        let b = b as i128;
        if b == 0 {
            return None;
        }

        if i > 0 {
            let exp = 10i128.pow(i as u32);
            rounding_div(a, b * exp)
        } else if i < 0 {
            let exp = 10i128.pow((-i) as u32);
            rounding_div(a * exp, b)
        } else {
            rounding_div(a, b)
        }
        .try_into()
        .ok()
    }
}

impl RawMulDiv for u64 {
    fn raw_mul(self, b: u64, i: i32) -> Option<u64> {
        let a = self as u128;
        let b = b as u128;

        let p = a * b;
        if i > 0 {
            rounding_div(p, 10u128.pow(i as u32))
        } else if i < 0 {
            p.checked_mul(10u128.pow((-i) as u32))?
        } else {
            p
        }
        .try_into()
        .ok()
    }
    fn raw_div(self, b: u64, i: i32) -> Option<u64> {
        let a = self as u128;
        let b = b as u128;
        if b == 0 {
            return None;
        }

        if i > 0 {
            let exp = 10u128.pow(i as u32);
            rounding_div(a, b * exp)
        } else if i < 0 {
            let exp = 10u128.pow((-i) as u32);
            rounding_div(a * exp, b)
        } else {
            rounding_div(a, b)
        }
        .try_into()
        .ok()
    }
}

impl RawMulDiv for i128 {
    fn raw_mul(self, b: i128, i: i32) -> Option<i128> {
        let ua = self.unsigned_abs();
        let ub = b.unsigned_abs();

        let up = ua.raw_mul(ub, i)?;
        if self ^ b >= 0 {
            up.try_into().ok()
        } else if up == i128::MAX as u128 + 1 {
            Some(i128::MIN)
        } else {
            up.try_into().ok().map(|x: i128| -x)
        }
    }
    fn raw_div(self, b: i128, i: i32) -> Option<i128> {
        let ua = self.unsigned_abs();
        let ub = b.unsigned_abs();

        let uq = ua.raw_div(ub, i)?;

        if self ^ b >= 0 {
            uq.try_into().ok()
        } else if uq == i128::MAX as u128 + 1 {
            Some(i128::MIN)
        } else {
            uq.try_into().ok().map(|x: i128| -x)
        }
    }
}

impl RawMulDiv for u128 {
    fn raw_mul(self, b: u128, i: i32) -> Option<u128> {
        let a = U256::from(self);
        let b = U256::from(b);

        let p = a * b;
        if i > 0 {
            rounding_div_u256(p, U256::exp10(i as usize))
        } else if i < 0 {
            p.checked_mul(U256::exp10((-i) as usize))?
        } else {
            p
        }
        .try_into()
        .ok()
    }
    fn raw_div(self, b: u128, i: i32) -> Option<u128> {
        if b == 0 {
            return None;
        }

        let a = U256::from(self);
        let b = U256::from(b);

        if i > 0 {
            let exp = U256::exp10(i as usize);
            rounding_div_u256(a, b * exp)
        } else if i < 0 {
            let exp = U256::exp10((-i) as usize);
            rounding_div_u256(a * exp, b)
        } else {
            rounding_div_u256(a, b)
        }
        .try_into()
        .ok()
    }
}

fn assert_bigbig<I>(n: I, ti: i32)
where
    I: FpdecInner + Debug,
{
    let mut power = I::ONE;
    for _ in 0..ti {
        power = power * I::TEN;
    }

    if n > I::ZERO {
        assert!(n >= I::MAX - power)
    } else {
        assert!(n <= I::MIN + power)
    }
}

fn rounding_div<I>(a: I, b: I) -> I
where
    I: FpdecInner + Debug,
{
    let q = a / b;
    let remain = a % b;
    if remain == I::ZERO {
        return q;
    }

    if (a ^ b) >= I::ZERO {
        // unsigned types, or signed types and self and b have same sign
        if I::MIN < I::ZERO {
            if b > I::ZERO {
                if (remain - (b - remain)) >= I::ZERO {
                    q + I::ONE
                } else {
                    q
                }
            } else {
                if (remain - (b - remain)) <= I::ZERO {
                    q + I::ONE
                } else {
                    q
                }
            }
        } else {
            // unsigned type
            if remain >= b - remain {
                q + I::ONE
            } else {
                q
            }
        }
    } else {
        // signed types and self and b have different sign.
        let r = remain.unsigned_abs();
        if r >= b.unsigned_abs() - r {
            q - I::ONE
        } else {
            q
        }
    }
}

fn rounding_div_u256(a: U256, b: U256) -> U256 {
    let q = a / b;
    let remain = a % b;
    if remain.is_zero() {
        return q;
    }

    if remain >= b - remain {
        q + U256::one()
    } else {
        q
    }
}
