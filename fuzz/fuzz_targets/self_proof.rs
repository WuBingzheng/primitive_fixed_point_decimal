#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str::FromStr;

use primitive_fixed_point_decimal::{fpdec, ConstScaleFpdec};

type Dec128 = ConstScaleFpdec<i128, 8>;
type Dec64 = ConstScaleFpdec<i64, 5>;

#[derive(Debug, arbitrary::Arbitrary)]
struct Data {
    a: i128,
    b: i128,
    c: i64,
    d: i64,
}

fuzz_target!(|data: Data| {
    test_bit128(data.a, data.b);
    test_bit64(data.c, data.d);
});

fn test_bit128(data_a: i128, data_b: i128) {
    let a = Dec128::from_mantissa(data_a);
    let b = Dec128::from_mantissa(data_b);

    // check: a + b - a = b
    match a.checked_add(b) {
        Some(sum) => assert_eq!(sum - a, b),
        None => assert!(data_a.checked_add(data_b).is_none()),
    }

    // check: a * b / a ~= b
    //
    // let diff = (a * b / a - b).abs()
    // check: diff * a < 1
    if let Some(p) = a.checked_mul::<i128, 8, 8>(b) {
        if data_a != 0 {
            match p.checked_div(a) {
                Some(q) => {
                    let diff = q - b;
                    assert!((diff * a).abs() < fpdec!(1));
                }
                None => assert_big_128(data_b, 8),
            }
        }
        if data_b != 0 {
            match p.checked_div(b) {
                Some(q) => {
                    let diff = q - a;
                    assert!((diff * b).abs() < fpdec!(1));
                }
                None => assert_big_128(data_a, 8),
            }
        }
    }

    // check: a / b * b ~= a
    if let Some(q) = a.checked_div::<i128, 8, 8>(b) {
        match q.checked_mul(b) {
            Some(p) => {
                let diff = p - a;
                if let Some(b_abs) = b.checked_abs() {
                    assert!(diff.abs() * 2 < b_abs);
                }
            }
            None => (),
        }
    }

    // check string dump and load
    let s = format!("{a}");
    let a1 = Dec128::from_str(&s).unwrap();
    assert_eq!(a, a1);

    let s = format!("{:.4}", b);
    match Dec128::from_str(&s) {
        Ok(b1) => assert_eq!(b1, b / 10000 * 10000),
        Err(_) => assert_big_128(data_b, 4),
    }
}

fn test_bit64(data_a: i64, data_b: i64) {
    let a = Dec64::from_mantissa(data_a);
    let b = Dec64::from_mantissa(data_b);

    // check: a + b - a = b
    match a.checked_add(b) {
        Some(sum) => assert_eq!(sum - a, b),
        None => assert!(data_a.checked_add(data_b).is_none()),
    }

    // check: a * b / a ~= b
    //
    // let diff = (a * b / a - b).abs()
    // check: diff * a < 1
    if let Some(p) = a.checked_mul::<i64, 5, 5>(b) {
        if data_a != 0 {
            match p.checked_div(a) {
                Some(q) => {
                    let diff = q - b;
                    assert!((diff * a).abs() < fpdec!(1));
                }
                None => assert_big_64(data_b, 5),
            }
        }
        if data_b != 0 {
            match p.checked_div(b) {
                Some(q) => {
                    let diff = q - a;
                    assert!((diff * b).abs() < fpdec!(1));
                }
                None => assert_big_64(data_a, 5),
            }
        }
    }

    // check: a / b * b ~= a
    if let Some(q) = a.checked_div::<i64, 5, 5>(b) {
        match q.checked_mul(b) {
            Some(p) => {
                let diff = p - a;
                if let Some(b_abs) = b.checked_abs() {
                    assert!(diff.abs() * 2 < b_abs);
                }
            }
            None => (),
        }
    }

    // check string dump and load
    let s = format!("{a}");
    let a1 = Dec64::from_str(&s).unwrap();
    assert_eq!(a, a1);

    let s = format!("{:.2}", b);
    match Dec64::from_str(&s) {
        Ok(b1) => assert_eq!(b1, b / 1000 * 1000),
        Err(_) => assert_big_64(data_b, 3),
    }
}

fn assert_big_128(n: i128, ti: u32) {
    assert!(n.unsigned_abs() > i128::MAX as u128 - 10_u128.pow(ti))
}
fn assert_big_64(n: i64, ti: u32) {
    assert!(n.unsigned_abs() > i64::MAX as u64 - 10_u64.pow(ti))
}
