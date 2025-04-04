use crate::define_common;
use crate::define_oob_exp;
use crate::utils::*;

use crate::define_oob_exp::parse_rounding; // XXX

// define and implement StaticFpdec16.
define_static::define_static_fpdec!(StaticFpdec16, i16, 16, 4);

impl<const P: u32> StaticFpdec16<P> {
    define_common::define_common!(StaticFpdec16, i16, 15);
}

// convert StaticFpdec16 into other StaticFpdec types.
//convert_into!(StaticFpdec16, static_fpdec32, StaticFpdec32);
//convert_into!(StaticFpdec16, static_fpdec64, StaticFpdec64);
//convert_into!(StaticFpdec16, static_fpdec128, StaticFpdec128);

// internal stuff needed by define_macro
const ALL_EXPS: [i16; 1 + 4] = [1,
    10_i16.pow(1), 10_i16.pow(2), 10_i16.pow(3), 10_i16.pow(4),
];

const fn calc_mul_div(a: i16, b: i16, c: i16, rounding: Rounding) -> Option<i16> {
    convert_lower!(rounding_div!(a as i32 * b as i32, c as i32, rounding), i16)
}

const fn calc_div_div(a: i16, b: i16, c: i16, rounding: Rounding) -> Option<i16> {
    convert_lower!(rounding_div!(a as i32, b as i32 * c as i32, rounding), i16)
}
