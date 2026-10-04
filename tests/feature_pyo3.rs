use primitive_fixed_point_decimal::ConstScaleFpdec;
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
