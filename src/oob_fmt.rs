use std::fmt;

pub trait OobPrecDisplay {
    fn display_fmt(&self, precision: i32, f: &mut fmt::Formatter) -> Result<(), fmt::Error>;
}

/// TODO
pub struct OobFmt<D>(pub D, pub i32);

/// Format the decimal.
///
/// The tailing zeros of fraction are truncated.
///
/// # Examples:
///
/// ```
/// use std::str::FromStr;
/// use primitive_fixed_point_decimal::{OobPrecFpdec16, OobFmt};
/// type Decimal = OobPrecFpdec16;
/// let fd = Decimal::try_from_str("1.5670", 4).unwrap();
/// assert_eq!(&format!("{}", OobFmt(fd, 4)), "1.567"); // omit tailing zeros
/// ```
impl<D> fmt::Display for OobFmt<D>
where D: OobPrecDisplay
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let precision = self.1;
        self.0.display_fmt(precision, f)
    }
}
