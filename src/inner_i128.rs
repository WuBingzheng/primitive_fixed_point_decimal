use crate::fpdec_inner::FpdecInner;
use crate::Rounding;

impl FpdecInner for i128 {
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const TEN: Self = 10;
    const HUNDRED: Self = 100;
    const MAX_POWERS: Self = 10_i128.pow(Self::DIGITS);
    const DIGITS: u32 = Self::MAX.ilog10();
    const NEG_MIN_STR: &'static str = "170141183460469231731687303715884105728";

    type Unsigned = u128;
    fn unsigned_abs(self) -> Self::Unsigned {
        self.unsigned_abs()
    }

    // Since we have our own calc_mul_div(), so we do not need these.
    type Wider = i128;
    fn as_wider(self) -> Self::Wider {
        unreachable!()
    }
    fn from_wider(_: Self::Wider) -> Option<Self> {
        unreachable!()
    }

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [i128; 39] = [
            1,
            10_i128.pow(1),
            10_i128.pow(2),
            10_i128.pow(3),
            10_i128.pow(4),
            10_i128.pow(5),
            10_i128.pow(6),
            10_i128.pow(7),
            10_i128.pow(8),
            10_i128.pow(9),
            10_i128.pow(10),
            10_i128.pow(11),
            10_i128.pow(12),
            10_i128.pow(13),
            10_i128.pow(14),
            10_i128.pow(15),
            10_i128.pow(16),
            10_i128.pow(17),
            10_i128.pow(18),
            10_i128.pow(19),
            10_i128.pow(20),
            10_i128.pow(21),
            10_i128.pow(22),
            10_i128.pow(23),
            10_i128.pow(24),
            10_i128.pow(25),
            10_i128.pow(26),
            10_i128.pow(27),
            10_i128.pow(28),
            10_i128.pow(29),
            10_i128.pow(30),
            10_i128.pow(31),
            10_i128.pow(32),
            10_i128.pow(33),
            10_i128.pow(34),
            10_i128.pow(35),
            10_i128.pow(36),
            10_i128.pow(37),
            10_i128.pow(38),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        // happy path, no overflow
        if let Some(r) = self.checked_mul(b) {
            return r.rounding_div(c, rounding);
        }

        // normal path

        // (mhigh, mlow) = a * b
        let (mhigh, mlow) = mul2(self.unsigned_abs(), b.unsigned_abs());

        // (last_dividend, q) = (mhigh, mlow) / abs(c)
        let unsigned_c = c.unsigned_abs();
        let (last_dividend, mut q) = reduce2(mhigh, mlow, unsigned_c, 2)?;

        // back to signed i128: last_dividend
        let mut last_dividend = match i128::try_from(last_dividend) {
            Ok(dividend) => dividend,
            Err(_) => {
                // one more division
                q = q.checked_add(last_dividend / unsigned_c)?;
                (last_dividend % unsigned_c) as i128
            }
        };
        if (self ^ b) < 0 {
            last_dividend = -last_dividend;
        }

        // back to signed i128: quotient
        let mut q = i128::try_from(q).ok()?;
        if (self ^ b ^ c) < 0 {
            q = -q;
        }

        // final division
        let last_q = last_dividend.rounding_div(c, rounding)?;

        q.checked_add(last_q)
    }

    fn calc_mul_div_exp(self, b: Self, i: usize, rounding: Rounding) -> Option<Self> {
        let exp = Self::get_exp(i)?;

        if self ^ b >= 0 {
            let extra = match rounding {
                Rounding::Floor | Rounding::TowardsZero => 0,
                Rounding::Ceiling | Rounding::AwayFromZero => exp - 1,
                Rounding::Round => exp / 2, // exp is even
            };

            if self.leading_zeros() + b.leading_zeros() >= 128 + 2 {
                // happy path, (self * b + extra) is not overflow
                Some(div_exp_fast_1word((self * b + extra) as u128, i) as i128)
            } else {
                let ua = self.unsigned_abs();
                let ub = b.unsigned_abs();
                let q = div_exp_fast_2word(ua, ub, extra as u128, exp as u128, i)?;
                i128::try_from(q).ok()
            }
        } else {
            let exp = exp as u128;
            let extra = match rounding {
                Rounding::Ceiling | Rounding::TowardsZero => 0,
                Rounding::Floor | Rounding::AwayFromZero => exp - 1,
                Rounding::Round => exp / 2, // exp is even
            };

            let ua = self.unsigned_abs();
            let ub = b.unsigned_abs();
            let q = div_exp_fast_2word(ua, ub, extra, exp, i)?;

            if q <= i128::MAX as u128 {
                Some(-(q as i128))
            } else if q == i128::MAX as u128 + 1 {
                Some(i128::MIN)
            } else {
                None
            }
        }
    }

