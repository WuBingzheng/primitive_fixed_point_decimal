/// Rounding kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Rounding {
    /// towards the nearest integer
    #[default]
    Round,
    /// towards negative infinity
    Floor,
    /// towards positive infinity
    Ceiling,
    /// towards zero
    TowardsZero,
    /// away from zero
    AwayFromZero,
}

pub trait RoundingDiv: Sized {
    fn rounding_div(self, b: Self, rounding: Rounding) -> Option<Self>;
}

macro_rules! impl_for_signed {
    ($($int_type:ty,)+ ) => {$(
        impl RoundingDiv for $int_type {
            fn rounding_div(self, b: Self, rounding: Rounding) -> Option<Self> {
                let q = self.checked_div(b)?;
                let remain = self % b;
                if remain == 0{
                    return Some(q);
                }

                Some(match rounding {
                    Rounding::Floor => {
                        if (self ^ b) > 0 {
                            q
                        } else {
                            q - 1
                        }
                    }
                    Rounding::Ceiling => {
                        if (self ^ b) > 0 {
                            q + 1
                        } else {
                            q
                        }
                    }
                    Rounding::Round => {
                        let r = remain.unsigned_abs();
                        if r.saturating_add(r) >= b.unsigned_abs() {
                            if (self ^ b) > 0 {
                                q + 1
                            } else {
                                q - 1
                            }
                        } else {
                            q
                        }
                    }
                    Rounding::TowardsZero => q,
                    Rounding::AwayFromZero => {
                        if (self ^ b) > 0 {
                            q + 1
                        } else {
                            q - 1
                        }
                    }
                })
            }
        }
    )*};
}

macro_rules! impl_for_unsigned {
    ($($int_type:ty,)+ ) => {$(
        impl RoundingDiv for $int_type {
            fn rounding_div(self, b: Self, rounding: Rounding) -> Option<Self> {
                let q = self.checked_div(b)?;
                let remain = self % b;
                if remain == 0 {
                    return Some(q);
                }
                Some(match rounding {
                    Rounding::Floor | Rounding::TowardsZero => q,
                    Rounding::Ceiling | Rounding::AwayFromZero => q + 1,
                    Rounding::Round => {
                        if remain.saturating_add(remain) >= b {
                            q + 1
                        } else {
                            q
                        }
                    }
                })
            }
        }
    )*};
}

impl_for_signed!(i8, i16, i32, i64, i128,);
impl_for_unsigned!(u8, u16, u32, u64, u128,);
