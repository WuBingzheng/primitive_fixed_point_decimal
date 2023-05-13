mod fixdec128;
mod fixdec64;
//mod fixdec32;


pub use crate::fixdec128::DIGITS as FIXDEC128_DIGITS;
pub use crate::fixdec128::MAX_PRECISION as FIXDEC128_MAX_PRECISION;

pub use crate::fixdec128::FixDec128;
pub use crate::fixdec64::FixDec64;
//pub use crate::fixdec32::FixDec32;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Empty,
    Invalid,
    Overflow,
}

#[derive(Debug, PartialEq)]
pub enum Rounding {
    Round,
    Ceil,
    Floor,
}