    fn rounding_div(self, b: Self, rounding: Rounding) -> Option<Self> {
        // try 64-bit first, which is much faster
        if let Ok(a64) = i64::try_from(self) {
            if let Ok(b64) = i64::try_from(b) {
                return a64.rounding_div(b64, rounding).map(|x| x as i128);
            }
        }
        self.do_rounding_div(b, rounding)
    }
}

impl FpdecInner for u128 {
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
    const TEN: Self = 10;
    const HUNDRED: Self = 100;
    const MAX_POWERS: Self = 10_u128.pow(Self::DIGITS);
    const DIGITS: u32 = Self::MAX.ilog10();

    #[doc(hidden)]
    const NEG_MIN_STR: &'static str = "unreachable";

    type Unsigned = Self;
    fn unsigned_abs(self) -> Self::Unsigned {
        self
    }

    // Since we have our own calc_mul_div(), so we do not need these.
    type Wider = i128;
    fn as_wider(self) -> Self::Wider {
        unreachable!()
    }
    fn from_wider(_: Self::Wider) -> Option<Self> {
        unreachable!()
    }

    fn get_exp(i: usize) -> Option<Self> {
        const ALL_EXPS: [u128; 39] = [
            1,
            10_u128.pow(1),
            10_u128.pow(2),
            10_u128.pow(3),
            10_u128.pow(4),
            10_u128.pow(5),
            10_u128.pow(6),
            10_u128.pow(7),
            10_u128.pow(8),
            10_u128.pow(9),
            10_u128.pow(10),
            10_u128.pow(11),
            10_u128.pow(12),
            10_u128.pow(13),
            10_u128.pow(14),
            10_u128.pow(15),
            10_u128.pow(16),
            10_u128.pow(17),
            10_u128.pow(18),
            10_u128.pow(19),
            10_u128.pow(20),
            10_u128.pow(21),
            10_u128.pow(22),
            10_u128.pow(23),
            10_u128.pow(24),
            10_u128.pow(25),
            10_u128.pow(26),
            10_u128.pow(27),
            10_u128.pow(28),
            10_u128.pow(29),
            10_u128.pow(30),
            10_u128.pow(31),
            10_u128.pow(32),
            10_u128.pow(33),
            10_u128.pow(34),
            10_u128.pow(35),
            10_u128.pow(36),
            10_u128.pow(37),
            10_u128.pow(38),
        ];

        ALL_EXPS.get(i).copied()
    }

    fn calc_mul_div(self, b: Self, c: Self, rounding: Rounding) -> Option<Self> {
        // happy path, no overflow
        if let Some(r) = self.checked_mul(b) {
            return r.rounding_div(c, rounding);
        }

        // normal path

        // (mhigh, mlow) = a * b
        let (mhigh, mlow) = mul2(self, b);

        // (last_dividend, q) = (mhigh, mlow) / c
        let (last_dividend, q) = if c < (1_u128 << 127) {
            reduce2(mhigh, mlow, c, 1)?
        } else {
            reduce2_big(mhigh, mlow, c)?
        };

        // final division
        let last_q = last_dividend.rounding_div(c, rounding)?;

        q.checked_add(last_q)
    }

    fn calc_mul_div_exp(self, b: Self, i: usize, rounding: Rounding) -> Option<Self> {
        let exp = Self::get_exp(i)?;

        let extra = match rounding {
            Rounding::Floor | Rounding::TowardsZero => 0,
            Rounding::Ceiling | Rounding::AwayFromZero => exp - 1,
            Rounding::Round => exp / 2, // exp is even
        };

        if self.leading_zeros() + b.leading_zeros() >= 128 + 1 {
            // happy path, (self * b + extra) is not overflow
            Some(div_exp_fast_1word(self * b + extra, i))
        } else {
            div_exp_fast_2word(self, b, extra as u128, exp, i)
        }
    }

