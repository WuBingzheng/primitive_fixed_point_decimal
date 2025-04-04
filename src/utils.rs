macro_rules! rounding_div {
    ($lhs:expr, $rhs:expr, $rounding:ident) => {
        'a: {
            if $rhs == 0 {
                break 'a None;
            }
            let d = $lhs / $rhs;
            let r = $lhs % $rhs;
            let is_carry = match $rounding {
                Rounding::Floor => 0,
                Rounding::Ceil => if r == 0 { 0 } else { 1 },
                Rounding::Round => if r * 2 < $rhs { 0 } else { 1 },
                Rounding::Unexpected => if r == 0 { 0 } else { break 'a None; }
            };
            Some(d + is_carry)
        }
    }
}

macro_rules! convert_lower {
    ($from:expr, $lower_type:ty) => {
        match $from {
            None => None,
            Some(r) => {
                let lower = r as $lower_type;
                if r > 0 {
                    if lower <= <$lower_type>::MAX { Some(lower) } else { None }
                } else {
                    if lower >= <$lower_type>::MIN { Some(lower) } else { None }
                }
            }
        }
    }
}

pub(crate) use rounding_div;
pub(crate) use convert_lower;
