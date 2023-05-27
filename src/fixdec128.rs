use crate::define_macro::*;

// define and implement FixDec128.
define_fixdec!(FixDec128, i128, 38, 128, 127);

// convert FixDec64 into other FixDec types.
convert_try_into!(FixDec128, fixdec16, FixDec16);
convert_try_into!(FixDec128, fixdec32, FixDec32);
convert_try_into!(FixDec128, fixdec64, FixDec64);

// internal stuff needed by define_macro
const ALL_EXPS: [i128; 38 + 1] = [1,
    10_i128.pow(1), 10_i128.pow(2), 10_i128.pow(3), 10_i128.pow(4),
    10_i128.pow(5), 10_i128.pow(6), 10_i128.pow(7), 10_i128.pow(8),
    10_i128.pow(9), 10_i128.pow(10), 10_i128.pow(11), 10_i128.pow(12),
    10_i128.pow(13), 10_i128.pow(14), 10_i128.pow(15), 10_i128.pow(16),
    10_i128.pow(17), 10_i128.pow(18), 10_i128.pow(19), 10_i128.pow(20),
    10_i128.pow(21), 10_i128.pow(22), 10_i128.pow(23), 10_i128.pow(24),
    10_i128.pow(25), 10_i128.pow(26), 10_i128.pow(27), 10_i128.pow(28),
    10_i128.pow(29), 10_i128.pow(30), 10_i128.pow(31), 10_i128.pow(32),
    10_i128.pow(33), 10_i128.pow(34), 10_i128.pow(35), 10_i128.pow(36),
    10_i128.pow(37), 10_i128.pow(38),
];

const fn calc_mul_div(a: i128, b: i128, c: i128, rounding: Rounding) -> Option<i128> {
    if let Some(r) = a.checked_mul(b) {
        rounding_div_i128(r, c, rounding)
    } else {
        let is_neg = (a < 0) ^ (b < 0) ^ (c < 0);
        let a = a.unsigned_abs();
        let b = b.unsigned_abs();
        let c = c.unsigned_abs();

        // calculate: (mhigh,mlow) = a * b
        let (ahigh, alow) = (a >> 64, a & u64::MAX as u128);
        let (bhigh, blow) = (b >> 64, b & u64::MAX as u128);
        let (mid, carry1) = (alow * bhigh).overflowing_add(ahigh * blow);
        let (mlow, carry2) = (alow * blow).overflowing_add(mid << 64);
        let mhigh = ahigh * bhigh + (mid >> 64) + carry1 as u128 + carry2 as u128;

        // overflow or c==0
        if mhigh * 2 >= c {
            return None;
        }

        // calculate: r = (mhigh,mlow) / c
        let r = if c <= u64::MAX as u128 {
            // special case: `c` has no more 64 bits
            let part1 = (mhigh << 64) | (mlow >> 64);
            let quotient1 = part1 / c;
            let remainder1 = part1 % c;

            let part2 = (remainder1 << 64) | (mlow & u64::MAX as u128);
            let Some(quotient2) = rounding_div_u128(part2, c, is_neg, rounding) else {
                return None;
            };

            debug_assert!(quotient1 <= u64::MAX as u128);
            debug_assert!(quotient2 <= u64::MAX as u128);
            quotient1 << 64 | quotient2

        } else {
            // generic case
            let mut dividend = mhigh;
            let mut r = 0;
            let mut total_shft = 0;
            loop {
                let zeros = dividend.leading_zeros();
                if zeros + total_shft < 128 {
                    dividend = dividend << zeros | mlow << total_shft >> (128 - zeros);
                    r = r << zeros | dividend / c;

                    dividend %= c;
                    total_shft += zeros;
                } else {
                    let shft = 128 - total_shft;
                    dividend = dividend << shft | mlow << total_shft >> (128 - shft);
                    let Some(quotient) = rounding_div_u128(dividend, c, is_neg, rounding) else {
                        return None;
                    };
                    break r << shft | quotient;
                }
            }
        };

        if r > i128::MAX as u128 {
            return None;
        }
        let r = r as i128;
        if is_neg { Some(-r) } else { Some(r) }
    }
}