    fn rounding_div(self, b: Self, rounding: Rounding) -> Option<Self> {
        // try 64-bit first, which is much faster
        if let Ok(a64) = u64::try_from(self) {
            if let Ok(b64) = u64::try_from(b) {
                return a64.rounding_div(b64, rounding).map(|x| x as u128);
            }
        }
        self.do_rounding_div(b, rounding)
    }
}

// calculate: a * b = (mhigh,mlow)
const fn mul2(a: u128, b: u128) -> (u128, u128) {
    let (ahigh, alow) = (a >> 64, a & u64::MAX as u128);
    let (bhigh, blow) = (b >> 64, b & u64::MAX as u128);

    let (mid, carry1) = (alow * bhigh).overflowing_add(ahigh * blow);
    let (mlow, carry2) = (alow * blow).overflowing_add(mid << 64);
    let mhigh = ahigh * bhigh + (mid >> 64) + ((carry1 as u128) << 64) + carry2 as u128;
    (mhigh, mlow)
}

// reduce: (mhigh, mlow) into (last_dividend, q) where last_dividend fits in 128-bit.
//
// calculate: (mhigh,mlow) / divisor = q .. last_dividend
// where last_dividend not {<divisor} while {fits in 128-bit}.
// So the caller should do the final division.
fn reduce2(mhigh: u128, mlow: u128, divisor: u128, rate: u128) -> Option<(u128, u128)> {
    // check overflow or c==0
    if mhigh * rate >= divisor {
        return None;
    }

    // no need to reduce
    if mhigh == 0 {
        return Some((mlow, 0));
    }

    // consume @mhigh and reduce the dividend to @mlow
    let mut dividend = mhigh;
    let mut total_shft = 0;
    let mut q = 0;
    loop {
        let zeros = dividend.leading_zeros();
        debug_assert_ne!(zeros, 0); // because divisor < u128::MAX/2
        if zeros + total_shft >= 128 {
            break;
        }
        dividend = dividend << zeros | mlow << total_shft >> (128 - zeros);
        q = q << zeros | dividend / divisor;

        dividend %= divisor;
        total_shft += zeros;
    }

    q <<= 128 - total_shft;
    dividend = dividend << (128 - total_shft) | mlow << total_shft >> total_shft;

    Some((dividend, q))
}

// Works for divisor >= u128::MAX/2
// We can not use division in loop, but use substraction.
fn reduce2_big(mhigh: u128, mlow: u128, divisor: u128) -> Option<(u128, u128)> {
    // check overflow or c==0
    if mhigh >= divisor {
        return None;
    }

    // no need to reduce
    if mhigh == 0 {
        return Some((mlow, 0));
    }

    // consume @mhigh and reduce the dividend to @mlow
    let mut dividend = mhigh;
    let mut total_shft = 0;
    let mut q: u128 = 0;
    loop {
        let mut zeros = dividend.leading_zeros();
        if zeros + total_shft >= 128 {
            break;
        }

        // here 'zeros' may be 0
        if zeros != 0 {
            dividend = dividend << zeros | mlow << total_shft >> (128 - zeros);
        }

        // shift 1 more bit to make sure zeros>0, if need
        if dividend < divisor {
            dividend = dividend << 1 | mlow << (total_shft + zeros) >> 127;
            zeros += 1;
        }

        // 'zeros' may be 128 because of the 1 more bit above,
        // this may happen only at the first loop where 'q'=0.
        q = q.unbounded_shl(zeros) + 1;

        // using 'wrapping_sub' because of the 1 more bit above.
        dividend = dividend.wrapping_sub(divisor);

        total_shft += zeros;
    }

    if total_shft < 128 {
        q <<= 128 - total_shft;
        dividend = dividend << (128 - total_shft) | mlow << total_shft >> total_shft;
    }

    Some((dividend, q))
}

