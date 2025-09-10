use crate::fpdec_inner::FpdecInner;
use int_div_cum_error::{CumErr, DivCumErr, Rounding};

macro_rules! calc_mul_div_higher {
    (
        $a:expr, $b:expr, $c:expr,
        $rounding:expr, $cum_error:expr,
        $origin_type:ty, $higher_type:ty
    ) => {{
        let dividend = $a as $higher_type * $b as $higher_type;
        let divisor = $c as $higher_type;
        match $cum_error {
            Some(_cum_error) => {
                todo!()
                /*
                let mut higher_cum_error = *cum_error as $higher_type;
                let q =
                    dividend.checked_div_with_cum_err(divisor, $rounding, &mut higher_cum_error)?;

                *cum_error = higher_cum_error as $origin_type;
                <$origin_type>::try_from(q).ok()
                */
            }
            None => {
                let q = dividend.checked_div_with_rounding(divisor, $rounding)?;
                <$origin_type>::try_from(q).ok()
            }
        }
    }};
}

impl FpdecInner for i8 {
    const MAX: Self = i8::MAX;
    const MIN: Self = i8::MIN;
    const TEN: Self = 10;
    const MAX_POWERS: Self = 10_i8.pow(Self::DIGITS);
    const DIGITS: u32 = i8::MAX.ilog10();

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i8; 3] = [1, 10_i8.pow(1), 10_i8.pow(2)];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(
        self,
        b: Self,
        c: Self,
        rounding: Rounding,
        cum_error: Option<&mut CumErr<Self>>,
    ) -> Option<Self> {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i8, i16)
    }
}

impl FpdecInner for i16 {
    const MAX: Self = i16::MAX;
    const MIN: Self = i16::MIN;
    const TEN: Self = 10;
    const MAX_POWERS: Self = 10_i16.pow(Self::DIGITS);
    const DIGITS: u32 = i16::MAX.ilog10();

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i16; 5] = [
            1,
            10_i16.pow(1),
            10_i16.pow(2),
            10_i16.pow(3),
            10_i16.pow(4),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(
        self,
        b: Self,
        c: Self,
        rounding: Rounding,
        cum_error: Option<&mut CumErr<Self>>,
    ) -> Option<Self> {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i16, i32)
    }
}

impl FpdecInner for i32 {
    const MAX: Self = i32::MAX;
    const MIN: Self = i32::MIN;
    const TEN: Self = 10;
    const MAX_POWERS: Self = 10_i32.pow(Self::DIGITS);
    const DIGITS: u32 = i32::MAX.ilog10();

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i32; 10] = [
            1,
            10_i32.pow(1),
            10_i32.pow(2),
            10_i32.pow(3),
            10_i32.pow(4),
            10_i32.pow(5),
            10_i32.pow(6),
            10_i32.pow(7),
            10_i32.pow(8),
            10_i32.pow(9),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(
        self,
        b: Self,
        c: Self,
        rounding: Rounding,
        cum_error: Option<&mut CumErr<Self>>,
    ) -> Option<Self> {
        calc_mul_div_higher!(self, b, c, rounding, cum_error, i32, i64)
    }
}

impl FpdecInner for i64 {
    const MAX: Self = i64::MAX;
    const MIN: Self = i64::MIN;
    const TEN: Self = 10;
    const MAX_POWERS: Self = 10_i64.pow(Self::DIGITS);
    const DIGITS: u32 = i64::MAX.ilog10();

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i64; 19] = [
            1,
            10_i64.pow(1),
            10_i64.pow(2),
            10_i64.pow(3),
            10_i64.pow(4),
            10_i64.pow(5),
            10_i64.pow(6),
            10_i64.pow(7),
            10_i64.pow(8),
            10_i64.pow(9),
            10_i64.pow(10),
            10_i64.pow(11),
            10_i64.pow(12),
            10_i64.pow(13),
            10_i64.pow(14),
            10_i64.pow(15),
            10_i64.pow(16),
            10_i64.pow(17),
            10_i64.pow(18),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(
        self,
        b: Self,
        c: Self,
        rounding: Rounding,
        cum_error: Option<&mut CumErr<Self>>,
    ) -> Option<Self> {
        // try to avoid casting to i128 which is slower
        match self.checked_mul(b) {
            Some(m) => match cum_error {
                Some(cum_error) => m.checked_div_with_cum_err(c, rounding, cum_error),
                None => m.checked_div_with_rounding(c, rounding),
            },
            None => calc_mul_div_higher!(self, b, c, rounding, cum_error, i64, i128),
        }
    }
}
