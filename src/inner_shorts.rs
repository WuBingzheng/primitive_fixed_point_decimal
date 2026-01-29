use crate::fpdec_inner::FpdecInner;
use crate::Rounding;

macro_rules! calc_mul_div_higher {
    (
        $a:expr, $b:expr, $c:expr, $rounding:expr,
        $origin_type:ty, $higher_type:ty
    ) => {{
        let dividend = $a as $higher_type * $b as $higher_type;
        let divisor = $c as $higher_type;
        let q = dividend.rounding_div(divisor, $rounding)?;
        <$origin_type>::try_from(q).ok()
    }};
}

macro_rules! signed_consts {
    ($typ:ty, $uns_typ:ty, $neg_min_str:expr) => {
        const MAX: Self = <$typ>::MAX;
        const MIN: Self = <$typ>::MIN;
        const TEN: Self = 10;
        const HUNDRED: Self = 100;
        const MAX_POWERS: Self = Self::TEN.pow(Self::DIGITS);
        const DIGITS: u32 = Self::MAX.ilog10();

        const NEG_MIN_STR: &'static str = $neg_min_str;

        type Unsigned = $uns_typ;
        fn unsigned_abs(self) -> Self::Unsigned {
            self.unsigned_abs()
        }
    };
}

macro_rules! unsigned_consts {
    ($typ:ty) => {
        const MAX: Self = <$typ>::MAX;
        const MIN: Self = <$typ>::MIN;
        const TEN: Self = 10;
        const HUNDRED: Self = 100;
        const MAX_POWERS: Self = Self::TEN.pow(Self::DIGITS);
        const DIGITS: u32 = Self::MAX.ilog10();

        #[doc(hidden)]
        const NEG_MIN_STR: &'static str = "unreachable";

        type Unsigned = $typ;
        fn unsigned_abs(self) -> Self::Unsigned {
            self
        }
    };
}

impl FpdecInner for i8 {
    signed_consts!(i8, u8, "128");

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i8; 3] = [1, 10_i8.pow(1), 10_i8.pow(2)];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        calc_mul_div_higher!(self, b, c, rounding, i8, i16)
    }
}

impl FpdecInner for i16 {
    signed_consts!(i16, u16, "32768");

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

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        calc_mul_div_higher!(self, b, c, rounding, i16, i32)
    }
}

impl FpdecInner for i32 {
    signed_consts!(i32, u32, "2147483648");

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

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        calc_mul_div_higher!(self, b, c, rounding, i32, i64)
    }
}

impl FpdecInner for i64 {
    signed_consts!(i64, u64, "9223372036854775808");

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

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        // try to avoid casting to i128 which is slower
        match self.checked_mul(b) {
            Some(m) => m.rounding_div(c, rounding),
            None => calc_mul_div_higher!(self, b, c, rounding, i64, i128),
        }
    }
}
impl FpdecInner for u8 {
    unsigned_consts!(u8);

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [u8; 3] = [1, 10_u8.pow(1), 10_u8.pow(2)];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        calc_mul_div_higher!(self, b, c, rounding, u8, u16)
    }
}

impl FpdecInner for u16 {
    unsigned_consts!(u16);

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [u16; 5] = [
            1,
            10_u16.pow(1),
            10_u16.pow(2),
            10_u16.pow(3),
            10_u16.pow(4),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        calc_mul_div_higher!(self, b, c, rounding, u16, u32)
    }
}

impl FpdecInner for u32 {
    unsigned_consts!(u32);

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [u32; 10] = [
            1,
            10_u32.pow(1),
            10_u32.pow(2),
            10_u32.pow(3),
            10_u32.pow(4),
            10_u32.pow(5),
            10_u32.pow(6),
            10_u32.pow(7),
            10_u32.pow(8),
            10_u32.pow(9),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        calc_mul_div_higher!(self, b, c, rounding, u32, u64)
    }
}

impl FpdecInner for u64 {
    unsigned_consts!(u64);

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [u64; 20] = [
            1,
            10_u64.pow(1),
            10_u64.pow(2),
            10_u64.pow(3),
            10_u64.pow(4),
            10_u64.pow(5),
            10_u64.pow(6),
            10_u64.pow(7),
            10_u64.pow(8),
            10_u64.pow(9),
            10_u64.pow(10),
            10_u64.pow(11),
            10_u64.pow(12),
            10_u64.pow(13),
            10_u64.pow(14),
            10_u64.pow(15),
            10_u64.pow(16),
            10_u64.pow(17),
            10_u64.pow(18),
            10_u64.pow(19),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        // try to avoid casting to u128 which is slower
        match self.checked_mul(b) {
            Some(m) => m.rounding_div(c, rounding),
            None => calc_mul_div_higher!(self, b, c, rounding, u64, u128),
        }
    }
}
