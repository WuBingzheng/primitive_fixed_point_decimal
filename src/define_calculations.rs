macro_rules! define_calculations {
    (
        $inner_type:ty,
        $digits:expr
    ) => {
        use std::str::FromStr;

        pub const fn checked_mul_with_rounding(
            a: $inner_type,
            b: $inner_type,
            diff_precision: i32, // = P + Q - R
            rounding: Rounding,
        ) -> Option<$inner_type> {
        
            if diff_precision > 0 {
                // a * b / diff_exp
                if diff_precision <= $digits {
                    calc_mul_div(a, b, ALL_EXPS[diff_precision as usize], rounding)

                } else if diff_precision <= $digits * 2 {
                    let diff_diff = diff_precision as usize - $digits;
                    let Some(tmp) = calc_mul_div(a, b, ALL_EXPS[diff_diff], rounding) else {
                        return None;
                    };
                    rounding_div!(tmp, ALL_EXPS[$digits], rounding)
                } else {
                    Some(0)
                }
        
            } else if diff_precision < 0 {
                // a * b * diff_exp
                let Some(r) = a.checked_mul(b) else {
                    return None;
                };
                if -diff_precision <= $digits {
                    r.checked_mul(ALL_EXPS[-diff_precision as usize])
                } else {
                    None
                }
        
            } else {
                a.checked_mul(b)
            }
        }
        
        pub const fn checked_div_with_rounding(
            a: $inner_type,
            b: $inner_type,
            diff_precision: i32, // = P - Q - R
            rounding: Rounding,
        ) -> Option<$inner_type> {
            if diff_precision > 0 {
                // a / b / diff_exp
                if b == 0 {
                    None
                } else if diff_precision <= $digits {
                    rounding_div!(a / b, ALL_EXPS[diff_precision as usize], rounding)
                } else {
                    Some(0)
                }
        
            } else if diff_precision < 0 {
                // a * diff_exp / b
                let abs_diff = -diff_precision as usize;
                if abs_diff <= $digits {
                    calc_mul_div(a, ALL_EXPS[abs_diff], b, rounding)
                } else if -diff_precision <= $digits * 2 {
                    let Some(tmp) = a.checked_mul(ALL_EXPS[$digits - abs_diff]) else {
                        return None;
                    };
                    calc_mul_div(tmp, ALL_EXPS[$digits], b, rounding)
                } else {
                    None
                }
        
            } else {
                rounding_div!(a, b, rounding)
            }
        }
        
        // diff_precision = src - dst
        pub const fn rescale(a: $inner_type, diff_precision: i32) -> Option<$inner_type> {
            if diff_precision > 0 {
                // to bigger precision
                if diff_precision <= $digits {
                    let exp = ALL_EXPS[diff_precision as usize];
                    if a % exp == 0 {
                        Some(a / exp)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if diff_precision < 0 {
                // to smaller precision
                if -diff_precision <= $digits {
                    a.checked_mul(ALL_EXPS[-diff_precision as usize])
                } else {
                    None
                }
            } else {
                Some(a)
            }
        }

        // diff_precision = src - dst
        pub const fn shrink_to_with_rounding(
            a: $inner_type,
            diff_precision: i32,
            rounding: Rounding,
        ) -> Option<$inner_type> {

            if diff_precision <= 0 {
                Some(a)

            } else if diff_precision >= $digits {
                if matches!(rounding, Rounding::Unexpected) && a != 0 {
                    return None;
                }
                Some(0)

            } else {
                let exp = ALL_EXPS[diff_precision as usize];
                let ret = a / exp * exp;
                let remain = a - ret;
                let carry = match rounding {
                    Rounding::Floor => 0,
                    Rounding::Ceil => if remain == 0 { 0 } else { exp },
                    Rounding::Round => if remain * 2 < exp { 0 } else { exp },
                    Rounding::Unexpected => if remain == 0 { 0 } else { return None },
                };
                Some(ret + carry)
            }
        }

        pub fn try_from_str(s: &str, precision: i32) -> Result<$inner_type, ParseError> {
            // sign
            let (s, is_neg) = match s.as_bytes().first() {
                None => return Err(ParseError::Empty),
                Some(b'-') => (&s[1..], true),
                Some(b'+') => (&s[1..], false),
                _ => (s, false),
            };

            if s == "0" || s == "0." {
                return Ok(0);
            }
            if s.is_empty() {
                return Err(ParseError::Empty);
            }

            // fraction part
            let (int_str, frac_num) = if let Some((int_str, frac_str)) = s.split_once('.') {
                let frac_len = frac_str.len();
                if frac_len as i32 > precision {
                    return Err(ParseError::Precision);
                }

                // here precision > 0
                let precision = precision as usize;

                let mut frac_num = <$inner_type>::from_str(frac_str)?;

                if frac_len < precision {
                    let diff_exp = *ALL_EXPS.get(precision - frac_len)
                        .ok_or(ParseError::Overflow)?;
                    frac_num = frac_num.checked_mul(diff_exp)
                        .ok_or(ParseError::Overflow)?;
                }

                (int_str, frac_num)
            } else {
                (s, 0)
            };

            // integer part
            let inner = if precision > $digits {
                if int_str != "0" {
                    return Err(ParseError::Overflow);
                }
                frac_num
            } else if precision >= 0 {
                <$inner_type>::from_str(int_str)?
                    .checked_mul(ALL_EXPS[precision as usize])
                    .ok_or(ParseError::Overflow)?
                    .checked_add(frac_num)
                    .ok_or(ParseError::Overflow)?

            } else {
                if s.len() <= -precision as usize {
                    return Err(ParseError::Precision);
                }
                let end = s.len() - (-precision) as usize;
                if *&int_str[end..].chars().all(|ch| ch == '0') {
                    return Err(ParseError::Precision);
                }

                <$inner_type>::from_str(&int_str[..end])?
            };

            if is_neg { Ok(-inner) } else { Ok(inner) }
        }

        fn display_fmt(a: $inner_type, precision: i32, f: &mut fmt::Formatter)
            -> Result<(), fmt::Error> 
        {
            if a == 0 {
                return write!(f, "0");
            }
            if precision == 0 {
                return write!(f, "{}", a);
            }
            if precision < 0 {
                return write!(f, "{}{:0>width$}", a, 0, width=(-precision) as usize);
            }

            // precision > 0
            let precision = precision as usize;

            if precision <= $digits {
                let exp = ALL_EXPS[precision];
                let i = a / exp;
                let mut frac = a % exp;
                if frac == 0 {
                    write!(f, "{}", i)
                } else {
                    if frac < 0 {
                        frac = -frac;
                    }
                    while frac % 10 == 0 {
                        frac /= 10;
                    }
                    write!(f, "{}.{}", i, frac)
                }
            } else if a >= 0 {
                write!(f, "0.{:0>width$}", a, width=precision)
            } else {
                write!(f, "-0.{:0>width$}", a.unsigned_abs(), width=precision)
            }
        }

        fn check_from_int(i2: $inner_type, precision: i32) -> Option<$inner_type> {
            if precision > $digits {
                None
            } else if precision > 0 {
                i2.checked_mul(ALL_EXPS[precision as usize])
            } else if -precision > $digits {
                Some(0)
            } else {
                i2.checked_div(ALL_EXPS[-precision as usize])
            }
        }
    }
}

pub(crate) use define_calculations;
