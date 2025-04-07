// define and implement StaticPrecFpdecX type.
macro_rules! define_static_prec_fpdec {
    (
        $fpdec_type:ident,
        $inner_type:ty,
        $digits:expr,

        // These are used only in doc comments.
        $bits:literal,
        $bits_minus_one:literal
    ) => {
        #[doc = concat!("A ", $bits, "-bits static-precision fixed-point decimal type, ")]
        #[doc = concat!("with about ", $digits, " significant digits.")]
        ///
        /// See [the module-level documentation](super) for more information.
        #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
        pub struct $fpdec_type<const P: i32> {
            inner: $inner_type,
        }

        impl<const P: i32> $fpdec_type<P> {

            crate::define_common::define_common!($fpdec_type, $inner_type, $bits_minus_one);

            /// Checked multiplication. Equivalent to
            #[doc = concat!("[`", stringify!($fpdec_type), "::checked_mul_with_rounding`] with `rounding=Rounding::Round`.")]
            pub const fn checked_mul<const Q: i32, const R: i32>(self, rhs: $fpdec_type<Q>) -> Option<$fpdec_type<R>> {
                self.checked_mul_with_rounding(rhs, Rounding::Round)
            }

            /// Checked multiplication. Computes `self * rhs`, returning `None` if overflow
            /// occurred, or precision loss with Rounding::Unexpected specified.
            ///
            /// The right operand and the result both could have different precisions
            /// against Self. So you can multiple
            #[doc = concat!("`", stringify!($fpdec_type), "::<4>` by `", stringify!($fpdec_type), "::<3>` ")]
            #[doc = concat!("and get a `", stringify!($fpdec_type), "::<2>`.")]
            ///
            /// # Examples
            /// 
            /// ```
            #[doc = concat!("use primitive_fixed_point_decimal::{", stringify!($fpdec_type), ", Rounding};")]
            #[doc = concat!("type Balance = ", stringify!($fpdec_type), "::<4>;")]
            #[doc = concat!("type FeeRate = ", stringify!($fpdec_type), "::<3>; // different precision")]
            ///
            /// let balance = Balance::try_from(2).unwrap();
            /// let rate = FeeRate::try_from(0.015).unwrap();
            ///
            /// let fee = Balance::try_from(0.03).unwrap();
            /// assert_eq!(balance.checked_mul_with_rounding(rate, Rounding::Round), Some(fee));
            /// ```
            pub const fn checked_mul_with_rounding<const Q: i32, const R: i32>(
                self,
                rhs: $fpdec_type<Q>,
                rounding: Rounding
            ) -> Option<$fpdec_type<R>> {
                let mut cum_error = 0;
                self.checked_mul_ext2(rhs, rounding, &mut cum_error)
            }

            /// Checked multiplication. Computes `self * rhs`, returning `None` if overflow
            /// occurred, or precision loss with Rounding::Unexpected specified.
            pub const fn checked_mul_ext2<const Q: i32, const R: i32>(
                self,
                rhs: $fpdec_type<Q>,
                rounding: Rounding,
                cum_error: &mut $inner_type,
            ) -> Option<$fpdec_type<R>> {
                let opt_inner = checked_mul_ext2(self.inner, rhs.inner, P + Q - R, rounding, cum_error);
                $fpdec_type::<R>::from_opt_inner(opt_inner)
            }

            /// Checked division. Equivalent to
            #[doc = concat!("[`", stringify!($fpdec_type), "::checked_div_with_rounding`] with `rounding=Rounding::Round`.")]
            pub const fn checked_div<const Q: i32, const R: i32>(self, rhs: $fpdec_type<Q>) -> Option<$fpdec_type<R>> {
                self.checked_div_with_rounding(rhs, Rounding::Round)
            }

            /// Checked division. Computes `self / rhs`, returning `None` if `rhs == 0` or
            /// the division results in overflow, or precision loss with Rounding::Unexpected specified.
            ///
            /// The right operand and the result both could have different precisions
            /// against Self. So you can divide
            #[doc = concat!("`", stringify!($fpdec_type), "::<4>` by `", stringify!($fpdec_type), "::<3>` ")]
            #[doc = concat!("and get a `", stringify!($fpdec_type), "::<2>`.")]
            ///
            /// # Examples
            /// 
            /// ```
            #[doc = concat!("use primitive_fixed_point_decimal::{", stringify!($fpdec_type), ", Rounding};")]
            #[doc = concat!("type Balance = ", stringify!($fpdec_type), "::<4>;")]
            #[doc = concat!("type FeeRate = ", stringify!($fpdec_type), "::<3>; // different precision")]
            ///
            /// let balance = Balance::try_from(2).unwrap();
            /// let fee = Balance::try_from(0.03).unwrap();
            /// let rate = FeeRate::try_from(0.015).unwrap();
            ///
            /// assert_eq!(fee.checked_div_with_rounding(balance, Rounding::Round), Some(rate));
            /// assert_eq!(fee.checked_div_with_rounding(rate, Rounding::Round), Some(balance));
            /// ```
            pub const fn checked_div_with_rounding<const Q: i32, const R: i32>(
                self,
                rhs: $fpdec_type<Q>,
                rounding: Rounding
            ) -> Option<$fpdec_type<R>> {
                let mut cum_error = 0;
                self.checked_div_ext2(rhs, rounding, &mut cum_error)
            }

            pub const fn checked_div_ext2<const Q: i32, const R: i32>(
                self,
                rhs: $fpdec_type<Q>,
                rounding: Rounding,
                cum_error: &mut $inner_type,
            ) -> Option<$fpdec_type<R>> {
                let opt_inner = checked_div_ext2(self.inner, rhs.inner, P - Q - R, rounding, cum_error);
                $fpdec_type::<R>::from_opt_inner(opt_inner)
            }

            /// Shrink to a lower precision. Equivalent to
            #[doc = concat!("[`", stringify!($fpdec_type), "::shrink_to_with_rounding`] with `rounding=Rounding::Round`.")]
            pub const fn shrink_to(self, precision: i32) -> Self {
                self.shrink_to_with_rounding(precision, Rounding::Round)
            }

            /// Shrink to a lower precision. Fail if lossing significant precision
            /// with `rounding=Rounding::Unexpected`.
            ///
            /// Negative precision argument means integer part.
            ///
            /// # Examples:
            ///
            /// ```
            #[doc = concat!("use primitive_fixed_point_decimal::{", stringify!($fpdec_type), ", Rounding};")]
            #[doc = concat!("type Decimal = ", stringify!($fpdec_type), "::<4>;")]
            /// let d = Decimal::try_from(1.2378).unwrap();
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Floor).unwrap(), Decimal::try_from(1.23).unwrap());
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Ceil).unwrap(), Decimal::try_from(1.24).unwrap());
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Round).unwrap(), Decimal::try_from(1.24).unwrap());
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Unexpected), None);
            ///
            /// // negative precision argument
            #[doc = concat!("type Decimal1 = ", stringify!($fpdec_type), "::<1>;")]
            /// let d = Decimal1::try_from(1234.5).unwrap();
            /// assert_eq!(d.shrink_to_with_rounding(-2, Rounding::Round).unwrap(), Decimal1::try_from(1200).unwrap());
            /// ```
            pub const fn shrink_to_with_rounding(self, precision: i32, rounding: Rounding) -> Self {
                let inner = shrink_with_rounding(self.inner, P - precision, rounding);
                Self::from_inner(inner)
            }
        }

        impl<const P: i32> fmt::Debug for $fpdec_type<P> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                write!(f, "Dec({},{})", self.inner, P)
            }
        }

        /// Format the decimal.
        ///
        /// # Examples:
        ///
        /// ```
        /// use std::str::FromStr;
        #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
        #[doc = concat!("type Decimal = ", stringify!($fpdec_type), "::<4>;")]
        /// let fd = Decimal::from_str("1.5670").unwrap();
        /// assert_eq!(&format!("{}", fd), "1.567"); // omit tailing zeros
        /// ```
        impl<const P: i32> fmt::Display for $fpdec_type<P> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                display_fmt(self.inner, P, f)
            }
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
        /// use std::str::FromStr;
        /// use primitive_fixed_point_decimal::ParseError;
        #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
        #[doc = concat!("type Decimal = ", stringify!($fpdec_type), "::<2>;")]
        /// assert_eq!(Decimal::from_str("100.00"), Decimal::try_from(100_i16));
        /// assert_eq!(Decimal::from_str("123.45"), Decimal::try_from(123.45_f32));
        /// assert_eq!(Decimal::from_str("1e2"), Err(ParseError::Invalid));
        /// assert_eq!(Decimal::from_str("100.000"), Err(ParseError::Precision));
        /// assert_eq!(Decimal::from_str("99999999999999999999999999999999999999999999.00"), Err(ParseError::Overflow));
        /// ```
        impl<const P: i32> std::str::FromStr for $fpdec_type<P> {
            type Err = ParseError;
            fn from_str(s: &str) -> Result<Self, ParseError> {
                try_from_str(s, P)
                    .map(|inner| Self::from_inner(inner))
            }
        }

        macro_rules! convert_static_from_int {
            ($from_int_type:ty) => {
                impl<const P: i32> TryFrom<$from_int_type> for $fpdec_type<P> {
                    type Error = ParseError;

                    #[doc = concat!("Try to convert ", stringify!($from_int_type), " into StaticPrecFpdec.")]
                    /// Fail if overflow occurred.
                    ///
                    /// # Examples:
                    ///
                    /// ```
                    /// use std::str::FromStr;
                    #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
                    #[doc = concat!("type Decimal = ", stringify!($fpdec_type), "::<2>;")]
                    #[doc = concat!("assert_eq!(Decimal::try_from(100_", stringify!($from_int_type), ").unwrap(), Decimal::from_str(\"100\").unwrap());")]
                    /// ```
                    fn try_from(i: $from_int_type) -> Result<Self, Self::Error> {
                        let i2: $inner_type = i as $inner_type;
                        if i2 as $from_int_type != i || (i2 > 0) != (i > 0){
                            return Err(ParseError::Overflow);
                        }
                        let inner = check_from_int(i2, P).ok_or(ParseError::Overflow)?;
                        Ok(Self::from_inner(inner))
                    }
                }
            }
        }
        convert_static_from_int!(i8);
        convert_static_from_int!(u8);
        convert_static_from_int!(i16);
        convert_static_from_int!(u16);
        convert_static_from_int!(i32);
        convert_static_from_int!(u32);
        convert_static_from_int!(i64);
        convert_static_from_int!(u64);
        convert_static_from_int!(i128);
        convert_static_from_int!(u128);

        macro_rules! convert_static_from_float {
            ($float_type:ty) => {
                impl<const P: i32> TryFrom<$float_type> for $fpdec_type<P> {
                    type Error = ParseError;

                    #[doc = concat!("Try to convert ", stringify!($float_type), " into StaticPrecFpdec.")]
                    /// Fail if overflow occurred.
                    ///
                    /// # Examples:
                    ///
                    /// ```
                    /// use std::str::FromStr;
                    #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
                    #[doc = concat!("type Decimal = ", stringify!($fpdec_type), "::<2>;")]
                    #[doc = concat!("assert_eq!(Decimal::try_from(123.456_", stringify!($float_type), ").unwrap(), Decimal::from_str(\"123.46\").unwrap());")]
                    /// ```
                    fn try_from(f: $float_type) -> Result<Self, Self::Error> {
                        let base: $float_type = 10.0;
                        let inner_f = f * base.powi(P) as $float_type;
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
                }

                impl<const P: i32> From<$fpdec_type<P>> for $float_type {
                    #[doc = concat!("Convert StaticPrecFpdec into ", stringify!($float_type), ".")]
                    ///
                    /// # Examples:
                    ///
                    /// ```
                    #[doc = concat!("use primitive_fixed_point_decimal::", stringify!($fpdec_type), ";")]
                    #[doc = concat!("type Decimal = ", stringify!($fpdec_type), "::<2>;")]
                    #[doc = concat!("let f: ", stringify!($float_type), " = Decimal::try_from(123.45).unwrap().into();")]
                    /// assert_eq!(f, 123.45);
                    /// ```
                    fn from(dec: $fpdec_type<P>) -> Self {
                        let base: $float_type = 10.0;
                        (dec.inner as $float_type) / base.powi(P)
                    }
                }
            }
        }

        convert_static_from_float!(f32);
        convert_static_from_float!(f64);

        impl<const P: i32> std::ops::Neg for $fpdec_type<P> {
            type Output = Self;
            fn neg(self) -> Self::Output {
                Self { inner: -self.inner }
            }
        }

        impl<const P: i32> std::ops::Add for $fpdec_type<P> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self { inner: self.inner + rhs.inner }
            }
        }

        impl<const P: i32> std::ops::Sub for $fpdec_type<P> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self { inner: self.inner - rhs.inner }
            }
        }

        impl<const P: i32> std::ops::AddAssign for $fpdec_type<P> {
            fn add_assign(&mut self, rhs: Self) {
                self.inner += rhs.inner;
            }
        }

        impl<const P: i32> std::ops::SubAssign for $fpdec_type<P> {
            fn sub_assign(&mut self, rhs: Self) {
                self.inner -= rhs.inner;
            }
        }


        #[cfg(feature="serde")]
        use serde::{Serialize, Deserialize, Serializer, Deserializer};

        #[cfg(feature="serde")]
        impl<const P: i32> Serialize for $fpdec_type<P> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
            {
                // XXX how to selete dump type?
                if serializer.is_human_readable() {
                    serializer.collect_str(self)
                } else {
                    Into::<f64>::into(*self).serialize(serializer)
                }
            }
        }

        #[cfg(feature="serde")]
        impl<'de, const P: i32> Deserialize<'de> for $fpdec_type<P> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>
            {
                use serde::de::{self, Visitor};
                use std::str::FromStr;

                struct StaticPrecFpdecVistor<const P: i32>;

                macro_rules! visit_num {
                    ($func_name:ident, $num_type:ty) => {
                        fn $func_name<E: de::Error>(self, n: $num_type) -> Result<Self::Value, E> {
                            $fpdec_type::<P>::try_from(n)
                                .map_err(|_| E::custom("decimal overflow"))
                        }
                    }
                }

                impl<'de, const P: i32> Visitor<'de> for StaticPrecFpdecVistor<P> {
                    type Value = $fpdec_type<P>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        write!(formatter, "decimal")
                    }

                    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                        $fpdec_type::<P>::from_str(s)
                            .map_err(|e| E::custom(format!("decimal {:?}", e)))
                    }

                    visit_num!(visit_f32, f32);
                    visit_num!(visit_f64, f64);
                    visit_num!(visit_i8, i8);
                    visit_num!(visit_u8, u8);
                    visit_num!(visit_i16, i16);
                    visit_num!(visit_u16, u16);
                    visit_num!(visit_i32, i32);
                    visit_num!(visit_u32, u32);
                    visit_num!(visit_i64, i64);
                    visit_num!(visit_u64, u64);
                    visit_num!(visit_i128, i128);
                    visit_num!(visit_u128, u128);
                }

                deserializer.deserialize_any(StaticPrecFpdecVistor)
            }
        }
    };
}

pub(crate) use define_static_prec_fpdec;