const fn calc_div_div(a: i128, b: i128, c: i128, rounding: Rounding) -> Option<i128> {
    if let Some(r) = b.checked_mul(c) {
        rounding_div_i128(a, r, rounding)
    } else {
        let is_neg = (a < 0) ^ (b < 0) ^ (c < 0);
        let more_half = if let Some(r) = b.checked_mul(c/2) {
            a >= r
        } else {
            false
        };

        if let Some(carry) = rounding_carry(a == 0, more_half, is_neg, rounding) {
            Some(carry as i128)
        } else {
            None
        }
    }
}

const fn rounding_div_u128(lhs: u128, rhs: u128, is_neg: bool, kind: Rounding) -> Option<u128> {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if let Some(carry) = rounding_carry(r == 0, r >= rhs / 2, is_neg, kind) {
        if carry == -1 {
            Some(d - 1)
        } else {
            Some(d + carry as u128)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_mul_div(a: i128, b: i128, c: i128, r: i128) {
        assert_eq!(calc_mul_div(a, b, c, Rounding::Unexpected), Some(r));
    }

    fn check_mul_div_pow(a: u32, b: u32, c: u32) {
        let r = 10_i128.pow(a + b - c);
        let a = 10_i128.pow(a);
        let b = 10_i128.pow(b);
        let c = 10_i128.pow(c);
        check_mul_div(a, b, c, r);
    }

    #[test]
    fn test_mul_div() {
        assert_eq!(calc_mul_div(1, 1, 0, Rounding::Round), None);

        assert_eq!(calc_mul_div(-100, 100, 7, Rounding::Round), Some(-1429));
        assert_eq!(calc_mul_div(100, -100, 7, Rounding::Up), Some(-1427)); // -1428
        assert_eq!(calc_mul_div(100, 100, -7, Rounding::Down), Some(-1428));
        assert_eq!(calc_mul_div(-100, -100, 7, Rounding::Unexpected), None);

        let max16 = u16::MAX as i128;
        check_mul_div(max16, max16, max16, max16);
        check_mul_div(max16/2, max16*2, max16, max16-1);

        let max64 = u64::MAX as i128;
        check_mul_div(max64/2, max64, max64, max64/2);
        check_mul_div(max64*99, max64*99, max64*99*9, max64*11);
        check_mul_div(max64*2, max64*2, max64, max64*4);
        check_mul_div(max64*8, max64*9, max64*6, max64*12);

        let big64 = u64::MAX as i128 - 15;
        check_mul_div(big64*2, big64*2, big64/2, big64*8);
        check_mul_div(big64/2, big64/2, big64*2, big64/8);
        check_mul_div(big64/2, big64/2, big64/2, big64/2);
        check_mul_div(big64*2, big64*2, big64*2, big64*2);

        let max128 = i128::MAX;
        check_mul_div(max128, max128, max128, max128);
        check_mul_div(max64, max64/2, 1, max128-max64-max64/2);

        check_mul_div_pow(0, 0, 0);
        check_mul_div_pow(1, 1, 0);
        check_mul_div_pow(3, 8, 10);
        check_mul_div_pow(13, 18, 10);
        check_mul_div_pow(13, 18, 20);
        check_mul_div_pow(13, 18, 30);
        check_mul_div_pow(38, 38, 38);
        check_mul_div_pow(18, 20, 38);
        check_mul_div_pow(38, 1, 1);
        check_mul_div_pow(38, 10, 12);
        check_mul_div_pow(38, 1, 10);
        check_mul_div_pow(37, 37, 37);

        assert_eq!(calc_mul_div(max64, max64*2, 1, Rounding::Round), None);
        assert_eq!(calc_mul_div(max64, max64*8, 0, Rounding::Round), None);
    }
}
