crate::define_both_fpdecs::define_both_fpdecs!(StaticPrecFpdec16, OobPrecFpdec16, i16, 4, 16, 15);

crate::define_convert::define_convert_into_longer!(
    StaticPrecFpdec16,
    OobPrecFpdec16,

    (fpdec_32, StaticPrecFpdec32, OobPrecFpdec32),
    (fpdec_64, StaticPrecFpdec64, OobPrecFpdec64),
    (fpdec_128, StaticPrecFpdec128, OobPrecFpdec128),
);

// internal stuff needed by define macros
const ALL_EXPS: [i16; 1 + 4] = [1,
    10_i16.pow(1), 10_i16.pow(2), 10_i16.pow(3), 10_i16.pow(4),
];

const fn calc_mul_div(a: i16, b: i16, c: i16, rounding: Rounding, cum_error: &mut i16) -> Option<i16> {
    calc_mul_div_higher!(a, b, c, rounding, cum_error, i16, i32)
}
