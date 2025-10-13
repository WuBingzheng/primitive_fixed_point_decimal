macro_rules! define_none_scale_common {
    () => {
        /// The zero value.
        pub const ZERO: Self = Self(I::ZERO);

        /// The largest value, (2<sup>b</sup> - 1) / 10<sup>S</sup>, where `b`
        /// is the bits of `I`.
        pub const MAX: Self = Self(I::MAX);

        /// The smallest value, -(2<sup>b</sup> / 10<sup>S</sup>), where `b`
        /// is the bits of `I`.
        pub const MIN: Self = Self(I::MIN);

        /// The smallest difference value, 10<sup>-S</sup> .
        pub const EPSILON: Self = Self(I::ONE);

        /// The largest powers of 10.
        pub const MAX_POWERS: Self = Self(I::MAX_POWERS);

        /// Approximate number of significant digits in base 10.
        pub const DIGITS: u32 = I::DIGITS;
        /// Checked addition. Computes `self + rhs`, returning `None` if overflow occurred.
        ///
        /// The right operand must have the same scale with self.
        pub fn checked_add(self, rhs: Self) -> Option<Self> {
            self.0.checked_add(&rhs.0).map(Self)
        }

        /// Checked subtraction. Computes `self - rhs`, returning `None` if overflow occurred.
        ///
        /// The right operand must have the same scale with self.
        pub fn checked_sub(self, rhs: Self) -> Option<Self> {
            self.0.checked_sub(&rhs.0).map(Self)
        }

        /// Checked multiplication with integer. Computes `self * n`, returning
        /// `None` if overflow occurred.
        pub fn checked_mul_int(self, n: impl Into<I>) -> Option<Self> {
            self.0.checked_mul(&n.into()).map(Self)
        }

        /// Computes `self * a/b`, returning `None` if overflow occurred.
        ///
        /// Equivalent to [`Self::checked_mul_ratio_ext`] with `Rounding::Round`.
        pub fn checked_mul_ratio<R>(self, a: R, b: R) -> Option<Self>
        where
            R: IntoRatioInt<I>,
        {
            self.checked_mul_ratio_ext(a, b, Rounding::Round, None)
        }

        /// Computes `self * a/b`, returning `None` if overflow occurred.
        ///
        /// The arguments `a` and `b` could be primitive integers,
        /// `ConstScaleFpdec` or `OobScaleFpdec` with same scale.
        ///
        /// Compared to calling [`Self::checked_mul_int`] and [`Self::checked_div_int`]
        /// separately, this can avoid some potential overflow.
        ///
        /// Examples:
        ///
        /// ```
        /// use primitive_fixed_point_decimal::{ConstScaleFpdec, fpdec};
        /// type Balance = ConstScaleFpdec<i64, 2>;
        /// type Quantity = ConstScaleFpdec<i32, 4>; // type for `a` and `b`
        ///
        /// let margin: Balance = fpdec!(30000);
        /// let deal: Quantity = fpdec!(0.2);
        /// let total: Quantity = fpdec!(0.3);
        /// assert_eq!(margin.checked_mul_ratio(deal, total).unwrap(), fpdec!(20000));
        ///
        /// // integer
        /// assert_eq!(margin.checked_mul_ratio(200, 300).unwrap(), fpdec!(20000));
        /// ```
        pub fn checked_mul_ratio_ext<R>(
            self,
            a: R,
            b: R,
            rounding: Rounding,
            cum_err: Option<&mut CumErr<I>>,
        ) -> Option<Self>
        where
            R: IntoRatioInt<I>,
        {
            self.0
                .calc_mul_div(a.to_int(), b.to_int(), rounding, cum_err)
                .map(Self)
        }

        /// Checked division by integer, with `Rounding::Round`.
        ///
        /// Computes `self / n`, returning `None` if `n == 0` or overflow occurres.
        pub fn checked_div_int(self, n: impl Into<I>) -> Option<Self> {
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
            n: impl Into<I>,
            rounding: Rounding,
            cum_err: Option<&mut CumErr<I>>,
        ) -> Option<Self> {
            self.0
                .checked_div_with_opt_cum_err(n.into(), rounding, cum_err)
                .map(Self)
        }

        /// Return if zero.
        pub fn is_zero(&self) -> bool {
            self.0.is_zero()
        }

        /// Create a decimal from the underlying integer representation.
        ///
        /// You must take care of the scale yourself.
        pub const fn from_mantissa(i: I) -> Self {
            Self(i)
        }

        /// Return the underlying integer representation.
        ///
        /// You must take care of the scale yourself.
        pub const fn mantissa(self) -> I {
            self.0
        }
    };
}

macro_rules! define_none_scale_common_signed {
    () => {
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

        /// Return if negative.
        pub fn is_neg(&self) -> bool {
            self.0.is_negative()
        }

        /// Return if positive.
        pub fn is_pos(&self) -> bool {
            self.0.is_positive()
        }
    };
}

pub(crate) use define_none_scale_common;
pub(crate) use define_none_scale_common_signed;
