use std::fmt;

use super::{ParseError, Rounding};
pub fn parse_rounding(s: &str, kind: Rounding) -> Result<bool, ParseError> {
    if s.chars().any(|ch| ch.to_digit(10).is_none()) {
        return Err(ParseError::Invalid);
    }

    let is_carry = match kind {
        Rounding::Floor => false,
        Rounding::Ceil => !s.trim_matches('0').is_empty(),
        Rounding::Round => {
            if let Some(first) = s.chars().next() {
                first >= '5'
            } else {
                false
            }
        }
        Rounding::Unexpected => {
            if s.trim_matches('0').is_empty() {
                false
            } else {
                return Err(ParseError::Precision);
            }
        }
    };
    Ok(is_carry)
}

pub trait OobPrecDisplay {
    fn display_fmt(&self, precision: i32, f: &mut fmt::Formatter) -> Result<(), fmt::Error>;
}

pub struct OobFmt<D>(pub D, pub i32);

/// Format the decimal.
///
/// The tailing zeros of fraction are truncated by default, while the
/// precision can be specified by `{:.N}`.
///
/// # Examples:
///
/// ```
/// use std::str::FromStr;
/// use primitive_fixed_point_decimal::{OobPrecFpdec16, OobFmt};
/// type Decimal = OobPrecFpdec16;
/// let fd = Decimal::try_from_str("1.5670", 4).unwrap();
/// assert_eq!(&format!("{}", OobFmt(fd, 4)), "1.567"); // omit tailing zeros
/// assert_eq!(&format!("{:.2}", OobFmt(fd, 4)), "1.57"); // rounding
/// ```
impl<D> fmt::Display for OobFmt<D>
where D: OobPrecDisplay
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let precision = self.1;
        self.0.display_fmt(precision, f)
    }
}

macro_rules! rounding_div {
    ($lhs:expr, $rhs:expr, $rounding:ident) => {
        'a: {
            if $rhs == 0 {
                break 'a None;
            }
            let d = $lhs / $rhs;
            let r = $lhs % $rhs;
            let is_carry = match $rounding {
                Rounding::Floor => 0,
                Rounding::Ceil => if r == 0 { 0 } else { 1 },
                Rounding::Round => if r * 2 < $rhs { 0 } else { 1 },
                Rounding::Unexpected => if r == 0 { 0 } else { break 'a None; }
            };
            Some(d + is_carry)
        }
    }
}

macro_rules! convert_lower {
    ($from:expr, $lower_type:ty) => {
        match $from {
            None => None,
            Some(r) => {
                let lower = r as $lower_type;
                if r > 0 {
                    if lower <= <$lower_type>::MAX { Some(lower) } else { None }
                } else {
                    if lower >= <$lower_type>::MIN { Some(lower) } else { None }
                }
            }
        }
    }
}

pub(crate) use rounding_div;
pub(crate) use convert_lower;
