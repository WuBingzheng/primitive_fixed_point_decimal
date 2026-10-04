use num_traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, Num, One, Signed, Zero};
use primitive_fixed_point_decimal::ConstScaleFpdec;
type P = ConstScaleFpdec<i64, 4>;

/// Generic code can now bound on `Num` -- the point of the feature.
fn sum_all<T: Num + Copy>(xs: &[T]) -> T {
    xs.iter().fold(T::zero(), |a, &b| a + b)
}

#[test]
fn num_traits_surface_behaves() {
    assert_eq!(P::zero(), ConstScaleFpdec::from_mantissa(0));
    assert_eq!(P::one(), ConstScaleFpdec::from_mantissa(10_000)); // 1.0 at scale 4
    assert!(P::zero().is_zero());

    let a: P = "2.5".parse().unwrap();
    let b: P = "1.5".parse().unwrap();
    assert_eq!(sum_all(&[a, b]), "4.0".parse::<P>().unwrap());

    assert_eq!(CheckedAdd::checked_add(&a, &b), Some("4.0".parse().unwrap()));
    assert_eq!(CheckedMul::checked_mul(&a, &b), Some("3.75".parse().unwrap()));
    assert_eq!(CheckedDiv::checked_div(&a, &b), Some("1.6667".parse().unwrap()));
    assert_eq!(CheckedDiv::checked_div(&a, &P::zero()), None);

    // MIN has no negation in two's complement.
    assert_eq!(CheckedNeg::checked_neg(&P::min_value()), None);
    assert_eq!(CheckedMul::checked_mul(&P::max_value(), &P::max_value()), None);

    assert_eq!(Signed::abs(&-a), a);
    assert_eq!(Signed::signum(&-a), -P::one());
    assert!(Signed::is_negative(&-a));

    // Only radix 10 is meaningful for a decimal.
    assert!(P::from_str_radix("2.5", 10).is_ok());
    assert!(P::from_str_radix("2.5", 16).is_err());

    // The inherent, more-general checked_mul still wins method resolution.
    let scaled: ConstScaleFpdec<i64, 2> = a.checked_mul::<i64, 4, 2>(b).unwrap();
    assert_eq!(scaled, "3.75".parse().unwrap());
}