// calculate fast: (n1 * n2 + extra) / exp, with i = log10(exp)
//
// divident is 2-word, 256-bit, double of divisor(exp).
//
// Use the algorithm in Moller and Granlund's paper: Improved division by
// invariant integers: https://gmplib.org/~tege/division-paper.pdf
fn div_exp_fast_2word(n1: u128, n2: u128, extra: u128, exp: u128, i: usize) -> Option<u128> {
    // The magics are generated by python code:
    //
    // def gen(d):
    //     zeros = 128 - d.bit_length()
    //     magic = pow(2, 256) // (d << zeros)
    //     magic = magic - pow(2, 128) # make magic fit in 128-bit
    //     return (magic, zeros)
    const MG_EXP_MAGICS: [(u128, u32); 39] = [
        (0, 0),
        (0x99999999999999999999999999999999, 124),
        (0x47ae147ae147ae147ae147ae147ae147, 121),
        (0x0624dd2f1a9fbe76c8b4395810624dd2, 118),
        (0xa36e2eb1c432ca57a786c226809d4951, 114),
        (0x4f8b588e368f08461f9f01b866e43aa7, 111),
        (0x0c6f7a0b5ed8d36b4c7f34938583621f, 108),
        (0xad7f29abcaf485787a6520ec08d23699, 104),
        (0x5798ee2308c39df9fb841a566d74f87a, 101),
        (0x12e0be826d694b2e62d01511f12a6061, 98),
        (0xb7cdfd9d7bdbab7d6ae6881cb5109a36, 94),
        (0x5fd7fe17964955fdef1ed34a2a73ae91, 91),
        (0x19799812dea11197f27f0f6e885c8ba7, 88),
        (0xc25c268497681c2650cb4be40d60df73, 84),
        (0x6849b86a12b9b01ea70909833de71928, 81),
        (0x203af9ee756159b21f3a6e0297ec1420, 78),
        (0xcd2b297d889bc2b6985d7cd0f3135367, 74),
        (0x70ef54646d496892137dfd73f5a90f85, 71),
        (0x2725dd1d243aba0e75fe645cc4873f9e, 68),
        (0xd83c94fb6d2ac34a5663d3c7a0d865ca, 64),
        (0x79ca10c9242235d511e976394d79eb08, 61),
        (0x2e3b40a0e9b4f7dda7edf82dd794bc06, 58),
        (0xe392010175ee5962a6498d1625bac670, 54),
        (0x82db34012b25144eeb6e0a781e2f0527, 51),
        (0x357c299a88ea76a58924d52ce4f26a85, 48),
        (0xef2d0f5da7dd8aa27507bb7b07ea4409, 44),
        (0x8c240c4aecb13bb52a6c95fc0655033a, 41),
        (0x3ce9a36f23c0fc90eebd44c99eaa68fb, 38),
        (0xfb0f6be50601941b17953adc3110a7f8, 34),
        (0x95a5efea6b34767c12ddc8b027408660, 31),
        (0x4484bfeebc29f863424b06f3529a051a, 28),
        (0x039d66589687f9e901d59f290ee19dae, 25),
        (0x9f623d5a8a732974cfbc31db4b0295e4, 21),
        (0x4c4e977ba1f5bac3d9635b15d59bab1c, 18),
        (0x09d8792fb4c495697ab5e277de16227d, 15),
        (0xa95a5b7f87a0ef0f2abc9d8c9689d0c8, 11),
        (0x54484932d2e725a5bbca17a3aba173d3, 8),
        (0x1039d428a8b8eaeafca1ac82efb45ca9, 5),
        (0xb38fb9daa78e44ab2dcf7a6b19209442, 1),
    ];

    // calc: (n_high, n_low) := n1 * n2 + extra
    let (n_high, n_low) = mul2(n1, n2);
    let (n_low, carry) = n_low.overflowing_add(extra);
    let n_high = n_high + carry as u128;

    // check overflow
    if n_high >= exp {
        return None;
    }

    // algorithm:
    //   zn = n << zeros
    //   q = (((magic * zn) >> 128) + zn) >> 128

    // SAFETY: exp has been read by i already in the caller
    let &(magic, zeros) = unsafe { MG_EXP_MAGICS.get_unchecked(i) };

    // calc: (z_high, z_low) := n << zeros
    let z_high = (n_high << zeros) | (n_low >> (128 - zeros));
    let z_low = n_low << zeros;

    // calc: (m_high, m_low) := (magic * zn) >> 128
    let (m1_high, _) = mul2(z_low, magic);
    let (m2_high, m2_low) = mul2(z_high, magic);

    let (m_low, carry) = m2_low.overflowing_add(m1_high);
    let m_high = m2_high + carry as u128;

    // calc: final q
    let (_, carry) = m_low.overflowing_add(z_low);
    let q = m_high + z_high + carry as u128;

    // correction by remainder
    // check: n - q * exp < exp
    let (pp_high, pp_low) = mul2(q, exp);
    let (r_low, borrow) = n_low.overflowing_sub(pp_low);
    debug_assert_eq!(n_high, pp_high + borrow as u128); // 10.pow(38)*2 < MAX

    if r_low < exp {
        Some(q)
    } else {
        Some(q + 1)
    }
}

