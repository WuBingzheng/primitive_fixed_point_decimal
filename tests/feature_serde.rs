//! Regression: the binary path must be EXACT and must actually work.
//!
//! Before this was fixed, `Serialize` wrote an `f64` (losing everything past
//! ~15.9 significant digits) and `Deserialize` called `deserialize_any`, which
//! non-self-describing formats reject outright -- so bincode was lossy in one
//! direction and broken in the other.
use primitive_fixed_point_decimal::{ConstScaleFpdec, OobFmt};
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
