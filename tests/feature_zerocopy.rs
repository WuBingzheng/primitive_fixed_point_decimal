//! `FromBytes` is derivable here because the type has NO range invariant
//! (`FpdecInner::MAX == I::MAX`), so every bit pattern is a valid decimal.
use primitive_fixed_point_decimal::ConstScaleFpdec;
use zerocopy::{FromBytes, IntoBytes};
type P = ConstScaleFpdec<i128, 12>;

#[test]
fn bytes_roundtrip_without_copying() {
    let d = P::from_mantissa(123_456_789_123_456_789_012);
    let bytes = d.as_bytes();
    assert_eq!(bytes.len(), 16);
    assert_eq!(P::read_from_bytes(bytes).unwrap(), d);

    // Any 16 bytes are a valid value -- this is the property that makes the
    // derive sound, so assert it rather than assume it.
    assert!(P::read_from_bytes(&[0xFFu8; 16]).is_ok());

    // Zero-copy view over a buffer of many decimals.
    let all = [P::from_mantissa(1), P::from_mantissa(-2), P::from_mantissa(3)];
    let raw = all.as_bytes();
    let view = <[P]>::ref_from_bytes(raw).unwrap();
    assert_eq!(view, &all[..]);
}
