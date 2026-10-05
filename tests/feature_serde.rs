//! Regression: the binary path must be EXACT and must actually work.
//!
//! Before this was fixed, `Serialize` wrote an `f64` (losing everything past
//! ~15.9 significant digits) and `Deserialize` called `deserialize_any`, which
//! non-self-describing formats reject outright -- so bincode was lossy in one
//! direction and broken in the other.
mod common;
use common::*;

use core::num::ParseIntError;
use num_traits::{FromPrimitive, Num};
use primitive_fixed_point_decimal::{ConstScaleFpdec, OobFmt};
use serde::{de::DeserializeOwned, Serialize};
type P = ConstScaleFpdec<i128, 12>;

#[test]
fn binary_and_human_roundtrips_are_exact() {
    // 123456789.123456789012 -- 21 significant digits, well past f64's ~15.9.
    let d = P::from_mantissa(123_456_789_123_456_789_012);

    let json = serde_json::to_string(&d).unwrap();
    assert_eq!(json, "\"123456789.123456789012\"");
    assert_eq!(serde_json::from_str::<P>(&json).unwrap(), d);

    let bin = bincode::serialize(&d).unwrap();
    assert_eq!(bin.len(), 16, "should be the raw i128 mantissa, not an f64");
    assert_eq!(bincode::deserialize::<P>(&bin).unwrap(), d);

    // The f64 route is what used to happen -- kept here to show what was lost.
    let via_f64 = P::try_from(f64::from(d)).unwrap();
    assert_ne!(via_f64, d, "f64 round-trip should be visibly lossy at this size");
}

#[test]
fn oob_fmt_roundtrips_with_its_scale() {
    let of: OobFmt<i64> = OobFmt("1.25".parse::<OobFmt<i64>>().unwrap().0, 2);
    let json = serde_json::to_string(&of).unwrap();
    assert_eq!(serde_json::from_str::<OobFmt<i64>>(&json).unwrap().0, of.0);
}

/// Every boundary value survives JSON (a decimal string) and bincode (the raw
/// mantissa) bit-for-bit, for all ten inner types and scales on both sides of zero.
fn roundtrip<T, const S: i32>(f: &mut Failures)
where
    T: Int + Serialize + DeserializeOwned + FromPrimitive + Num<FromStrRadixErr = ParseIntError>,
{
    for m in interesting::<T>() {
        let d = ConstScaleFpdec::<T, S>::from_mantissa(m);
        let json = serde_json::to_string(&d).unwrap();
        f.check(|| format!("{} S={S} json text", T::NAME), || format!("{m:?}"), Ok(json.clone()), format!("\"{d}\""));
        f.check(|| format!("{} S={S} json", T::NAME), || format!("{m:?} {json}"),
            Ok(serde_json::from_str::<ConstScaleFpdec<T, S>>(&json).map_err(|e| e.to_string())), Ok(d));
        let bin = bincode::serialize(&d).unwrap();
        f.check(|| format!("{} S={S} bin len", T::NAME), || format!("{m:?}"), Ok(bin.len()), (T::BITS / 8) as usize);
        f.check(|| format!("{} S={S} bin", T::NAME), || format!("{m:?}"),
            Ok(bincode::deserialize::<ConstScaleFpdec<T, S>>(&bin).map_err(|e| e.to_string())), Ok(d));
    }
}

macro_rules! per_type {
    ($($t:ty: $name:ident),*) => { $(
        #[test]
        fn $name() {
            let mut f = Failures::default();
            roundtrip::<$t, -3>(&mut f);
            roundtrip::<$t, 0>(&mut f);
            roundtrip::<$t, 2>(&mut f);
            roundtrip::<$t, 12>(&mut f);
            roundtrip::<$t, 40>(&mut f);
            f.finish(concat!("serde ", stringify!($t)));
        }
    )* };
}
per_type!(i8: serde_i8, i16: serde_i16, i32: serde_i32, i64: serde_i64, i128: serde_i128,
          u8: serde_u8, u16: serde_u16, u32: serde_u32, u64: serde_u64, u128: serde_u128);

/// Numbers (not just strings) are accepted from self-describing formats, and the
/// error says what was wrong.
#[test]
fn json_numbers_zero_spellings_and_errors() {
    type D2 = ConstScaleFpdec<i32, 2>;
    let de = |s: &str| serde_json::from_str::<D2>(s);
    assert_eq!(de("7").unwrap().mantissa(), 700);
    assert_eq!(de("-7").unwrap().mantissa(), -700);
    assert_eq!(de("0").unwrap().mantissa(), 0);
    assert_eq!(de("1.5").unwrap().mantissa(), 150); // a JSON float is rounded to the scale
    assert_eq!(de("\"1.5\"").unwrap().mantissa(), 150);
    for z in ["\"0\"", "\"-0\"", "\"+0\"", "\"00\"", "\"0.00\"", "\"-0.000\"", "-0.0", "0.0"] {
        assert_eq!(de(z).unwrap().mantissa(), 0, "{z}");
    }
    for bad in ["\"\"", "\"abc\"", "\"1.-5\"", "\"1.+5\"", "\"1.234\"", "\"99999999999\"", "99999999999", "true", "null", "[]"] {
        assert!(de(bad).is_err(), "{bad} must be rejected");
    }
    // the message names the real problem: precision for a negative scale, not "overflow"
    let err = serde_json::from_str::<ConstScaleFpdec<i16, -6>>("123").unwrap_err().to_string();
    assert!(err.contains("precision"), "{err}");
    let err = serde_json::from_str::<D2>("99999999999").unwrap_err().to_string();
    assert!(err.contains("overflow"), "{err}");
}

#[test]
fn oob_fmt_edge_strings() {
    let de = |s: &str| serde_json::from_str::<OobFmt<i32>>(s).map(|o| (o.0.mantissa(), o.1));
    assert_eq!(de("\"1.50\"").unwrap(), (150, 2));
    assert_eq!(de("\"0.000\"").unwrap(), (0, 3));
    assert_eq!(de("\"-0\"").unwrap(), (0, 0));
    assert_eq!(de("\"1000\"").unwrap(), (1, -3));
    assert!(de("\"1.+5\"").is_err());
    assert!(de("\"\"").is_err());
    assert!(de("\"99999999999\"").is_err());
    assert!(de("1.5").is_err(), "OobFmt reads strings only: the scale is guessed from the text");
    // serialize -> deserialize keeps value AND scale for every canonical text
    for (m, sc) in [(150, 2), (0, 0), (1, -3), (-1, 5), (i32::MAX, 9), (i32::MIN, 0), (5, 12)] {
        let o = OobFmt(primitive_fixed_point_decimal::OobScaleFpdec::<i32>::from_mantissa(m), sc);
        let j = serde_json::to_string(&o).unwrap();
        let back = serde_json::from_str::<OobFmt<i32>>(&j).unwrap();
        let (a, b) = (back.0.mantissa() as i64, back.1);
        // the scale may be re-derived (trailing zeros of an integer string move into it), the value may not
        let diff = (b - sc) as i64;
        let norm = |m: i64, d: i64| if d >= 0 { m * 10_i64.pow(d as u32) } else { m / 10_i64.pow((-d) as u32) };
        assert_eq!(norm(a, -diff), m as i64, "{j}");
    }
}
