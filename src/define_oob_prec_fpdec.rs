// define and implement OobPrecFpdecX type.
macro_rules! define_oob_prec_fpdec {
    (
        $fpdec_type:ident,
        $inner_type:ty,
        $digits:expr,

        // These are used only in doc comments.
        $bits:literal,
        $bits_minus_one:literal
    ) => {
        #[doc = concat!("A ", $bits, "-bits out-of-band-precision fixed-point decimal type, ")]
        #[doc = concat!("with about ", $digits, " significant digits.")]
        ///
        /// See [the module-level documentation](super) for more information.
        #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
        pub struct $fpdec_type {
            inner: $inner_type,
        }

        impl $fpdec_type {

            crate::define_common::define_common!($fpdec_type, $inner_type, $bits_minus_one);

            /// Checked multiplication. Equivalent to
            #[doc = concat!("[`", stringify!($fpdec_type), "::checked_mul_with_rounding`] with `rounding=Rounding::Round`.")]
            pub const fn checked_mul(self, rhs: Self, diff_precision: i32) -> Option<Self> {
                self.checked_mul_with_rounding(rhs, diff_precision, Rounding::Round)
            }

            /// Checked multiplication with rounding. Computes `self * rhs`, returning `None` if overflow
            /// occurred, or precision loss with `Rounding::Unexpected` specified.
            ///
            /// Equivalent to
            #[doc = concat!("[`", stringify!($fpdec_type), "::checked_mul_ext2`]")]
            /// but ignore cumulative error.
            pub const fn checked_mul_with_rounding(
                self,
                rhs: Self,
                diff_precision: i32, // P(self) + P(rhs) - P(result)
                rounding: Rounding
            ) -> Option<Self> {
                let mut cum_error = 0;
                self.checked_mul_ext2(rhs, diff_precision, rounding, &mut cum_error)
            }

            /// Computes `self * rhs`, returning `None` if overflow occurred or precison loss with
            /// `Rounding::Unexpected` specified.
            ///
            /// This is the for all other `checked_mul_*` methods.
            ///
            /// See the *cumulative error* section in the [module-level documentation](super)
            /// for more information abount cumulative error.
            ///
            /// The right operand and the result both could have different precisions
            /// against Self. And the `diff_precision` argument specifies the difference:
            ///     `diff_precision = precision(self) + precision(rhs) - precision(result)`
            ///
            /// # Examples
            /// 
            /// ```
            #[doc = concat!("use primitive_fixed_point_decimal::{", stringify!($fpdec_type), ", Rounding};")]
            #[doc = concat!("type Balance = ", stringify!($fpdec_type), ";")]
            #[doc = concat!("type FeeRate = ", stringify!($fpdec_type), ";")]
            ///
            /// let balance = Balance::try_from_i16(2, 4).unwrap();  // precision: 4
            /// let rate = FeeRate::try_from_f32(0.015, 3).unwrap(); // precision: 3
            ///
            /// let fee = Balance::try_from_f32(0.03, 4).unwrap();   // precision: 4
            /// assert_eq!(balance.checked_mul_with_rounding(rate, 3, Rounding::Round), Some(fee));
            /// ```
            pub const fn checked_mul_ext2(
                self,
                rhs: Self,
                diff_precision: i32, // P(self) + P(rhs) - P(result)
                rounding: Rounding,
                cum_error: &mut $inner_type,
            ) -> Option<Self> {
                let opt_inner = checked_mul_ext2(self.inner, rhs.inner, diff_precision, rounding, cum_error);
                Self::from_opt_inner(opt_inner)
            }

            /// Checked division. Equivalent to
            #[doc = concat!("[`", stringify!($fpdec_type), "::checked_div_with_rounding`] with `rounding=Rounding::Round`.")]
            pub const fn checked_div(self, rhs: Self, diff_precision: i32) -> Option<Self> {
                self.checked_div_with_rounding(rhs, diff_precision, Rounding::Round)
            }

            /// Checked division. Computes `self / rhs`, returning `None` if `rhs == 0` or
            /// the division results in overflow, or precision loss with Rounding::Unexpected specified.
            ///
            /// The right operand and the result both could have different precisions
            /// against Self. And the `diff_precision` argument specifies the difference:
            ///     `diff_precision = precision(self) - precision(rhs) - precision(result)`
            ///
            /// # Examples
            /// 
            /// ```
            #[doc = concat!("use primitive_fixed_point_decimal::{", stringify!($fpdec_type), ", Rounding};")]
            #[doc = concat!("type Balance = ", stringify!($fpdec_type), ";")]
            #[doc = concat!("type FeeRate = ", stringify!($fpdec_type), ";")]
            ///
            /// let balance = Balance::try_from_i16(2, 4).unwrap();  // precision: 4
            /// let fee = Balance::try_from_f32(0.03, 4).unwrap();   // precision: 4
            /// let rate = FeeRate::try_from_f32(0.015, 3).unwrap(); // precision: 3
            ///
            /// assert_eq!(fee.checked_div_with_rounding(balance, -3, Rounding::Round), Some(rate));
            /// assert_eq!(fee.checked_div_with_rounding(rate, -3, Rounding::Round), Some(balance));
            /// ```
            pub const fn checked_div_with_rounding(
                self,
                rhs: Self,
                diff_precision: i32, // P(self) - P(rhs) - P(result)
                rounding: Rounding
            ) -> Option<Self> {
                let mut cum_error = 0;
                self.checked_div_ext2(rhs, diff_precision, rounding, &mut cum_error)
            }

            /// 
            pub const fn checked_div_ext2(
                self,
                rhs: Self,
                diff_precision: i32, // P(self) - P(rhs) - P(result)
                rounding: Rounding,
                cum_error: &mut $inner_type,
            ) -> Option<Self> {
                let opt_inner = checked_div_ext2(self.inner, rhs.inner, diff_precision, rounding, cum_error);
                Self::from_opt_inner(opt_inner)
            }

            /// Rescale to another precision representation.
            ///
            /// Fail if overflow occurred when to bigger precision, or losing significant
            /// digits when to smaller precision.
            /// And the `diff_precision` argument specifies the difference:
            ///     `diff_precision = precision(self) - precision(result)`
            ///
            /// # Examples:
            ///
            /// ```
            #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
            #[doc = concat!("type Dec2 = ", stringify!($fpdec_type), ";")]
            #[doc = concat!("type Dec4 = ", stringify!($fpdec_type), ";")] // same type actually
            /// let d2 = Dec2::try_from_f32(1.23, 2).unwrap();
            /// let d4 = Dec4::try_from_f32(1.23, 4).unwrap();
            /// assert_eq!(d4.rescale(2).unwrap(), d2);
            /// assert_eq!(d2.rescale(-2).unwrap(), d4);
            pub const fn rescale(self, diff_precision: i32) -> Option<Self> {
                self.rescale_with_rounding(diff_precision, Rounding::Round)
            }

            pub const fn rescale_with_rounding(self, diff_precision: i32, rounding: Rounding) -> Option<Self> {
                let opt_inner = rescale_with_rounding(self.inner, diff_precision, rounding);
                Self::from_opt_inner(opt_inner)
            }

            /// Shrink to a lower precision. Fail if lossing significant precision
            /// with `rounding=Rounding::Unexpected`.
            ///
            /// And the `diff_precision` argument specifies the difference:
            ///     `diff_precision = precision(self) - precision(result)`
            ///
            /// Negative precision argument means integer part.
            ///
            /// # Examples:
            ///
            /// ```
            #[doc = concat!("use primitive_fixed_point_decimal::{", stringify!($fpdec_type), ", Rounding};")]
            #[doc = concat!("type Decimal = ", stringify!($fpdec_type), ";")]
            /// let d = Decimal::try_from_f32(1.2378, 4).unwrap();
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Floor).unwrap(), Decimal::try_from_f32(1.23, 4).unwrap());
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Ceil).unwrap(), Decimal::try_from_f32(1.24, 4).unwrap());
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Round).unwrap(), Decimal::try_from_f32(1.24, 4).unwrap());
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Unexpected), None);
            /// ```
            pub const fn shrink_to_with_rounding(self, diff_precision: i32, rounding: Rounding)
                -> Option<Self>
            {
                let opt_inner = shrink_to_with_rounding(self.inner, diff_precision, rounding);
                Self::from_opt_inner(opt_inner)
            }

            /// Read decimal from string.
            ///
            /// This method has 2 limitations:
            /// 1. Support decimal format only but not scientific notation;
            /// 2. Return `ParseError::Precision` if the string has more precision than `P`.
            ///
            /// If you want to skip these limitations, you can parse the string
            /// to float number first and then convert the number to this decimal.
            ///
            /// # Examples:
            ///
            /// ```
            /// use primitive_fixed_point_decimal::ParseError;
            #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
            #[doc = concat!("type Decimal = ", stringify!($fpdec_type), ";")]
            /// assert_eq!(Decimal::try_from_str("100.00", 2), Decimal::try_from_i16(100_i16, 2));
            /// assert_eq!(Decimal::try_from_str("123.45", 2), Decimal::try_from_f32(123.45_f32, 2));
            /// assert_eq!(Decimal::try_from_str("1e2", 2), Err(ParseError::Invalid));
            /// assert_eq!(Decimal::try_from_str("100.000", 2), Err(ParseError::Precision));
            /// assert_eq!(Decimal::try_from_str("99999999999999999999999999999999999999999999.00", 2), Err(ParseError::Overflow));
            /// ```
            pub fn try_from_str(s: &str, precision: i32) -> Result<Self, ParseError> {
                try_from_str(s, precision)
                    .map(|inner| Self::from_inner(inner))
            }
        }

        impl crate::oob_fmt::OobPrecDisplay for $fpdec_type {
            fn display_fmt(&self, precision: i32, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                display_fmt(self.inner, precision, f)
            }
        }

        macro_rules! convert_from_int {
            ($fn_name:ident, $from_int_type:ty) => {
                impl $fpdec_type {
                    #[doc = concat!("Try to convert ", stringify!($from_int_type), " into OobPrecFpdec.")]
                    /// Fail if overflow occurred.
                    ///
                    /// # Examples:
                    ///
                    /// ```
                    /// use std::str::FromStr;
                    #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
                    #[doc = concat!("type Decimal = ", stringify!($fpdec_type), ";")]
                    #[doc = concat!("assert_eq!(Decimal::", stringify!($fn_name), "(100_", stringify!($from_int_type), ", 2).unwrap(), Decimal::try_from_str(\"100\", 2).unwrap());")]
                    /// ```
                    pub fn $fn_name(i: $from_int_type, precision: i32) -> Result<Self, ParseError> {
                        let i2: $inner_type = i as $inner_type;
                        if i2 as $from_int_type != i || (i2 > 0) != (i > 0){
                            return Err(ParseError::Overflow);
                        }
                        let inner = check_from_int(i2, precision).ok_or(ParseError::Overflow)?;
                        Ok(Self::from_inner(inner))
                    }
                }
            }
        }
        convert_from_int!(try_from_i8, i8);
        convert_from_int!(try_from_u8, u8);
        convert_from_int!(try_from_i16, i16);
        convert_from_int!(try_from_u16, u16);
        convert_from_int!(try_from_i32, i32);
        convert_from_int!(try_from_u32, u32);
        convert_from_int!(try_from_i64, i64);
        convert_from_int!(try_from_u64, u64);
        convert_from_int!(try_from_i128, i128);
        convert_from_int!(try_from_u128, u128);

        macro_rules! convert_from_float {
            ($from_fn_name:ident, $into_fn_name:ident, $float_type:ty) => {
                impl $fpdec_type {
                    #[doc = concat!("Try to convert ", stringify!($float_type), " into OobPrecFpdec.")]
                    /// Fail if overflow occurred.
                    ///
                    /// # Examples:
                    ///
                    /// ```
                    /// use std::str::FromStr;
                    #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
                    #[doc = concat!("type Decimal = ", stringify!($fpdec_type), ";")]
                    #[doc = concat!("assert_eq!(Decimal::", stringify!($from_fn_name), "(123.456_", stringify!($float_type), ", 2).unwrap(), Decimal::try_from_str(\"123.46\", 2).unwrap());")]
                    /// ```
                    pub fn $from_fn_name(f: $float_type, precision: i32) -> Result<Self, ParseError> {
                        let base: $float_type = 10.0;
                        let inner_f = f * base.powi(precision) as $float_type;
                        if !inner_f.is_finite() {
                            return Err(ParseError::Overflow);
                        }
                        let inner_f = inner_f.round();
                        let inner_i = inner_f as $inner_type;
                        if (inner_i as $float_type != inner_f) {
                            return Err(ParseError::Overflow);
                        }
                        Ok(Self::from_inner(inner_i))
                    }

                    #[doc = concat!("Convert OobPrecFpdec into ", stringify!($float_type), ".")]
                    ///
                    /// # Examples:
                    ///
                    /// ```
                    #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
                    #[doc = concat!("type Decimal = ", stringify!($fpdec_type), ";")]
                    /// let dec = Decimal::try_from_f32(123.45, 2).unwrap();
                    #[doc = concat!("let f = dec.", stringify!($into_fn_name), "(2);")]
                    /// assert_eq!(f, 123.45);
                    /// ```
                    pub fn $into_fn_name(self, precision: i32) -> $float_type {
                        let base: $float_type = 10.0;
                        (self.inner as $float_type) / base.powi(precision)
                    }
                }
            }
        }
        convert_from_float!(try_from_f32, into_f32, f32);
        convert_from_float!(try_from_f64, into_f64, f64);

        impl fmt::Debug for $fpdec_type {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                write!(f, "Dec({})", self.inner)
            }
        }

        impl std::ops::Neg for $fpdec_type {
            type Output = Self;
            fn neg(self) -> Self::Output {
                Self { inner: -self.inner }
            }
        }

        impl std::ops::Add for $fpdec_type {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self { inner: self.inner + rhs.inner }
            }
        }

        impl std::ops::Sub for $fpdec_type {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self { inner: self.inner - rhs.inner }
            }
        }

        impl std::ops::AddAssign for $fpdec_type {
            fn add_assign(&mut self, rhs: Self) {
                self.inner += rhs.inner;
            }
        }

        impl std::ops::SubAssign for $fpdec_type {
            fn sub_assign(&mut self, rhs: Self) {
                self.inner -= rhs.inner;
            }
        }
    };
}

