use crate::fpdec_inner::FpdecInner;
use crate::Rounding;

macro_rules! common_consts {
    ($wider_typ:ty) => {
        const MAX: Self = Self::MAX;
        const MIN: Self = Self::MIN;
        const TEN: Self = 10;
        const HUNDRED: Self = 100;
        const MAX_POWERS: Self = Self::TEN.pow(Self::DIGITS);
        const DIGITS: u32 = Self::MAX.ilog10();

        type Wider = $wider_typ;
        fn as_wider(self) -> Self::Wider {
            self as $wider_typ
        }
        fn from_wider(w: Self::Wider) -> Option<Self> {
            Self::try_from(w).ok()
        }
    };
}

macro_rules! signed_consts {
    ($wider_typ:ty, $uns_typ:ty, $neg_min_str:expr) => {
        common_consts!($wider_typ);

        const NEG_MIN_STR: &'static str = $neg_min_str;

        type Unsigned = $uns_typ;
        fn unsigned_abs(self) -> Self::Unsigned {
            self.unsigned_abs()
        }
    };
}

macro_rules! unsigned_consts {
    ($wider_typ:ty) => {
        common_consts!($wider_typ);

        #[doc(hidden)]
        const NEG_MIN_STR: &'static str = "unreachable";

        type Unsigned = Self;
        fn unsigned_abs(self) -> Self::Unsigned {
            self
        }
    };
}

impl FpdecInner for i8 {
    signed_consts!(i16, u8, "128");

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i8; 3] = [1, 10_i8.pow(1), 10_i8.pow(2)];

        ALL_EXPS.get(i).copied()
    }
}

impl FpdecInner for i16 {
    signed_consts!(i32, u16, "32768");

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
}

impl FpdecInner for i32 {
    signed_consts!(i64, u32, "2147483648");

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
}

impl FpdecInner for i64 {
    signed_consts!(i128, u64, "9223372036854775808");

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

    fn calc_mul_div_exp(self, b: Self, i: usize, rounding: Rounding) -> Option<Self> {
        match self.checked_mul(b) {
            Some(m) => m.rounding_div(Self::get_exp(i)?, rounding),
            None => {
                let p = self as i128 * b as i128;
                let exp = Self::get_exp(i)? as i128;
                debug_assert!(p.checked_add(exp).is_some());

                let extra = if self >= 0 {
                    match rounding {
                        Rounding::Floor | Rounding::TowardsZero => 0,
                        Rounding::Ceiling | Rounding::AwayFromZero => exp - 1,
                        Rounding::Round => exp / 2, // exp is even
                    }
                } else {
                    match rounding {
                        Rounding::Ceiling | Rounding::TowardsZero => 0,
                        Rounding::Floor | Rounding::AwayFromZero => 1 - exp,
                        Rounding::Round => -exp / 2, // exp is even
                    }
                };

                let q = (p + extra).div_exp(exp, i);
                i64::try_from(q).ok()
            }
        }
    }
}

impl FpdecInner for u8 {
    unsigned_consts!(u16);

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [u8; 3] = [1, 10_u8.pow(1), 10_u8.pow(2)];

        ALL_EXPS.get(i).copied()
    }
}

impl FpdecInner for u16 {
    unsigned_consts!(u32);

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
}

impl FpdecInner for u32 {
    unsigned_consts!(u64);

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
}

impl FpdecInner for u64 {
    unsigned_consts!(u128);

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

    fn calc_mul_div_exp(self, b: Self, i: usize, rounding: Rounding) -> Option<Self> {
        match self.checked_mul(b) {
            Some(m) => m.rounding_div(Self::get_exp(i)?, rounding),
            None => {
                let p = self as u128 * b as u128;
                let exp = Self::get_exp(i)? as u128;
                debug_assert!(p.checked_add(exp).is_some());

                let extra = match rounding {
                    Rounding::Floor | Rounding::TowardsZero => 0,
                    Rounding::Ceiling | Rounding::AwayFromZero => exp - 1,
                    Rounding::Round => exp / 2, // exp is even
                };

                let q = (p + extra).div_exp(exp, i);
                u64::try_from(q).ok()
            }
        }
    }
}
