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
        let exp = Self::get_exp(i)?;

        if self ^ b >= 0 {
            let extra = match rounding {
                Rounding::Floor | Rounding::TowardsZero => 0,
                Rounding::Ceiling | Rounding::AwayFromZero => exp - 1,
                Rounding::Round => exp / 2, // exp is even
            };

            if self.leading_zeros() + b.leading_zeros() >= 64 + 2 {
                // happy path, (self * b + extra) is not overflow
                Some((self * b + extra) / exp)
            } else {
                let n = self as i128 * b as i128;
                let q = div_exp_fast(n as u128 + extra as u128, exp as u64, i)?;
                i64::try_from(q).ok()
            }
        } else {
            let extra = match rounding {
                Rounding::Ceiling | Rounding::TowardsZero => 0,
                Rounding::Floor | Rounding::AwayFromZero => exp - 1,
                Rounding::Round => exp / 2, // exp is even
            };

            let n = self as i128 * b as i128;
            let q = div_exp_fast(-n as u128 + extra as u128, exp as u64, i)?;
            i64::try_from(-(q as i128)).ok()
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
        let exp = Self::get_exp(i)?;

        let extra = match rounding {
            Rounding::Floor | Rounding::TowardsZero => 0,
            Rounding::Ceiling | Rounding::AwayFromZero => exp - 1,
            Rounding::Round => exp / 2, // exp is even
        };

        if self.leading_zeros() + b.leading_zeros() >= 64 + 1 {
            // happy path, (self * b + extra) is not overflow
            Some((self * b + extra) / exp)
        } else {
            let n = self as u128 * b as u128;
            div_exp_fast(n + extra as u128, exp, i)
        }
    }
}

// calculate fast: n / exp, with i = log10(exp)
//
// Use the algorithm in Moller and Granlund's paper: Improved division by
// invariant integers: https://gmplib.org/~tege/division-paper.pdf
fn div_exp_fast(n: u128, exp: u64, i: usize) -> Option<u64> {
    // The magics are generated by python code:
    //
    // def gen(d):
    //     zeros = 64 - d.bit_length()
    //     magic = pow(2, 128) // (d << zeros)
    //     magic = magic - pow(2, 64) # make magic fit in 128-bit
    //     return (magic, zeros)
    const MG_EXP_MAGICS: [(u64, u32); 20] = [
        (0, 0),
        (0x9999999999999999, 60),
        (0x47ae147ae147ae14, 57),
        (0x0624dd2f1a9fbe76, 54),
        (0xa36e2eb1c432ca57, 50),
        (0x4f8b588e368f0846, 47),
        (0x0c6f7a0b5ed8d36b, 44),
        (0xad7f29abcaf48578, 40),
        (0x5798ee2308c39df9, 37),
        (0x12e0be826d694b2e, 34),
        (0xb7cdfd9d7bdbab7d, 30),
        (0x5fd7fe17964955fd, 27),
        (0x19799812dea11197, 24),
        (0xc25c268497681c26, 20),
        (0x6849b86a12b9b01e, 17),
        (0x203af9ee756159b2, 14),
        (0xcd2b297d889bc2b6, 10),
        (0x70ef54646d496892, 7),
        (0x2725dd1d243aba0e, 4),
        (0xd83c94fb6d2ac34a, 0),
    ];

    // check overflow
    if (n >> 64) as u64 >= exp {
        return None;
    }

    // algorithm:
    //   zn = n << zeros
    //   q = (((magic * zn) >> 64) + zn) >> 64

    // SAFETY: exp has been read by i already
    let &(magic, zeros) = unsafe { MG_EXP_MAGICS.get_unchecked(i) };

    // calc: (high, low) := n << zeros
    let zn = n << zeros;
    let high = (zn >> 64) as u64;
    let low = zn as u64;

    // calc: mul := (magic * zn) >> 64
    let mul_low = (low as u128 * magic as u128) >> 64;
    let mul = (high as u128 * magic as u128) + mul_low;

    // calc: final q
    let q = (mul + zn) >> 64;

    // correction by remainder
    let exp = exp as u128;
    if n - q * exp < exp {
        Some(q as u64)
    } else {
        Some(q as u64 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_mul_div() {
        for iexp in 1..20 {
            let exp = 10_u64.pow(iexp);

            for i in 0..iexp {
                let a = exp - i as u64;

                // enlarge this range for more test
                for j in 0..1000 {
                    let b = u64::MAX - j * 13;

                    let q1 = a.calc_mul_div(b, exp, Rounding::Floor);
                    let q2 = a.calc_mul_div_exp(b, iexp as usize, Rounding::Floor);
                    assert_eq!(q1, q2);
                    let q1 = a.calc_mul_div(b, exp, Rounding::Ceiling);
                    let q2 = a.calc_mul_div_exp(b, iexp as usize, Rounding::Ceiling);
                    assert_eq!(q1, q2);
                    let q1 = a.calc_mul_div(b, exp, Rounding::Round);
                    let q2 = a.calc_mul_div_exp(b, iexp as usize, Rounding::Round);
                    assert_eq!(q1, q2);
                }
            }
        }
    }
}