macro_rules! define_oob_mul_static {
    ($oob_type:ident, $static_type:ident, $inner_type:ty) => {
        impl $oob_type {
            /// Checked multiplication with . Equivalent to
            #[doc = concat!("Checked multiplication with `", stringify!($static_type), "`. Equivalent to")]
            #[doc = concat!("[`", stringify!($oob_type), "::checked_mul_static_with_rounding`] with `rounding=Rounding::Round`.")]
            pub const fn checked_mul_static<const Q: i32>(self, rhs: $static_type<Q>) -> Option<Self> {
                self.checked_mul_static_with_rounding(rhs, Rounding::Round)
            }

            #[doc = concat!("Checked multiplication with `", stringify!($static_type), "`.")]
            ///
            /// The result value inherits the same precision from self.
            pub const fn checked_mul_static_with_rounding<const Q: i32>(
                self,
                rhs: $static_type<Q>,
                rounding: Rounding,
            ) -> Option<Self> {
                self.checked_mul_with_rounding(Self::from_inner(rhs.inner), Q, rounding)
            }

            #[doc = concat!("Checked multiplication with `", stringify!($static_type), "`.")]
            ///
            /// The result value inherits the same precision from self.
            pub const fn checked_mul_static_ext2<const Q: i32>(
                self,
                rhs: $static_type<Q>,
                rounding: Rounding,
                cum_error: &mut $inner_type,
            ) -> Option<Self> {
                self.checked_mul_ext2(Self::from_inner(rhs.inner), Q, rounding, cum_error)
            }

            //-----
            //
            pub const fn checked_div_static<const Q: i32>(self, rhs: $static_type<Q>) -> Option<Self> {
                self.checked_div_static_with_rounding(rhs, Rounding::Round)
            }

            pub const fn checked_div_static_with_rounding<const Q: i32>(
                self,
                rhs: $static_type<Q>,
                rounding: Rounding,
            ) -> Option<Self> {
                self.checked_div_with_rounding(Self::from_inner(rhs.inner), -Q, rounding)
            }

            pub const fn checked_div_static_ext2<const Q: i32>(
                self,
                rhs: $static_type<Q>,
                rounding: Rounding,
                cum_error: &mut $inner_type,
            ) -> Option<Self> {
                self.checked_div_ext2(Self::from_inner(rhs.inner), -Q, rounding, cum_error)
            }
        }
    }
}


pub(crate) use define_oob_prec_fpdec;
pub(crate) use define_oob_mul_static;
