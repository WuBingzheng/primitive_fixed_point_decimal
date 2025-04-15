use crate::fpdec_inner::FpdecInner;
use int_div_cum_error::{
    Rounding,
    checked_divide_with_rounding,
    checked_divide_with_cum_error,
};

macro_rules! calc_mul_div_higher {
    (
        $a:expr, $b:expr, $c:expr,
        $rounding:expr, $cum_error:expr,
        $origin_type:ty, $higher_type:ty
    ) => {
        {
            match $cum_error {
                Some(cum_error) => {
                    let mut higher_cum_error = *cum_error as $higher_type;
                    let q = checked_divide_with_cum_error(
                        $a as $higher_type * $b as $higher_type,
                        $c as $higher_type,
                        $rounding,
                        &mut higher_cum_error)?;

                    *cum_error = higher_cum_error as $origin_type;
                    <$origin_type>::try_from(q).ok()
                }
                None => {
                    let q = checked_divide_with_rounding(
                        $a as $higher_type * $b as $higher_type,
                        $c as $higher_type,
                        $rounding)?;
                    <$origin_type>::try_from(q).ok()
                }
            }
        }
    }
}

impl FpdecInner for i8 {
    const MAX: Self = i8::MAX;
    const MIN: Self = i8::MIN;
    const MAX_POWERS: Self = 10_i8.pow(2);

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i8; 3] = [
            1, 10_i8.pow(1), 10_i8.pow(2)
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding, cum_error: Option<&mut Self>)
        -> Option<Self>
    {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i8, i16)
    }
}

impl FpdecInner for i16 {
    const MAX: Self = i16::MAX;
    const MIN: Self = i16::MIN;
    const MAX_POWERS: Self = 10_i16.pow(4);

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i16; 5] = [
            1,
            10_i16.pow(1), 10_i16.pow(2), 10_i16.pow(3), 10_i16.pow(4),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding, cum_error: Option<&mut Self>)
        -> Option<Self>
    {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i16, i32)
    }
}

impl FpdecInner for i32 {
    const MAX: Self = i32::MAX;
    const MIN: Self = i32::MIN;
    const MAX_POWERS: Self = 10_i32.pow(9);

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i32; 10] = [
            1,
            10_i32.pow(1), 10_i32.pow(2), 10_i32.pow(3), 10_i32.pow(4),
            10_i32.pow(5), 10_i32.pow(6), 10_i32.pow(7), 10_i32.pow(8),
            10_i32.pow(9)
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding, cum_error: Option<&mut Self>)
        -> Option<Self>
    {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i32, i64)
    }
}

impl FpdecInner for i64 {
    const MAX: Self = i64::MAX;
    const MIN: Self = i64::MIN;
    const MAX_POWERS: Self = 10_i64.pow(18);

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i64; 19] = [
            1,
            10_i64.pow(1), 10_i64.pow(2), 10_i64.pow(3), 10_i64.pow(4),
            10_i64.pow(5), 10_i64.pow(6), 10_i64.pow(7), 10_i64.pow(8),
            10_i64.pow(9), 10_i64.pow(10), 10_i64.pow(11), 10_i64.pow(12),
            10_i64.pow(13), 10_i64.pow(14), 10_i64.pow(15), 10_i64.pow(16),
            10_i64.pow(17), 10_i64.pow(18),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding, cum_error: Option<&mut Self>)
    -> Option<Self>
    {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i64, i128)
    }
}
