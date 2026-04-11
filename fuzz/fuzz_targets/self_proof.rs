#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str::FromStr;

use primitive_fixed_point_decimal::ConstScaleFpdec;

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
    // 128-bit
    let a = Dec128::from_mantissa(data.a);
    let b = Dec128::from_mantissa(data.b);
    let _ = a.checked_add(a);
    let _ = a.checked_sub(a);
    let _: Option<Dec128> = a.checked_mul(a);
    let _: Option<Dec128> = a.checked_div(a);

    let s = format!("{a}");
    let a1 = Dec128::from_str(&s).unwrap();
    assert_eq!(a, a1);
    let s = format!("{:.4}", b);
    match Dec128::from_str(&s) {
        Ok(b1) => assert_eq!(b1, b / 10000 * 10000),
        Err(_) => assert!(data.b.unsigned_abs() > i128::MAX as u128 - 10000),
    }

    // 64-bit
    let a = Dec64::from_mantissa(data.c);
    let b = Dec64::from_mantissa(data.d);
    let _ = a.checked_add(a);
    let _ = a.checked_sub(a);
    let _: Option<Dec64> = a.checked_mul(a);
    let _: Option<Dec64> = a.checked_div(a);

    let s = format!("{a}");
    let a1 = Dec64::from_str(&s).unwrap();
    assert_eq!(a, a1);
    let s = format!("{:.2}", b);
    match Dec64::from_str(&s) {
        Ok(b1) => assert_eq!(b1, b / 1000 * 1000),
        Err(_) => assert!(data.d.unsigned_abs() > i64::MAX as u64 - 1000),
    }
});
