use crate::define_common;
use crate::define_static_prec;
use crate::utils::*;

// define and implement StaticPrecFpdec16.
define_static_prec::define_static_prec_fpdec!(StaticPrecFpdec16, i16, 16, 4);

impl<const P: i32> StaticPrecFpdec16<P> {
    define_common::define_common!(StaticPrecFpdec16, i16, 15, StaticPrecFpdec16::<4>);
}

// convert StaticPrecFpdec16 into other StaticPrecFpdec types.
//convert_into!(StaticPrecFpdec16, static_prec_fpdec32, StaticPrecFpdec32);
//convert_into!(StaticPrecFpdec16, static_prec_fpdec64, StaticPrecFpdec64);
//convert_into!(StaticPrecFpdec16, static_prec_fpdec128, StaticPrecFpdec128);

// internal stuff needed by define_static_prec_fpdec! macro.
const ALL_EXPS: [i16; 1 + 4] = [1,
    10_i16.pow(1), 10_i16.pow(2), 10_i16.pow(3), 10_i16.pow(4),
];

const fn calc_mul_div(a: i16, b: i16, c: i16, rounding: Rounding) -> Option<i16> {
    convert_lower!(rounding_div!(a as i32 * b as i32, c as i32, rounding), i16)
}

const fn calc_div_div(a: i16, b: i16, c: i16, rounding: Rounding) -> Option<i16> {
    convert_lower!(rounding_div!(a as i32, b as i32 * c as i32, rounding), i16)
}
