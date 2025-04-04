// common part of StaticFpdec* and OobExpFpdec* types.
macro_rules! define_common {
    (
        $fpdec_type:ident,
        $inner_type:ty,

        // These are used only in doc comments.
        $bits_minus_one:literal,
        $type_in_doc:ty
    ) => {
        /// The zero value, 0.
        pub const ZERO: Self = Self { inner: 0 };

        #[doc = concat!("The largest value, (2<sup>", $bits_minus_one, "</sup> - 1) / 10<sup>P</sup>.")]
        pub const MAX: Self = Self { inner: <$inner_type>::MAX };

        #[doc = concat!("The smallest value, -(2<sup>", $bits_minus_one, "</sup> / 10<sup>P</sup>).")]
        pub const MIN: Self = Self { inner: <$inner_type>::MIN };

        /// The smallest positive value, 10<sup>-P</sup> .
        pub const MIN_POSITIVE: Self = Self { inner: 1 };

        /// Computes the absolute value of self.
        /// 
        /// # Overflow behavior
        ///
        /// The absolute value of `MIN` cannot be represented as this type,
        /// and attempting to calculate it will cause an overflow. This means that
        /// code in debug mode will trigger a panic on this case and optimized code
        /// will return `MIN` without a panic.
        /// 
        /// # Examples
        /// 
        /// ```
        #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
        #[doc = concat!("type Decimal = ", stringify!($type_in_doc), ";")]
        ///
        /// assert_eq!(Decimal::MAX.abs(), Decimal::MAX);
        /// assert_eq!((-Decimal::MAX).abs(), Decimal::MAX);
        /// assert_eq!(Decimal::ZERO.abs(), Decimal::ZERO);
        /// ```
        pub const fn abs(self) -> Self {
            Self { inner: self.inner.abs() }
        }

        /// Checked absolute value. Computes `self.abs()`, returning `None` if `self == MIN`.
        /// 
        /// # Examples
        /// 
        /// ```
        #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
        #[doc = concat!("type Decimal = ", stringify!($type_in_doc), ";")]
        ///
        /// assert_eq!(Decimal::MIN.checked_abs(), None);
        /// ```
        pub const fn checked_abs(self) -> Option<Self> {
            Self::from_opt_inner(self.inner.checked_abs())
        }

        /// Checked addition. Computes `self + rhs`, returning `None` if overflow occurred.
        ///
        /// The right operand must have the same precision with self. So you can not add
        #[doc = concat!("`", stringify!($type_in_doc), "` by `", stringify!($fpdec_type), "::<5>`.")]
        ///
        /// If you really want to add a value with different precision, convert it by
        #[doc = concat!("[`", stringify!($fpdec_type), "::rescale`] first.")]
        ///
        /// # Examples
        /// 
        /// ```
        #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
        #[doc = concat!("type Decimal = ", stringify!($type_in_doc), ";")]
        ///
        /// let left = Decimal::try_from(1.23).unwrap();
        /// let right = Decimal::try_from(0.45).unwrap();
        ///
        /// let res = Decimal::try_from(1.68).unwrap();
        /// assert_eq!(left.checked_add(right), Some(res));
        /// ```
        pub const fn checked_add(self, rhs: Self) -> Option<Self> {
            Self::from_opt_inner(self.inner.checked_add(rhs.inner))
        }

        /// Checked subtraction. Computes `self - rhs`, returning `None` if overflow occurred.
        ///
        /// The right operand must have the same precision with self. So you can not subtract
        #[doc = concat!("`", stringify!($type_in_doc), "` by `", stringify!($fpdec_type), "::<5>`.")]
        ///
        /// If you really want to subtract a value with different precision, convert it by
        #[doc = concat!("[`", stringify!($fpdec_type), "::rescale`] first.")]
        ///
        /// # Examples
        /// 
        /// ```
        #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
        #[doc = concat!("type Decimal = ", stringify!($type_in_doc), ";")]
        ///
        /// let left = Decimal::try_from(1.68).unwrap();
        /// let right = Decimal::try_from(1.23).unwrap();
        ///
        /// let res = Decimal::try_from(0.45).unwrap();
        /// assert_eq!(left.checked_sub(right), Some(res));
        /// ```
        pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
            Self::from_opt_inner(self.inner.checked_sub(rhs.inner))
        }

        /// Checked multiplication with integer. Computes `self * n`, returning
        /// `None` if overflow occurred.
        ///
        /// # Examples
        /// 
        /// ```
        #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
        #[doc = concat!("type Decimal = ", stringify!($type_in_doc), ";")]
        ///
        /// let dec = Decimal::try_from(0.123).unwrap();
        /// let res = Decimal::try_from(1.23).unwrap();
        /// assert_eq!(dec.checked_mul_int(10), Some(res));
        /// ```
        pub const fn checked_mul_int(self, n: $inner_type) -> Option<Self> {
            Self::from_opt_inner(self.inner.checked_mul(n))
        }

        /// Checked division with integer. Equivalent to
        #[doc = concat!("[`", stringify!($fpdec_type), "::checked_div_int_with_rounding`] with `rounding=Rounding::Round`.")]
        ///
        /// # Examples
        /// 
        /// ```
        #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
        #[doc = concat!("type Decimal = ", stringify!($type_in_doc), ";")]
        ///
        /// let dec = Decimal::try_from(1.23).unwrap();
        /// let res = Decimal::try_from(0.123).unwrap();
        /// assert_eq!(dec.checked_div_int(10), Some(res));
        /// assert_eq!(dec.checked_div_int(0), None);
        /// ```
        pub const fn checked_div_int(self, n: $inner_type) -> Option<Self> {
            self.checked_div_int_with_rounding(n, Rounding::Round)
        }

        /// Checked division with integer. Computes `self / n`, returning
        /// `None` if `n == 0` or precison loss with Rounding::Unexpected specified.
        ///
        /// # Examples
        /// 
        /// ```
        #[doc = concat!("use primitive_fixed_point_decimal::{", stringify!($fpdec_type), ", Rounding};")]
        #[doc = concat!("type Decimal = ", stringify!($type_in_doc), ";")]
        ///
        /// let dec = Decimal::try_from(0.2).unwrap();
        /// let res1 = Decimal::try_from(0.0666).unwrap();
        /// let res2 = Decimal::try_from(0.0667).unwrap();
        /// assert_eq!(dec.checked_div_int_with_rounding(3, Rounding::Floor), Some(res1));
        /// assert_eq!(dec.checked_div_int_with_rounding(3, Rounding::Ceil), Some(res2));
        /// assert_eq!(dec.checked_div_int_with_rounding(3, Rounding::Unexpected), None);
        /// ```
        pub const fn checked_div_int_with_rounding(
            self,
            n: $inner_type,
            rounding: Rounding
        ) -> Option<Self> {
            Self::from_opt_inner(rounding_div!(self.inner, n, rounding))
        }

        /// Return if negative.
        pub const fn is_neg(&self) -> bool {
            self.inner < 0
        }

        /// Return if positive.
        pub const fn is_pos(&self) -> bool {
            self.inner > 0
        }

        /// Return if zero.
        pub const fn is_zero(&self) -> bool {
            self.inner == 0
        }

        /// Shrink to a lower precision. Equivalent to
        #[doc = concat!("[`", stringify!($fpdec_type), "::shrink_to_with_rounding`] with `rounding=Rounding::Round`.")]
        pub const fn shrink_to(self, precision: i32) -> Self {
            match self.shrink_to_with_rounding(precision, Rounding::Round) {
                Some(d) => d,
                None => unreachable!(),
            }
        }

        const fn from_opt_inner(opt: Option<$inner_type>) -> Option<Self> {
            // because `const fn` does not support `Option::map()` or `?` by now
            if let Some(inner) = opt { Some(Self { inner }) } else { None }
        }

        /// Construct from inner directly. This API is low-level. Use it carefully.
        ///
        #[doc = concat!("Making a ", stringify!($fpdec_type), "&lt;P&gt; from `inner` gets value: inner<sup>-P</sup>.")]
        ///
        /// If you want to convert an integer to Fixdec *keeping* its value, use
        #[doc = concat!("[`", stringify!($fpdec_type), "::try_from`].")]
        ///
        /// # Examples:
        ///
        /// ```
        #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
        #[doc = concat!("type Decimal = ", stringify!($type_in_doc), ";")]
        /// assert_eq!(Decimal::from_inner(12345), Decimal::try_from(1.2345).unwrap());
        /// ```
        pub const fn from_inner(inner: $inner_type) -> Self {
            Self { inner }
        }
    };
}

// export macros
pub(crate) use define_common;
