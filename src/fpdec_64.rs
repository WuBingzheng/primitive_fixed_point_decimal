crate::define_both_fpdecs::define_both_fpdecs!(StaticPrecFpdec64, OobPrecFpdec64, i64, 18, 64, 63);

crate::define_convert::define_convert_into_longer!(
    StaticPrecFpdec64,
    OobPrecFpdec64,

    (fpdec_128, StaticPrecFpdec128, OobPrecFpdec128),
);

crate::define_convert::define_convert_try_into_shorter!(
    StaticPrecFpdec64,
    OobPrecFpdec64,

    (fpdec_16, StaticPrecFpdec16, OobPrecFpdec16),
    (fpdec_32, StaticPrecFpdec32, OobPrecFpdec32),
);

// internal stuff needed by define_macro
const ALL_EXPS: [i64; 19] = [1,
    10_i64.pow(1), 10_i64.pow(2), 10_i64.pow(3), 10_i64.pow(4),
    10_i64.pow(5), 10_i64.pow(6), 10_i64.pow(7), 10_i64.pow(8),
    10_i64.pow(9), 10_i64.pow(10), 10_i64.pow(11), 10_i64.pow(12),
    10_i64.pow(13), 10_i64.pow(14), 10_i64.pow(15), 10_i64.pow(16),
    10_i64.pow(17), 10_i64.pow(18),
];

const fn calc_mul_div(a: i64, b: i64, c: i64, rounding: Rounding) -> Option<i64> {
    // try i64 first because I guess it's faster than i128
    if let Some(r) = a.checked_mul(b) {
        rounding_div!(r, c, rounding)
    } else {
        convert_lower!(rounding_div!(a as i128 * b as i128, c as i128, rounding), i64)
    }
}
