// define and implement FixDec128.
use crate::define_macro::define_fixdec;
define_fixdec!(FixDec128, i128, 38, 128, 127);

// convert FixDec64 into other FixDec types.
use crate::define_macro::convert_try_into;
convert_try_into!(FixDec128, fixdec16, FixDec16);
convert_try_into!(FixDec128, fixdec32, FixDec32);
convert_try_into!(FixDec128, fixdec64, FixDec64);

// need by define_fixdec
const ALL_EXPS: [i128; 38 + 1] = [1,
    10_i128.pow(1), 10_i128.pow(2), 10_i128.pow(3), 10_i128.pow(4),
    10_i128.pow(5), 10_i128.pow(6), 10_i128.pow(7), 10_i128.pow(8),
    10_i128.pow(9), 10_i128.pow(10), 10_i128.pow(11), 10_i128.pow(12),
    10_i128.pow(13), 10_i128.pow(14), 10_i128.pow(15), 10_i128.pow(16),
    10_i128.pow(17), 10_i128.pow(18), 10_i128.pow(19), 10_i128.pow(20),
    10_i128.pow(21), 10_i128.pow(22), 10_i128.pow(23), 10_i128.pow(24),
    10_i128.pow(25), 10_i128.pow(26), 10_i128.pow(27), 10_i128.pow(28),
    10_i128.pow(29), 10_i128.pow(30), 10_i128.pow(31), 10_i128.pow(32),
    10_i128.pow(33), 10_i128.pow(34), 10_i128.pow(35), 10_i128.pow(36),
    10_i128.pow(37), 10_i128.pow(38),
];

const fn calc_mul_div(a: i128, b: i128, c: i128, rounding: Rounding) -> Option<i128> {
    if let Some(r) = a.checked_mul(b) {
        rounding_div(r, c, rounding)
    } else {
        None // todo!("mul overflow");
    }
}
