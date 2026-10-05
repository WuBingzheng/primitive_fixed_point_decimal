//! PyO3 interop (feature = `pyo3`).
//!
//! Lets [`ConstScaleFpdec`] and [`OobFmt`] cross the Rust<->Python boundary as
//! native `decimal.Decimal` objects, so they work directly as `#[pyfunction]`
//! arguments and return values.
//!
//! * **Rust -> Python**: an exact `decimal.Decimal` built from the value's
//!   `Display` form, which never uses scientific notation. Nothing is lost.
//! * **Python -> Rust**: accepts `decimal.Decimal`, `str`, and `int`. Each is
//!   read via its fixed-point string form and parsed exactly.
//!
//! # `float` is rejected
//!
//! Unlike the integer and string paths, a Python `float` cannot represent most
//! decimals -- it holds ~15.9 significant digits where `ConstScaleFpdec<i128,
//! 12>` holds up to 39. Accepting one would silently round money on the way in,
//! and whether it rounded would depend on the literal a caller happened to type.
//! Pass a `Decimal` or a string. (Whole numbers arrive exactly as `int`.)
//!
//! `bool` is rejected too: it is an `int` subclass in Python, so `True` would
//! otherwise be silently accepted as `1`.
//!
//! # `OobScaleFpdec`
//!
//! Not convertible on its own -- its scale is out of band, so there is nothing
//! to build a `Decimal` from. Use [`OobFmt`], which pairs the value with its
//! scale, exactly as the `serde` and `Display` impls do.

use core::num::ParseIntError;
use core::str::FromStr;

use std::string::{String, ToString};

use num_traits::{cast::FromPrimitive, Num};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyBool, PyFloat, PyInt, PyType};

use crate::const_scale_fpdec::ConstScaleFpdec;
use crate::fpdec_inner::FpdecInner;
use crate::oob_scale_fpdec::OobFmt;

/// The `decimal.Decimal` class, imported once per interpreter and cached.
/// Avoids a per-conversion `import("decimal")` on every call.
fn decimal_type(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    static DECIMAL: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    DECIMAL.import(py, "decimal", "Decimal")
}

/// Builds a Python `decimal.Decimal` from an exact decimal string.
fn to_py_decimal<'py>(py: Python<'py>, s: &str) -> PyResult<Bound<'py, PyAny>> {
    decimal_type(py)?.call1((s,))
}

/// Normalizes a Python `Decimal`, `str` or `int` to a fixed-point decimal string
/// (never scientific).
///
/// `str(Decimal("0.0000001"))` is `"1E-7"`, which the parsers reject;
/// `format(x, "f")` yields `"0.0000001"` instead, so every representable value
/// round-trips.
fn fixed_point_string(ob: Borrowed<'_, '_, PyAny>) -> PyResult<String> {
    if ob.is_instance_of::<PyBool>() {
        return Err(PyValueError::new_err("bool is not a valid decimal value"));
    }
    if ob.is_instance_of::<PyFloat>() {
        return Err(PyValueError::new_err(
            "refusing to convert a float to a fixed-point decimal (precision \
             would be lost); pass a decimal.Decimal, a string, or an int",
        ));
    }
    // `Decimal(int)` is quadratic in the digits (seconds for `10**1_000_000`), so a huge int is
    // refused before it is converted. 4096 bits is ~1200 digits, far past any fixed-point range.
    if ob.is_instance_of::<PyInt>() && ob.call_method0("bit_length")?.extract::<u64>()? > MAX_INT_BITS {
        return Err(PyValueError::new_err("integer out of range"));
    }
    let decimal = decimal_type(ob.py())?;
    // An EXACT `Decimal` is used as is: a subclass may override `__format__` and return
    // something that is not the numeric value, so anything else is converted first.
    let dec = if ob.get_type().is(decimal) {
        ob.to_owned()
    } else {
        decimal.call1((ob,))?
    };
    // `format(x, "f")` writes every digit, so `Decimal("1E+999999999")` (one line of Python, and
    // possibly untrusted input) would build a gigabyte string only to fail the range check.
    // Nothing a fixed-point type holds is anywhere near this long.
    let adjusted = dec.call_method0("adjusted")?.extract::<i64>()?;
    if adjusted.unsigned_abs() > MAX_ADJUSTED {
        return Err(PyValueError::new_err("decimal exponent out of range"));
    }
    dec.call_method1("__format__", ("f",))?.extract::<String>()
}

/// `|Decimal.adjusted()|` above which a value cannot be a fixed-point decimal (at most 39
/// integer digits and 200 fraction digits are ever meaningful).
const MAX_ADJUSTED: u64 = 1000;

/// `int.bit_length()` above which an int cannot be a fixed-point decimal.
const MAX_INT_BITS: u64 = 4096;

// ---------------------------------------------------------------------------
// ConstScaleFpdec
// ---------------------------------------------------------------------------

impl<'py, I, const S: i32> IntoPyObject<'py> for ConstScaleFpdec<I, S>
where
    I: FpdecInner + core::fmt::Display,
{
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        to_py_decimal(py, &self.to_string())
    }
}

impl<'py, I, const S: i32> IntoPyObject<'py> for &ConstScaleFpdec<I, S>
where
    I: FpdecInner + core::fmt::Display,
{
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'a, 'py, I, const S: i32> FromPyObject<'a, 'py> for ConstScaleFpdec<I, S>
where
    I: FpdecInner + FromPrimitive + Num<FromStrRadixErr = ParseIntError>,
{
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let s = fixed_point_string(ob)?;
        Self::from_str(&s).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// OobFmt -- the value paired with its out-of-band scale
// ---------------------------------------------------------------------------

impl<'py, I> IntoPyObject<'py> for OobFmt<I>
where
    I: FpdecInner + core::fmt::Display,
{
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        to_py_decimal(py, &self.to_string())
    }
}

/// Note the asymmetry with [`ConstScaleFpdec`]: a `Decimal` carries its own
/// scale, so extraction produces an `OobFmt` whose scale is whatever the Python
/// value had. Call [`OobFmt::rescale`] to move it onto the scale your domain
/// expects.
impl<'a, 'py, I> FromPyObject<'a, 'py> for OobFmt<I>
where
    I: FpdecInner + FromPrimitive + Num<FromStrRadixErr = ParseIntError>,
{
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let s = fixed_point_string(ob)?;
        Self::from_str(&s).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}
