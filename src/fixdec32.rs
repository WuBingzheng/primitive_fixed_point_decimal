use crate::define_macro::*;
use crate::utils::*;

// define and implement FixDec32.
define_fixdec!(FixDec32, i32, 9, 32, 31);

// convert FixDec32 into other FixDec types.
convert_try_into!(FixDec32, fixdec16, FixDec16);
convert_into!(FixDec32, fixdec64, FixDec64);
convert_into!(FixDec32, fixdec128, FixDec128);

// internal stuff needed by define_macro
const ALL_EXPS: [i32; 10] = [1,
    10_i32.pow(1), 10_i32.pow(2), 10_i32.pow(3), 10_i32.pow(4),
    10_i32.pow(5), 10_i32.pow(6), 10_i32.pow(7), 10_i32.pow(8),
    10_i32.pow(9)
];

const fn calc_mul_div(a: i32, b: i32, c: i32, rounding: Rounding) -> Option<i32> {
    convert_lower!(rounding_div!(a as i64 * b as i64, c as i64, rounding), i32)
}

const fn calc_div_div(a: i32, b: i32, c: i32, rounding: Rounding) -> Option<i32> {
    convert_lower!(rounding_div!(a as i64, b as i64 * c as i64, rounding), i32)
}
