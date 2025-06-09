macro_rules! define_none_prec_common {
    () => {
        /// The zero value.
        pub const ZERO: Self = Self(I::ZERO);

        /// The largest value, (2<sup>b</sup> - 1) / 10<sup>P</sup>, where `b`
        /// is the bits of `I`.
        pub const MAX: Self = Self(I::MAX);

        /// The smallest value, -(2<sup>b</sup> / 10<sup>P</sup>), where `b`
        /// is the bits of `I`.
        pub const MIN: Self = Self(I::MIN);

        /// The smallest positive value, 10<sup>-P</sup> .
        pub const MIN_POSITIVE: Self = Self(I::ONE);

        /// The largest powers of 10.
        pub const MAX_POWERS: Self = Self(I::MAX_POWERS);

        /// Computes the absolute value of self.
        ///
        /// # Overflow behavior
        ///
        /// The absolute value of `MIN` cannot be represented as this type,
        /// and attempting to calculate it will cause an overflow. This means that
        /// code in debug mode will trigger a panic on this case and optimized code
        /// will return `MIN` without a panic.
        pub fn abs(self) -> Self {
            Self(self.0.abs())
        }

        /// Checked absolute value. Computes `self.abs()`, returning `None` if `self == MIN`.
        pub fn checked_abs(self) -> Option<Self> {
            if self == Self::MIN {
                None
            } else {
                Some(self.abs())
            }
        }

        /// Checked addition. Computes `self + rhs`, returning `None` if overflow occurred.
        ///
        /// The right operand must have the same precision with self.
        pub fn checked_add(self, rhs: Self) -> Option<Self> {
            self.0.checked_add(&rhs.0).map(Self)
        }

        /// Checked subtraction. Computes `self - rhs`, returning `None` if overflow occurred.
        ///
        /// The right operand must have the same precision with self.
        pub fn checked_sub(self, rhs: Self) -> Option<Self> {
            self.0.checked_sub(&rhs.0).map(Self)
        }

        /// Checked multiplication with integer. Computes `self * n`, returning
        /// `None` if overflow occurred.
        pub fn checked_mul_int(self, n: I) -> Option<Self> {
            self.0.checked_mul(&n).map(Self)
        }

        /// Checked division by integer, with `Rounding::Round`.
        ///
        /// Computes `self / n`, returning `None` if `n == 0` or overflow occurres.
        pub fn checked_div_int(self, n: I) -> Option<Self> {
            self.checked_div_int_ext(n, Rounding::Round, None)
        }

        /// Checked division by integer with rounding and cumulative-error.
        ///
        /// Computes `self / n`, returning `None` if `n == 0` or overflow occurres.
        ///
        /// See the [cumulative error section](index.html#cumulative-error)
        /// for more information and examples.
        pub fn checked_div_int_ext(
            self,
            n: I,
            rounding: Rounding,
            cum_error: Option<&mut I>,
        ) -> Option<Self> {
            checked_divide(self.0, n, rounding, cum_error).map(Self)
        }

        /// Return if negative.
        pub fn is_neg(&self) -> bool {
            self.0.is_negative()
        }

        /// Return if positive.
        pub fn is_pos(&self) -> bool {
            self.0.is_positive()
        }

        /// Return if zero.
        pub fn is_zero(&self) -> bool {
            self.0.is_zero()
        }

        /// Create a decimal from the underlying integer representation.
        ///
        /// You must take care of the precision yourself.
        pub const fn from_mantissa(i: I) -> Self {
            Self(i)
        }

        /// Return the underlying integer representation.
        ///
        /// You must take care of the precision yourself.
        pub const fn mantissa(self) -> I {
            self.0
        }
    };
}

pub(crate) use define_none_prec_common;
