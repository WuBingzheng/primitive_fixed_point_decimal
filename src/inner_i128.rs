use crate::fpdec_inner::FpdecInner;
use int_div_cum_error::{checked_divide, Rounding};

impl FpdecInner for i128 {
    const MAX: Self = i128::MAX;
    const MIN: Self = i128::MIN;
    const MAX_POWERS: Self = 10_i128.pow(38);

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
        cum_error: Option<&mut Self>,
    ) -> Option<Self> {
        // happy path, no overflow
        if let Some(r) = self.checked_mul(b) {
            return checked_divide(r, c, rounding, cum_error);
        }

        // unhappy path
        let is_ab_neg = (self ^ b) < 0;

        // (mhigh, mlow) = a * b
        let (mhigh, mlow) = mul2(self.unsigned_abs(), b.unsigned_abs());

        // q = (mhigh, mlow) /  c
        div2(mhigh, mlow, is_ab_neg, c, rounding, cum_error)
    }
}

// calculate: a * b = (mhigh,mlow)
fn mul2(a: u128, b: u128) -> (u128, u128) {
    let (ahigh, alow) = (a >> 64, a & u64::MAX as u128);
    let (bhigh, blow) = (b >> 64, b & u64::MAX as u128);

    let (mid, carry1) = (alow * bhigh).overflowing_add(ahigh * blow);
    let (mlow, carry2) = (alow * blow).overflowing_add(mid << 64);
    let mhigh = ahigh * bhigh + (mid >> 64) + carry1 as u128 + carry2 as u128;
    (mhigh, mlow)
}

// calculate: (mhigh,mlow) / divisor
fn div2(
    mhigh: u128,
    mlow: u128,
    is_dividend_neg: bool,
    divisor: i128,
    rounding: Rounding,
    cum_error: Option<&mut i128>,
) -> Option<i128> {
    let is_divisor_neg = divisor < 0;
    let divisor = divisor.unsigned_abs();

    // check overflow or c==0
    if mhigh * 2 >= divisor {
        return None;
    }

    let mut q = 0;

    // consume @mhigh and reduce the dividend to @mlow
    let dividend = if mhigh != 0 {
        let mut dividend = mhigh;
        let mut total_shft = 0;
        loop {
            let zeros = dividend.leading_zeros();
            if zeros + total_shft >= 128 {
                break;
            }
            dividend = dividend << zeros | mlow << total_shft >> (128 - zeros);
            q = q << zeros | dividend / divisor;

            dividend %= divisor;
            total_shft += zeros;
        }

        q <<= 128 - total_shft;
        dividend << (128 - total_shft) | mlow << total_shft >> total_shft
    } else {
        mlow
    };

    // back to signed i128: dividend
    let mut dividend = match i128::try_from(dividend) {
        Ok(dividend) => dividend,
        Err(_) => {
            // one more division
            q = q.checked_add(dividend / divisor)?;
            (dividend % divisor) as i128
        }
    };
    if is_dividend_neg {
        dividend = -dividend;
    }

    // back to signed i128: divisor
    let mut divisor = divisor as i128;
    if is_divisor_neg {
        divisor = -divisor;
    }

    // back to signed i128: quotient
    let mut q = i128::try_from(q).ok()?;
    if is_divisor_neg != is_dividend_neg {
        q = -q;
    }

    // final division
    let last_q = checked_divide(dividend, divisor, rounding, cum_error)?;

    q.checked_add(last_q)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc_mul_add_div(a: i128, b: i128, e: i128, c: i128) -> i128 {
        let mut cum_error = 0;

        // happy path, no overflow
        if let Some(r) = a.checked_mul(b) {
            if let Some(r) = r.checked_add(e) {
                let r = checked_divide(r, c, Rounding::Round, Some(&mut cum_error)).unwrap();
                assert_eq!(cum_error, 0);
                return r;
            }
        }

        // unhappy path
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
        div2(
            mhigh,
            mlow,
            is_ab_neg,
            c,
            Rounding::Round,
            Some(&mut cum_error),
        )
        .unwrap()
    }

    fn check_calc_mul_div(a: i128, b: i128, c: i128) {
        // calc
        let mut cum_error = 0;
        let Some(q) = a.calc_mul_div(b, c, Rounding::Round, Some(&mut cum_error)) else {
            return;
        };

        // check
        if b != 0 {
            assert_eq!(calc_mul_add_div(q, c, cum_error, b), a);
        } else {
            assert_eq!(q, 0);
            assert_eq!(cum_error, 0);
        }

        if a != 0 {
            assert_eq!(calc_mul_add_div(q, c, cum_error, a), b);
        } else {
            assert_eq!(q, 0);
            assert_eq!(cum_error, 0);
        }
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
}
