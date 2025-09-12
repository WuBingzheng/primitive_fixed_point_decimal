use crate::fpdec_inner::FpdecInner;
use int_div_cum_error::{CumErr, DivCumErr, Rounding};

impl FpdecInner for i128 {
    const MAX: Self = i128::MAX;
    const MIN: Self = i128::MIN;
    const TEN: Self = 10;
    const HUNDRED: Self = 100;
    const MAX_POWERS: Self = 10_i128.pow(Self::DIGITS);
    const DIGITS: u32 = i128::MAX.ilog10();
    const NEG_MIN_STR: &'static str = "170141183460469231731687303715884105728";

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

    fn calc_mul_div(
        self,
        b: Self,
        c: Self,
        rounding: Rounding,
        cum_err: Option<&mut CumErr<Self>>,
    ) -> Option<Self> {
        // happy path, no overflow
        if let Some(r) = self.checked_mul(b) {
            return r.checked_div_with_opt_cum_err(c, rounding, cum_err);
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
        let last_q = last_dividend.checked_div_with_opt_cum_err(c, rounding, cum_err)?;

        q.checked_add(last_q)
    }
}

impl FpdecInner for u128 {
    const MAX: Self = u128::MAX;
    const MIN: Self = u128::MIN;
    const TEN: Self = 10;
    const HUNDRED: Self = 100;
    const MAX_POWERS: Self = 10_u128.pow(Self::DIGITS);
    const DIGITS: u32 = u128::MAX.ilog10();

    #[doc(hidden)]
    const NEG_MIN_STR: &'static str = "unreachable";

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

    fn calc_mul_div(
        self,
        b: Self,
        c: Self,
        rounding: Rounding,
        cum_err: Option<&mut CumErr<Self>>,
    ) -> Option<Self> {
        // happy path, no overflow
        if let Some(r) = self.checked_mul(b) {
            return r.checked_div_with_opt_cum_err(c, rounding, cum_err);
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
        let last_q = last_dividend.checked_div_with_opt_cum_err(c, rounding, cum_err)?;

        q.checked_add(last_q)
    }
}

// calculate: a * b = (mhigh,mlow)
fn mul2(a: u128, b: u128) -> (u128, u128) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn calc_mul_add_div(a: i128, b: i128, e: i128, c: i128) -> i128 {
        let mut cum_err = CumErr::new();

        // happy path, no overflow
        if let Some(r) = a.checked_mul(b) {
            if let Some(r) = r.checked_add(e) {
                let r = r
                    .checked_div_with_cum_err(c, Rounding::Round, &mut cum_err)
                    .unwrap();
                assert_eq!(cum_err, Default::default());
                return r;
            }
        }

        // normal path
        let is_ab_neg = (a ^ b) < 0;
        let is_add_neg = (a ^ b ^ e) < 0;
        let a = a.unsigned_abs();
        let b = b.unsigned_abs();

        // (mhigh, mlow) = a * b
        let (mut mhigh, mut mlow) = mul2(a, b);

        if !is_add_neg {
            let (tmp, carry) = mlow.overflowing_add(e.unsigned_abs());
            mlow = tmp;
            if carry {
                mhigh += 1;
            }
        } else {
            let (tmp, carry) = mlow.overflowing_sub(e.unsigned_abs());
            mlow = tmp;
            if carry {
                mhigh -= 1;
            }
        }

        // q = (mhigh, mlow) /  c
        let unsigned_c = c.unsigned_abs();
        let (last_dividend, mut q) = reduce2(mhigh, mlow, unsigned_c, 2).unwrap();

        // back to signed i128: last_dividend
        let mut last_dividend = match i128::try_from(last_dividend) {
            Ok(dividend) => dividend,
            Err(_) => {
                // one more division
                q = q.checked_add(last_dividend / unsigned_c).unwrap();
                (last_dividend % unsigned_c) as i128
            }
        };
        if is_ab_neg {
            last_dividend = -last_dividend;
        }

        // back to signed i128: quotient
        let mut q = i128::try_from(q).unwrap();
        if is_ab_neg != (c < 0) {
            q = -q;
        }

        // final division
        let last_q = last_dividend
            .checked_div_with_cum_err(c, Rounding::Round, &mut cum_err)
            .unwrap();

        q.checked_add(last_q).unwrap()
    }

    fn check_calc_mul_div(a: i128, b: i128, c: i128) {
        // calc
        let mut cum_err = CumErr::new();
        let Some(q) = a.calc_mul_div(b, c, Rounding::Round, Some(&mut cum_err)) else {
            return;
        };

        if b != 0 {
            assert_eq!(calc_mul_add_div(q, c, cum_err.0, b), a);
        } else {
            assert_eq!(q, 0);
            assert_eq!(cum_err.0, 0);
        }

        if a != 0 {
            assert_eq!(calc_mul_add_div(q, c, cum_err.0, a), b);
        } else {
            assert_eq!(q, 0);
            assert_eq!(cum_err.0, 0);
        }
        //TODO
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

    fn calc_mul_add_div_unsigned(a: u128, b: u128, e: u128, c: u128) -> u128 {
        let mut cum_err = CumErr::new();

        // happy path, no overflow
        if let Some(r) = a.checked_mul(b) {
            if let Some(r) = r.checked_add(e) {
                let r = r
                    .checked_div_with_cum_err(c, Rounding::Floor, &mut cum_err)
                    .unwrap();
                assert_eq!(cum_err, Default::default());
                return r;
            }
        }

        // normal path

        // (mhigh, mlow) = a * b
        let (mut mhigh, mut mlow) = mul2(a, b);

        // add cum_err
        let (tmp, carry) = mlow.overflowing_add(e);
        mlow = tmp;
        if carry {
            mhigh += 1;
        }

        // q = (mhigh, mlow) /  c
        let (last_dividend, q) = if c.leading_zeros() != 0 {
            reduce2(mhigh, mlow, c, 1).unwrap()
        } else {
            reduce2_big(mhigh, mlow, c).unwrap()
        };

        // final division
        let last_q = last_dividend
            .checked_div_with_cum_err(c, Rounding::Floor, &mut cum_err)
            .unwrap();

        q.checked_add(last_q).unwrap()
    }

    fn check_calc_mul_div_unsigned(a: u128, b: u128, c: u128) {
        // calc
        let mut cum_err = CumErr::new();
        let Some(q) = a.calc_mul_div(b, c, Rounding::Floor, Some(&mut cum_err)) else {
            return;
        };

        if b != 0 {
            assert_eq!(calc_mul_add_div_unsigned(q, c, cum_err.0, b), a);
        } else {
            assert_eq!(q, 0);
            assert_eq!(cum_err.0, 0);
        }

        if a != 0 {
            assert_eq!(calc_mul_add_div_unsigned(q, c, cum_err.0, a), b);
        } else {
            assert_eq!(q, 0);
            assert_eq!(cum_err.0, 0);
        }
        //TODO
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
}
