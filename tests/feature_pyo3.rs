use primitive_fixed_point_decimal::{ConstScaleFpdec, OobFmt};
use pyo3::prelude::*;
use pyo3::types::PyAnyMethods;
type P = ConstScaleFpdec<i128, 12>;

#[test]
fn decimal_crosses_the_boundary_exactly() {
    Python::attach(|py| {
        // Rust -> Python: an exact decimal.Decimal, never scientific notation.
        let d = P::from_mantissa(123_456_789_123_456_789_012);
        let obj = d.into_pyobject(py).unwrap();
        assert_eq!(obj.get_type().name().unwrap().to_string(), "Decimal");
        assert_eq!(obj.str().unwrap().to_string(), "123456789.123456789012");

        // Python -> Rust: exact round-trip, past f64's reach.
        assert_eq!(obj.extract::<P>().unwrap(), d);

        // A value Decimal would print as "1E-7" must still parse.
        let tiny = py.eval(c"__import__('decimal').Decimal('1E-7')", None, None).unwrap();
        assert_eq!(tiny.extract::<P>().unwrap(), P::from_mantissa(100_000));

        // str and int are accepted...
        let s = py.eval(c"'2.5'", None, None).unwrap();
        assert_eq!(s.extract::<P>().unwrap(), "2.5".parse::<P>().unwrap());
        let i = py.eval(c"7", None, None).unwrap();
        assert_eq!(i.extract::<P>().unwrap(), "7".parse::<P>().unwrap());

        // ...float and bool are not.
        let f = py.eval(c"0.1", None, None).unwrap();
        assert!(f.extract::<P>().is_err(), "float must be rejected");
        let b = py.eval(c"True", None, None).unwrap();
        assert!(b.extract::<P>().is_err(), "bool must be rejected");
    });
}

/// The values that sit at the seams of `format(x, "f")` + the string parser: every
/// spelling of zero, exponent forms, the limits, and inputs that must be an error
/// (never a panic, never a silently wrong value).
#[test]
fn edge_values_cross_the_boundary() {
    Python::attach(|py| {
        let ev = |code: &core::ffi::CStr| py.eval(code, None, None).unwrap();
        let zero = P::from_mantissa(0);
        for z in [
            c"__import__('decimal').Decimal('-0')",
            c"__import__('decimal').Decimal('0E-30')",
            c"__import__('decimal').Decimal('0E+30')",
            c"__import__('decimal').Decimal('-0.000')",
            c"'-0.000'",
            c"'00'",
            c"'+0'",
            c"0",
        ] {
            assert_eq!(ev(z).extract::<P>().unwrap(), zero, "{z:?}");
        }
        // exponent forms: Decimal('1E+3') is 1000, and Decimal('1.5E+1') is 15
        assert_eq!(ev(c"__import__('decimal').Decimal('1E+3')").extract::<P>().unwrap(), "1000".parse::<P>().unwrap());
        assert_eq!(ev(c"__import__('decimal').Decimal('1.5E+1')").extract::<P>().unwrap(), "15".parse::<P>().unwrap());
        // negative values
        assert_eq!(ev(c"-7").extract::<P>().unwrap(), "-7".parse::<P>().unwrap());
        assert_eq!(ev(c"'-0.5'").extract::<P>().unwrap(), "-0.5".parse::<P>().unwrap());

        // errors, not panics
        for bad in [
            c"__import__('decimal').Decimal('NaN')",
            c"__import__('decimal').Decimal('Infinity')",
            c"__import__('decimal').Decimal('-Infinity')",
            c"'abc'",
            c"''",
            c"'1.-5'", // a sign inside the fraction
            c"'1.+5'",
            c"None",
            c"[1]",
            c"10**40",                                    // too big for i128 at scale 12
            c"'0.0000000000001'",                         // 13 fraction digits at scale 12
            c"__import__('decimal').Decimal('1E-13')",
            c"1.5",                                        // float
        ] {
            assert!(ev(bad).extract::<P>().is_err(), "{bad:?} must be rejected");
        }

        // the limits round-trip exactly, in both directions
        for d in [P::MAX, P::MIN, P::EPSILON, P::ZERO, P::from_mantissa(-1)] {
            let obj = d.into_pyobject(py).unwrap();
            assert_eq!(obj.extract::<P>().unwrap(), d);
        }

        // OobFmt keeps the scale the Python value came with
        let o = ev(c"__import__('decimal').Decimal('1.50')").extract::<OobFmt<i128>>().unwrap();
        assert_eq!((o.0.mantissa(), o.1), (150, 2));
        let z = ev(c"__import__('decimal').Decimal('0.000')").extract::<OobFmt<i128>>().unwrap();
        assert_eq!((z.0.mantissa(), z.1), (0, 3));
    });
}

/// Regression: `format(Decimal("1E+999999999"), "f")` is a gigabyte string. The exponent is
/// checked first, so a hostile value is a quick `ValueError`, not an allocation.
#[test]
fn huge_exponents_are_rejected_before_formatting() {
    Python::attach(|py| {
        let ev = |code: &core::ffi::CStr| py.eval(code, None, None).unwrap();
        let start = std::time::Instant::now();
        for bad in [
            c"__import__('decimal').Decimal('1E+999999999')",
            c"__import__('decimal').Decimal('1E-999999999')",
            c"__import__('decimal').Decimal('0E-999999999')",
            c"__import__('decimal').Decimal('1E+1001')",
            c"10**1000000",
        ] {
            assert!(ev(bad).extract::<P>().is_err(), "{bad:?} must be rejected");
            assert!(ev(bad).extract::<OobFmt<i128>>().is_err(), "{bad:?} must be rejected");
        }
        assert!(start.elapsed().as_secs() < 5, "rejection must not build the huge string");
        // the largest meaningful ones still convert, or fail on range, as before
        assert_eq!(ev(c"__import__('decimal').Decimal('1E-12')").extract::<P>().unwrap(), P::from_mantissa(1));
        assert!(ev(c"__import__('decimal').Decimal('1E+38')").extract::<P>().is_err()); // too big for i128 at scale 12
        assert_eq!(ev(c"__import__('decimal').Decimal('0E+500')").extract::<P>().unwrap(), P::from_mantissa(0));
    });
}
