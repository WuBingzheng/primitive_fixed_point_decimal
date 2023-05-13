// define and implement FixDec16.
use crate::define_macro::define_fixdec;
define_fixdec!(FixDec16, i16, 4, 16, 15);

// convert FixDec16 into other FixDec types.
use crate::define_macro::convert_into;
convert_into!(FixDec16, fixdec32, FixDec32);
convert_into!(FixDec16, fixdec64, FixDec64);
convert_into!(FixDec16, fixdec128, FixDec128);

// need by define_fixdec
const ALL_EXPS: [i16; 1 + 4] = [1,
    10_i16.pow(1), 10_i16.pow(2), 10_i16.pow(3), 10_i16.pow(4),
];

const fn calc_mul_div(a: i16, b: i16, c: i16) -> Option<i16> {
    if c == 0 {
        None
    } else {
        Some((a as i32 * b as i32 / c as i32) as i16)
    }
}
