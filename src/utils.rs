macro_rules! rounding_div {
    ($lhs:expr, $rhs:expr, $rounding:expr, $cum_error:expr) => {
        'a: {
            if $rhs == 0 {
                break 'a None;
            }
            let mut tmp_cum_error = *$cum_error;

            let mut d = $lhs / $rhs;
            let r = $lhs % $rhs;

            match tmp_cum_error.checked_add(r) {
                Some(tmp) => tmp_cum_error = tmp,
                None => {
                    d += 1;
                    tmp_cum_error += r - $rhs;
                }
            }

            let is_carry = match $rounding {
                Rounding::Floor => false,
                Rounding::Ceil => tmp_cum_error > 0,
                Rounding::Round => tmp_cum_error * 2 > $rhs,
                Rounding::Unexpected => if tmp_cum_error == 0 { false } else { break 'a None; }
            };

            if is_carry {
                d += 1;
                *$cum_error = tmp_cum_error - $rhs;
            } else {
                *$cum_error = tmp_cum_error;
            }

            Some(d)
        }
    }
}

macro_rules! calc_mul_div_higher {
    ($a:expr, $b:expr, $c:expr, $rounding:expr, $cum_error:expr, $origin_type:ty, $higher_type:ty) => {
        {
        let mut higher_cum_error = *$cum_error as $higher_type;
        match rounding_div!($a as $higher_type * $b as $higher_type,
            $c as $higher_type,
            $rounding,
            &mut higher_cum_error) {

            None => None,
            Some(r) => {
                *$cum_error = higher_cum_error as $origin_type;

                let lower = r as $origin_type;
                if r > 0 {
                    if lower <= <$origin_type>::MAX { Some(lower) } else { None }
                } else {
                    if lower >= <$origin_type>::MIN { Some(lower) } else { None }
                }
            }
        }
        }
    }
}

pub(crate) use rounding_div;
pub(crate) use calc_mul_div_higher;
