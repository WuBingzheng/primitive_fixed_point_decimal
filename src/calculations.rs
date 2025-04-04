macro_rules! define_calculations {
    (
        $inner_type:ty,
        $digits:expr
    ) => {

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
                    let Some(tmp) = calc_mul_div(a, b, ALL_EXPS[$digits], rounding) else {
                        return None;
                    };
                    tmp.checked_div(ALL_EXPS[diff_precision as usize - $digits])
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
                // a / (diff_exp * b)
                if diff_precision <= $digits {
                    calc_div_div(a, ALL_EXPS[diff_precision as usize], b, rounding)
                } else if b == 0 {
                    None
                } else {
                    Some(0)
                }
        
            } else if diff_precision < 0 {
                // a * diff_exp / b
                if -diff_precision <= $digits {
                    calc_mul_div(a, ALL_EXPS[-diff_precision as usize], b, rounding)
                } else if -diff_precision <= $digits * 2 {
                    let Some(tmp) = calc_mul_div(a, ALL_EXPS[$digits], b, rounding) else {
                        return None;
                    };
                    tmp.checked_mul(ALL_EXPS[diff_precision as usize - $digits])
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

        pub fn try_from_str(s: &str, mut precision: i32) -> Result<$inner_type, ParseError> {
            // sign
            let (s, is_neg) = match s.as_bytes().first() {
                None => return Err(ParseError::Empty),
                Some(b'-') => (&s[1..], true),
                Some(b'+') => (&s[1..], false),
                _ => (s, false),
            };
        
            if s.is_empty() {
                return Err(ParseError::Empty);
            }

            // negative precision, trim the tailing digits
            let s = if precision < 0 {
                if s.len() <= -precision as usize {
                    return Err(ParseError::Precision);
                }
                let end = s.len() - (-precision) as usize;
                if *&s[end..].chars().all(|ch| ch == '0') {
                    return Err(ParseError::Precision);
                }
                precision = 0;
                &s[..end]
            } else {
                s
            };

            // parse
            let mut point = None;
            let mut int_num: $inner_type = 0;
            for (i, ch) in s.chars().enumerate() {
                if ch == '.' {
                    if point.is_some() {
                        return Err(ParseError::Invalid);
                    }
                    point = Some(i);
                    continue;
                }

                if ch < '0' || ch > '9' {
                    return Err(ParseError::Invalid);
                }

                if let Some(p) = point {
                    if i - p > precision as usize {
                        return Err(ParseError::Precision);
                    }
                }

                int_num = int_num.checked_mul(10)
                    .ok_or(ParseError::Overflow)?;
                int_num = int_num.checked_add((ch as u32 - '0' as u32) as $inner_type)
                    .ok_or(ParseError::Overflow)?;
            }

            // rescale
            let fracs = match point {
                Some(point) => s.len() - point - 1,
                None => 0,
            };

            if precision as usize > fracs {
                int_num = int_num.checked_mul(ALL_EXPS[precision as usize - fracs])
                    .ok_or(ParseError::Overflow)?;
            }

            if is_neg {
                int_num = -int_num;
            }

            Ok(int_num)
        }

        fn display_fmt(a: $inner_type, precision: i32, f: &mut fmt::Formatter)
            -> Result<(), fmt::Error> 
        {
            let exp = ALL_EXPS[precision as usize];
            let intg = a / exp;
            let mut frac = a % exp;
            if frac == 0 {
                return write!(f, "{}", intg);
            }

            if intg == 0 && a < 0 {
                write!(f, "-0.")?;
                frac = -frac;
            } else {
                write!(f, "{}.", intg)?;
                frac = frac.abs();
            }

            if let Some(fmt_prec) = f.precision() {
                if fmt_prec < precision as usize {
                    let exp = ALL_EXPS[precision as usize - fmt_prec];
                    let rem = frac % exp;
                    frac = frac / exp + if rem * 2 >= exp { 1 } else { 0 };
                    write!(f, "{:0width$}", frac, width=fmt_prec)
                } else {
                    write!(f, "{:0width$}", frac, width=precision as usize)
                }
            } else if precision > 0 {
                let mut ie = precision as usize - 1;
                while frac != 0 {
                    let exp = ALL_EXPS[ie];
                    write!(f, "{}", frac / exp)?;
                    frac %= exp;
                    ie -= 1;
                }
                Ok(())
            } else {
                Ok(())
            }
        }
    }
}

pub(crate) use define_calculations;