// calculate fast: n / 10.pow(i)
//
// n is 1-word, 128-bit, same with divisor(exp).
//
// Use the algorithm in Granlund & Montgomery's paper:
//   https://gmplib.org/%7Etege/divcnst-pldi94.pdf
// A good tutorial for the algorithm:
//   https://homepage.divms.uiowa.edu/%7Ejones/bcd/divide.html
fn div_exp_fast_1word(n: u128, i: usize) -> u128 {
    // The magics are generated by python code:
    //
    // def gen(d):
    //    l = math.ceil( math.log2(d) )
    //    m = pow(2, 128+l) // d + 1
    //    m = m - pow(2, 128) # make m fit in 128-bit
    //    return (m, l - 1)
    const GM_EXP_MAGICS: [(u128, u32); 39] = [
        (0, 0),
        (0x9999999999999999999999999999999a, 3),
        (0x47ae147ae147ae147ae147ae147ae148, 6),
        (0x0624dd2f1a9fbe76c8b4395810624dd3, 9),
        (0xa36e2eb1c432ca57a786c226809d4952, 13),
        (0x4f8b588e368f08461f9f01b866e43aa8, 16),
        (0x0c6f7a0b5ed8d36b4c7f349385836220, 19),
        (0xad7f29abcaf485787a6520ec08d2369a, 23),
        (0x5798ee2308c39df9fb841a566d74f87b, 26),
        (0x12e0be826d694b2e62d01511f12a6062, 29),
        (0xb7cdfd9d7bdbab7d6ae6881cb5109a37, 33),
        (0x5fd7fe17964955fdef1ed34a2a73ae92, 36),
        (0x19799812dea11197f27f0f6e885c8ba8, 39),
        (0xc25c268497681c2650cb4be40d60df74, 43),
        (0x6849b86a12b9b01ea70909833de71929, 46),
        (0x203af9ee756159b21f3a6e0297ec1421, 49),
        (0xcd2b297d889bc2b6985d7cd0f3135368, 53),
        (0x70ef54646d496892137dfd73f5a90f86, 56),
        (0x2725dd1d243aba0e75fe645cc4873f9f, 59),
        (0xd83c94fb6d2ac34a5663d3c7a0d865cb, 63),
        (0x79ca10c9242235d511e976394d79eb09, 66),
        (0x2e3b40a0e9b4f7dda7edf82dd794bc07, 69),
        (0xe392010175ee5962a6498d1625bac671, 73),
        (0x82db34012b25144eeb6e0a781e2f0528, 76),
        (0x357c299a88ea76a58924d52ce4f26a86, 79),
        (0xef2d0f5da7dd8aa27507bb7b07ea440a, 83),
        (0x8c240c4aecb13bb52a6c95fc0655033b, 86),
        (0x3ce9a36f23c0fc90eebd44c99eaa68fc, 89),
        (0xfb0f6be50601941b17953adc3110a7f9, 93),
        (0x95a5efea6b34767c12ddc8b027408661, 96),
        (0x4484bfeebc29f863424b06f3529a051b, 99),
        (0x039d66589687f9e901d59f290ee19daf, 102),
        (0x9f623d5a8a732974cfbc31db4b0295e5, 106),
        (0x4c4e977ba1f5bac3d9635b15d59bab1d, 109),
        (0x09d8792fb4c495697ab5e277de16227e, 112),
        (0xa95a5b7f87a0ef0f2abc9d8c9689d0c9, 116),
        (0x54484932d2e725a5bbca17a3aba173d4, 119),
        (0x1039d428a8b8eaeafca1ac82efb45caa, 122),
        (0xb38fb9daa78e44ab2dcf7a6b19209443, 126),
    ];

    // SAFETY: caller must make sure i is in range
    let magic = unsafe { GM_EXP_MAGICS.get_unchecked(i) };

    // (n + ((n * m) >> 128)) >> l
    let (high, _) = mul2(n, magic.0);

    // n + high may overflow, so
    // note: magic.1 = l - 1
    (high + ((n - high) >> 1)) >> magic.1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_calc_mul_div(a: i128, b: i128, c: i128) {
        // calc
        let Some(q) = a.calc_mul_div(b, c, Rounding::TowardsZero) else {
            return;
        };

        if q != 0 {
            assert_eq!(q >= 0, (a ^ b ^ c) >= 0);
        }

        let (x1, x2) = mul2(a.unsigned_abs(), b.unsigned_abs());
        let (y1, y2) = mul2(c.unsigned_abs(), q.unsigned_abs());

        let remain = if x1 == y1 {
            assert!(x2 >= y2);
            x2 - y2
        } else {
            assert!(x1 - 1 == y1);
            assert!(x2 < y2);
            u128::MAX - y2 + x2 + 1
        };

        assert!(remain < c.unsigned_abs());
    }

    fn check_calc_mul_div_signs(a: i128, b: i128, c: i128) {
        check_calc_mul_div(a, b, c);
        check_calc_mul_div(-a, b, c);
        check_calc_mul_div(a, b, -c);
        check_calc_mul_div(-a, b, -c);
    }

    #[test]
    fn test_mul_div_some() {
        check_calc_mul_div_signs(120, 7, 14);
        check_calc_mul_div_signs(120, 7, 17);
    }
    #[test]
    fn test_mul_div_big() { // TODO
    }
    #[test]
    fn test_mul_div_list() {
        let nums = [
            4,
            101,
            256,
            9999999,
            10000000,
            100000003,
            i32::MAX as i128,
            i32::MAX as i128 + 1,
            i32::MAX as i128 + 2,
            i32::MAX as i128 * 2 + 7,
            i64::MAX as i128,
            i64::MAX as i128 + 1,
            i64::MAX as i128 + 2,
            i64::MAX as i128 * 2 + 7,
            i128::MAX / 127,
            i128::MAX / 2,
            i128::MAX - 3,
            i128::MAX,
        ];
        let num2 = [
            1,
            3,
            7,
            10,
            100,
            10000,
            9999999,
            i32::MAX as i128,
            i64::MAX as i128,
        ];
        for a in nums {
            for c in nums {
                check_calc_mul_div_signs(a, a, c);
                check_calc_mul_div_signs(a, c, c);
                check_calc_mul_div_signs(a, c, a);
                for rat in num2 {
                    check_calc_mul_div_signs(a, a - rat, c - rat);
                    check_calc_mul_div_signs(a, a - rat, c / rat);
                    check_calc_mul_div_signs(a, a / rat, c / rat);
                    check_calc_mul_div_signs(a, a / rat, c - rat);

                    check_calc_mul_div_signs(a, c - rat, c - rat);
                    check_calc_mul_div_signs(a, c - rat, c / rat);
                    check_calc_mul_div_signs(a, c / rat, c / rat);
                    check_calc_mul_div_signs(a, c / rat, c - rat);

                    check_calc_mul_div_signs(a, c - rat, a - rat);
                    check_calc_mul_div_signs(a, c - rat, a / rat);
                    check_calc_mul_div_signs(a, c / rat, a / rat);
                    check_calc_mul_div_signs(a, c / rat, a - rat);
                }
            }
        }
    }

    // --- unsigned

    fn check_calc_mul_div_unsigned(a: u128, b: u128, c: u128) {
        // calc
        let Some(q) = a.calc_mul_div(b, c, Rounding::Floor) else {
            return;
        };

        let (x1, x2) = mul2(a, b);
        let (y1, y2) = mul2(c, q);

        let remain = if x1 == y1 {
            assert!(x2 >= y2);
            x2 - y2
        } else {
            assert!(x1 - 1 == y1);
            assert!(x2 < y2);
            u128::MAX - y2 + x2 + 1
        };
        assert!(remain < c);
    }

    #[test]
    fn test_mul_div_list_unsigned() {
        let nums = [
            4,
            101,
            256,
            9999999,
            10000000,
            100000003,
            u32::MAX as u128,
            u32::MAX as u128 + 1,
            u32::MAX as u128 + 2,
            u32::MAX as u128 * 2 + 7,
            u64::MAX as u128,
            u64::MAX as u128 + 1,
            u64::MAX as u128 + 2,
            u64::MAX as u128 * 2 + 7,
            u128::MAX / 127,
            u128::MAX / 2,
            u128::MAX - 3,
            u128::MAX,
        ];
        let num2 = [
            1,
            3,
            7,
            10,
            100,
            10000,
            9999999,
            u32::MAX as u128,
            u64::MAX as u128,
        ];
        for a in nums {
            for c in nums {
                check_calc_mul_div_unsigned(a, a, c);
                check_calc_mul_div_unsigned(a, c, c);
                check_calc_mul_div_unsigned(a, c, a);
                for rat in num2 {
                    let a2 = a.wrapping_sub(rat);
                    let c2 = c.wrapping_sub(rat);
                    check_calc_mul_div_unsigned(a, a2, c2);
                    check_calc_mul_div_unsigned(a, a2, c / rat);
                    check_calc_mul_div_unsigned(a, a / rat, c / rat);
                    check_calc_mul_div_unsigned(a, a / rat, c2);

                    check_calc_mul_div_unsigned(a, c2, c2);
                    check_calc_mul_div_unsigned(a, c2, c / rat);
                    check_calc_mul_div_unsigned(a, c / rat, c / rat);
                    check_calc_mul_div_unsigned(a, c / rat, c2);

                    check_calc_mul_div_unsigned(a, c2, a2);
                    check_calc_mul_div_unsigned(a, c2, a / rat);
                    check_calc_mul_div_unsigned(a, c / rat, a / rat);
                    check_calc_mul_div_unsigned(a, c / rat, a2);
                }
            }
        }
    }

    fn do_test_calc_mul_div(a: u128, b: u128, exp: u128, iexp: usize) {
        let q1 = a.calc_mul_div(b, exp, Rounding::Floor);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Floor);
        assert_eq!(q1, q2);
        let q1 = a.calc_mul_div(b, exp, Rounding::Ceiling);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Ceiling);
        assert_eq!(q1, q2);
        let q1 = a.calc_mul_div(b, exp, Rounding::Round);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Round);
        assert_eq!(q1, q2);
    }

    fn do_test_calc_mul_div_signed(a: i128, b: i128, exp: i128, iexp: usize) {
        // +-
        let q1 = a.calc_mul_div(b, exp, Rounding::Floor);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Floor);
        assert_eq!(q1, q2);
        let q1 = a.calc_mul_div(b, exp, Rounding::Ceiling);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Ceiling);
        assert_eq!(q1, q2);
        let q1 = a.calc_mul_div(b, exp, Rounding::Round);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Round);
        assert_eq!(q1, q2);

        // --
        let a = -a;
        let q1 = a.calc_mul_div(b, exp, Rounding::Floor);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Floor);
        assert_eq!(q1, q2);
        let q1 = a.calc_mul_div(b, exp, Rounding::Ceiling);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Ceiling);
        assert_eq!(q1, q2);
        let q1 = a.calc_mul_div(b, exp, Rounding::Round);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Round);
        assert_eq!(q1, q2);

        // ++
        let b = b.wrapping_neg();
        let q1 = a.calc_mul_div(b, exp, Rounding::Floor);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Floor);
        assert_eq!(q1, q2);
        let q1 = a.calc_mul_div(b, exp, Rounding::Ceiling);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Ceiling);
        assert_eq!(q1, q2);
        let q1 = a.calc_mul_div(b, exp, Rounding::Round);
        let q2 = a.calc_mul_div_exp(b, iexp, Rounding::Round);
        assert_eq!(q1, q2);
    }

    #[test]
    fn test_calc_mul_div() {
        for iexp in 1..39 {
            let exp = 10_u128.pow(iexp);

            for i in 0..iexp {
                let a = exp - i as u128;

                // enlarge this range for more test
                for j in 0..1000 {
                    let b = u128::MAX - j * 113;
                    do_test_calc_mul_div(a, b, exp, iexp as usize);
                }
                for j in 0..1000 {
                    let b = u128::MAX - j * 11113;
                    do_test_calc_mul_div(a, b, exp, iexp as usize);
                }

                // small values
                for j in 0..1000 {
                    let b = u64::MAX as u128 - j * 113;
                    do_test_calc_mul_div(a, b, exp, iexp as usize);
                }
            }

            // signed
            let exp = 10_i128.pow(iexp);

            for i in 0..iexp {
                let a = exp - i as i128;

                // enlarge this range for more test
                for j in 0..1000 {
                    let b = i128::MIN + j * 113 + 1;
                    do_test_calc_mul_div_signed(a, b, exp, iexp as usize);
                }
                for j in 0..1000 {
                    let b = i128::MIN + j * 111113 + 1;
                    do_test_calc_mul_div_signed(a, b, exp, iexp as usize);
                }

                // small values
                for j in 0..1000 {
                    let b = i64::MIN as i128 + j * 113;
                    do_test_calc_mul_div_signed(a, b, exp, iexp as usize);
                }
            }
        }
    }
}
