crate::define_both_fpdecs::define_both_fpdecs!(StaticPrecFpdec32, OobPrecFpdec32, i32, 9, 32, 31);

crate::define_convert::define_convert_into_longer!(
    StaticPrecFpdec32,
    OobPrecFpdec32,

    (fpdec_64, StaticPrecFpdec64, OobPrecFpdec64),
    (fpdec_128, StaticPrecFpdec128, OobPrecFpdec128),
);

crate::define_convert::define_convert_try_into_shorter!(
    StaticPrecFpdec32,
    OobPrecFpdec32,

    (fpdec_16, StaticPrecFpdec16, OobPrecFpdec16),
);

// internal stuff needed by define macros
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
