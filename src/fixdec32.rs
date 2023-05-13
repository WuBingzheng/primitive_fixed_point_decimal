// define and implement FixDec32.
use crate::define_macro::define_fixdec;
define_fixdec!(FixDec32, i32, 9, 32, 31);

// convert FixDec32 into other FixDec types.
use crate::define_macro::{convert_into, convert_try_into};
convert_try_into!(FixDec32, fixdec16, FixDec16);
convert_into!(FixDec32, fixdec64, FixDec64);
convert_into!(FixDec32, fixdec128, FixDec128);

// need by define_fixdec
const ALL_EXPS: [i32; 10] = [1,
    10_i32.pow(1), 10_i32.pow(2), 10_i32.pow(3), 10_i32.pow(4),
    10_i32.pow(5), 10_i32.pow(6), 10_i32.pow(7), 10_i32.pow(8),
    10_i32.pow(9)
];

const fn calc_mul_div(a: i32, b: i32, c: i32) -> Option<i32> {
    if c == 0 {
        None
    } else {
        Some((a as i64 * b as i64 / c as i64) as i32)
    }
}
