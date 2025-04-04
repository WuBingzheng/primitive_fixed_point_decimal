use crate::define_common;
use crate::define_static_prec;
use crate::define_oob_prec;
use crate::utils::*;

define_static_prec::define_static_prec_fpdec!(StaticPrecFpdec16, i16, 4, 16, 15, StaticPrecFpdec16::<4>);

// define and implement OobPrecFpdec16.
define_oob_prec::define_oob_prec_fpdec!(OobPrecFpdec16, i16, 4, 16, 15);

// convert OobPrecFpdec16 into other OobPrecFpdec types.
//convert_into!(OobPrecFpdec16, oob_prec_fpdec32, OobPrecFpdec32);
//convert_into!(OobPrecFpdec16, oob_prec_fpdec64, OobPrecFpdec64);
//convert_into!(OobPrecFpdec16, oob_prec_fpdec128, OobPrecFpdec128);

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
