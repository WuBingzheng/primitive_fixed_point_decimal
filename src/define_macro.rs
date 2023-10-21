// define and implement FixDecX type.
//
// ALL_EXPS, calc_mul_div and calc_div_div are needed to use this macro.
macro_rules! define_fixdec {
    (
        $fixdec_type:ident,
        $inner_type:ty,
        $digits:expr,

        // These are used only in doc comments.
        $bits:literal,
        $bits_minus_one:literal
    ) => {
        use std::fmt;
        use std::str::FromStr;
        use std::ops::{Neg, Add, Sub, AddAssign, SubAssign};
        use super::{ParseError, Rounding};

        #[doc = concat!("Approximate number of significant decimal digits of FixDec", $bits, ".")]
        /// This is also the precision limit.
        pub const DIGITS: u32 = $digits;

        #[doc = concat!("A ", $bits, "-bits primitive fixed-point decimal type, ")]
        #[doc = concat!("with about ", $digits, " significant digits.")]
        ///
        /// See [the module-level documentation](super) for more information.
        #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
        pub struct $fixdec_type<const P: u32> {
            inner: $inner_type,
        }

        impl<const P: u32> $fixdec_type<P> {
            const EXP: $inner_type = <$inner_type>::pow(10, P);

            /// The zero value, 0.
            pub const ZERO: Self = Self { inner: 0 };

            /// The one value, 1.
            pub const ONE: Self = Self { inner: Self::EXP };

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
            #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
            #[doc = concat!("type Decimal = FixDec", $bits, "::<4>;")]
            ///
            /// assert_eq!(Decimal::ONE.abs(), Decimal::ONE);
            /// assert_eq!(Decimal::MAX.abs(), Decimal::MAX);
            /// assert_eq!((-Decimal::ONE).abs(), Decimal::ONE);
            /// assert_eq!((-Decimal::MAX).abs(), Decimal::MAX);
            /// assert_eq!(Decimal::ZERO.abs(), Decimal::ZERO);
            /// ```
            #[inline]
            pub const fn abs(self) -> Self {
                Self { inner: self.inner.abs() }
            }

            /// Checked absolute value. Computes `self.abs()`, returning `None` if `self == MIN`.
            /// 
            /// # Examples
            /// 
            /// ```
            #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
            #[doc = concat!("type Decimal = FixDec", $bits, "::<4>;")]
            ///
            /// assert_eq!((-Decimal::ONE).checked_abs(), Some(Decimal::ONE));
            /// assert_eq!(Decimal::MIN.checked_abs(), None);
            /// ```
            #[inline]
            pub const fn checked_abs(self) -> Option<Self> {
                Self::from_opt_inner(self.inner.checked_abs())
            }

            /// Checked addition. Computes `self + rhs`, returning `None` if overflow occurred.
            ///
            /// The right operand must have the same precision with self. So you can not add
            #[doc = concat!("`FixDec", $bits, "::<4>` by `FixDec", $bits, "::<5>`.")]
            ///
            /// If you really want to add a value with different precision, convert it by
            #[doc = concat!("[`FixDec", $bits, "::rescale`] first.")]
            ///
            /// # Examples
            /// 
            /// ```
            /// use std::str::FromStr;
            #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
            #[doc = concat!("type Decimal = FixDec", $bits, "::<4>;")]
            ///
            /// let left = Decimal::from_str("1.23").unwrap();
            /// let right = Decimal::from_str("0.45").unwrap();
            ///
            /// let res = Decimal::from_str("1.68").unwrap();
            /// assert_eq!(left.checked_add(right), Some(res));
            /// ```
            #[inline]
            pub const fn checked_add(self, rhs: Self) -> Option<Self> {
                Self::from_opt_inner(self.inner.checked_add(rhs.inner))
            }

            /// Checked subtraction. Computes `self - rhs`, returning `None` if overflow occurred.
            ///
            /// The right operand must have the same precision with self. So you can not subtract
            #[doc = concat!("`FixDec", $bits, "::<4>` by `FixDec", $bits, "::<5>`.")]
            ///
            /// If you really want to subtract a value with different precision, convert it by
            #[doc = concat!("[`FixDec", $bits, "::rescale`] first.")]
            ///
            /// # Examples
            /// 
            /// ```
            /// use std::str::FromStr;
            #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
            #[doc = concat!("type Decimal = FixDec", $bits, "::<4>;")]
            ///
            /// let left = Decimal::from_str("1.68").unwrap();
            /// let right = Decimal::from_str("1.23").unwrap();
            ///
            /// let res = Decimal::from_str("0.45").unwrap();
            /// assert_eq!(left.checked_sub(right), Some(res));
            /// ```
            #[inline]
            pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
                Self::from_opt_inner(self.inner.checked_sub(rhs.inner))
            }

            /// Checked multiplication. Equivalent to
            #[doc = concat!("[`FixDec", $bits, "::checked_mul_with_rounding`] with `rounding=Rounding::Round`.")]
            pub const fn checked_mul<const Q: u32, const R: u32>(self, rhs: $fixdec_type<Q>) -> Option<$fixdec_type<R>> {
                self.checked_mul_with_rounding(rhs, Rounding::Round)
            }

            /// Checked multiplication. Computes `self * rhs`, returning `None` if overflow
            /// occurred, or precison loss with Rounding::Unexpected specified.
            ///
            /// The right operand and the result both could have different precisions
            /// against Self. So you can multiple
            #[doc = concat!("`FixDec", $bits, "::<4>` by `FixDec", $bits, "::<3>` ")]
            #[doc = concat!("and get a `FixDec", $bits, "::<2>`.")]
            ///
            /// # Panics
            ///
            /// Panics if `P + Q - R > DIGITS`.
            ///
            /// # Examples
            /// 
            /// ```
            /// use std::str::FromStr;
            #[doc = concat!("use primitive_fixed_point_decimal::{FixDec", $bits, ", Rounding};")]
            #[doc = concat!("type Balance = FixDec", $bits, "::<4>;")]
            #[doc = concat!("type FeeRate = FixDec", $bits, "::<3>; // different precision")]
            ///
            /// let balance = Balance::from_str("2").unwrap();
            /// let rate = FeeRate::from_str("0.015").unwrap();
            ///
            /// let fee = Balance::from_str("0.03").unwrap();
            /// assert_eq!(balance.checked_mul_with_rounding(rate, Rounding::Round), Some(fee));
            /// ```
            pub const fn checked_mul_with_rounding<const Q: u32, const R: u32>(
                self,
                rhs: $fixdec_type<Q>,
                rounding: Rounding
            ) -> Option<$fixdec_type<R>> {

                let opt_inner = if P + Q > R {
                    // self.inner * rhs.inner / diff_exp
                    debug_assert!(P + Q - R <= DIGITS);
                    calc_mul_div(self.inner, rhs.inner, ALL_EXPS[(P + Q - R) as usize], rounding)
                } else {
                    // self.inner * rhs.inner * diff_exp
                    let Some(r) = self.inner.checked_mul(rhs.inner) else {
                        return None;
                    };
                    r.checked_mul(ALL_EXPS[(R - P - Q) as usize])
                };
                $fixdec_type::<R>::from_opt_inner(opt_inner)
            }

            /// Checked division. Equivalent to
            #[doc = concat!("[`FixDec", $bits, "::checked_div_with_rounding`] with `rounding=Rounding::Round`.")]
            pub const fn checked_div<const Q: u32, const R: u32>(self, rhs: $fixdec_type<Q>) -> Option<$fixdec_type<R>> {
                self.checked_div_with_rounding(rhs, Rounding::Round)
            }

            /// Checked division. Computes `self / rhs`, returning `None` if `rhs == 0` or
            /// the division results in overflow, or precison loss with Rounding::Unexpected specified.
            ///
            /// The right operand and the result both could have different precisions
            /// against Self. So you can divide
            #[doc = concat!("`FixDec", $bits, "::<4>` by `FixDec", $bits, "::<3>` ")]
            #[doc = concat!("and get a `FixDec", $bits, "::<2>`.")]
            ///
            /// # Panics
            ///
            /// Panics if `Q + R - P > DIGITS`.
            ///
            /// # Examples
            /// 
            /// ```
            /// use std::str::FromStr;
            #[doc = concat!("use primitive_fixed_point_decimal::{FixDec", $bits, ", Rounding};")]
            #[doc = concat!("type Balance = FixDec", $bits, "::<4>;")]
            #[doc = concat!("type FeeRate = FixDec", $bits, "::<3>; // different precision")]
            ///
            /// let balance = Balance::from_str("2").unwrap();
            /// let fee = Balance::from_str("0.03").unwrap();
            /// let rate = FeeRate::from_str("0.015").unwrap();
            ///
            /// assert_eq!(fee.checked_div_with_rounding(balance, Rounding::Round), Some(rate));
            /// assert_eq!(fee.checked_div_with_rounding(rate, Rounding::Round), Some(balance));
            /// ```
            pub const fn checked_div_with_rounding<const Q: u32, const R: u32>(
                self,
                rhs: $fixdec_type<Q>,
                rounding: Rounding
            ) -> Option<$fixdec_type<R>> {

                let opt_inner = if P < Q + R {
                    // self.inner * diff_exp / rhs.inner
                    debug_assert!(Q + R - P <= DIGITS);
                    calc_mul_div(self.inner, ALL_EXPS[(Q + R - P) as usize], rhs.inner, rounding)
                } else {
                    // self.inner / (diff_exp * rhs.inner)
                    calc_div_div(self.inner, ALL_EXPS[(P - Q - R) as usize], rhs.inner, rounding)
                };
                $fixdec_type::<R>::from_opt_inner(opt_inner)
            }

            /// Rescale to another precision representation.
            ///
            /// Fail if overflow occurred when to bigger precision, or losing significant
            /// digits when to smaller precision.
            ///
            /// # Examples:
            ///
            /// ```
            /// use std::str::FromStr;
            #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
            #[doc = concat!("type Dec2 = FixDec", $bits, "::<2>;")]
            #[doc = concat!("type Dec4 = FixDec", $bits, "::<4>;")]
            /// let d2 = Dec2::from_str("1.23").unwrap();
            /// let d4 = Dec4::from_str("1.23").unwrap();
            /// assert_eq!(d4.rescale::<2>().unwrap(), d2);
            /// assert_eq!(d2.rescale::<4>().unwrap(), d4);
            pub const fn rescale<const Q: u32>(self) -> Option<$fixdec_type<Q>> {
                if Q == P {
                    Some($fixdec_type::<Q>::from_inner(self.inner))
                } else if Q > P {
                    // to bigger precision
                    $fixdec_type::<Q>::from_opt_inner(self.inner.checked_mul(ALL_EXPS[(Q - P) as usize]))
                } else {
                    // to smaller precision
                    let exp = ALL_EXPS[(P - Q) as usize];
                    if self.inner % exp == 0 {
                        Some($fixdec_type::<Q>::from_inner(self.inner / exp))
                    } else {
                        None
                    }
                }
            }

            /// Return if negative.
            #[inline]
            pub const fn is_neg(&self) -> bool {
                self.inner < 0
            }

            /// Return if positive.
            #[inline]
            pub const fn is_pos(&self) -> bool {
                self.inner > 0
            }

            /// Return if zero.
            #[inline]
            pub const fn is_zero(&self) -> bool {
                self.inner == 0
            }

            /// Shrink to a lower precision. Equivalent to
            #[doc = concat!("[`FixDec", $bits, "::shrink_to_with_rounding`] with `rounding=Rounding::Round`.")]
            pub const fn shrink_to(self, precision: i32) -> Self {
                match self.shrink_to_with_rounding(precision, Rounding::Round) {
                    Some(d) => d,
                    None => unreachable!(),
                }
            }

            /// Shrink to a lower precision. Fail if lossing significant precision
            /// with `rounding=Rounding::Unexpected`.
            ///
            /// Negative precision argument means integer part.
            ///
            /// # Examples:
            ///
            /// ```
            /// use std::str::FromStr;
            #[doc = concat!("use primitive_fixed_point_decimal::{FixDec", $bits, ", Rounding};")]
            #[doc = concat!("type Decimal = FixDec", $bits, "::<4>;")]
            /// let d = Decimal::from_str("1.2378").unwrap();
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Floor).unwrap(), Decimal::from_str("1.23").unwrap());
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Ceil).unwrap(), Decimal::from_str("1.24").unwrap());
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Round).unwrap(), Decimal::from_str("1.24").unwrap());
            /// assert_eq!(d.shrink_to_with_rounding(2, Rounding::Unexpected), None);
            ///
            /// // negative precision argument
            #[doc = concat!("type Decimal1 = FixDec", $bits, "::<1>;")]
            /// let d = Decimal1::from_str("1234.5").unwrap();
            /// assert_eq!(d.shrink_to_with_rounding(-2, Rounding::Round).unwrap(), Decimal1::from_str("1200").unwrap());
            /// ```
            pub const fn shrink_to_with_rounding(self, precision: i32, rounding: Rounding)
                -> Option<Self>
            {
                let diff = P as i32 - precision;
                if diff <= 0 {
                    Some(self)
                } else if diff as u32 >= DIGITS {
                    if matches!(rounding, Rounding::Unexpected) && self.inner != 0 {
                        return None;
                    }
                    Some(Self::ZERO)
                } else {
                    let e = ALL_EXPS[diff as usize];
                    let inner = self.inner / e * e;
                    let remain = self.inner - inner;
                    let carry = match rounding {
                        Rounding::Floor => 0,
                        Rounding::Ceil => if remain == 0 { 0 } else { e },
                        Rounding::Round => if remain * 2 < e { 0 } else { e },
                        Rounding::Unexpected => if remain == 0 { 0 } else { return None },
                    };
                    Some(Self::from_inner(inner + carry))
                }
            }

            #[inline]
            const fn from_opt_inner(opt: Option<$inner_type>) -> Option<Self> {
                // because `const fn` does not support `Option::map()` or `?` by now
                if let Some(inner) = opt { Some(Self { inner }) } else { None }
            }

            /// Construct from inner directly. This API is low-level. Use it carefully.
            ///
            #[doc = concat!("Making a FixDec", $bits, "&lt;P&gt; from `inner` gets value: inner<sup>-P</sup>.")]
            ///
            /// If you want to convert an integer to Fixdec *keeping* its value, use
            #[doc = concat!("[`FixDec", $bits, "::try_from`].")]
            ///
            /// # Examples:
            ///
            /// ```
            /// use std::str::FromStr;
            #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
            #[doc = concat!("type Decimal = FixDec", $bits, "::<4>;")]
            /// assert_eq!(Decimal::from_inner(12345), Decimal::from_str("1.2345").unwrap());
            /// ```
            #[inline]
            pub const fn from_inner(inner: $inner_type) -> Self {
                debug_assert!(P <= DIGITS, "too big precision!");
                Self { inner }
            }

            /// Read decimal from string, with specified rounding kind.
            ///
            /// # Examples:
            ///
            /// ```
            /// use std::str::FromStr;
            /// use primitive_fixed_point_decimal::{Rounding, ParseError};
            #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
            #[doc = concat!("type Decimal = FixDec", $bits, "::<4>;")]
            ///
            /// fn check(origin: &str, rounding: Rounding, expect: &str) {
            ///     let fd = Decimal::from_str_with_rounding(origin, rounding).unwrap();
            ///     assert_eq!(fd, Decimal::from_str(expect).unwrap());
            /// }
            /// check("1.23456789", Rounding::Floor, "1.2345");
            /// check("1.23456789", Rounding::Ceil, "1.2346");
            /// check("1.23456789", Rounding::Round, "1.2346");
            /// check("1.23455000", Rounding::Round, "1.2346");
            /// check("1.23", Rounding::Round, "1.23");
            ///
            /// assert_eq!(Decimal::from_str_with_rounding("1.23789", Rounding::Unexpected),
            ///            Err(ParseError::Precision));
            /// ```
            pub fn from_str_with_rounding(s: &str, rounding: Rounding)
                -> Result<Self, ParseError> {
        
                // sign part
                let (s, is_neg) = match s.as_bytes().first() {
                    None => return Err(ParseError::Empty),
                    Some(b'-') => (&s[1..], true),
                    Some(b'+') => (&s[1..], false),
                    _ => (s, false),
                };
        
                if s.is_empty() {
                    return Err(ParseError::Empty);
                }
        
                let (int_str, frac_num) = if let Some((int_str, frac_str)) = s.split_once('.') {
                    // fraction part
                    let frac_len = frac_str.len();
                    let frac_num = if (P as usize) < frac_len {
                        let (keep, discard) = frac_str.split_at(P as usize);
                        parse_int(keep)? + parse_rounding(discard, rounding)? as $inner_type
                    } else {
                        parse_int(frac_str)? * ALL_EXPS[P as usize - frac_len]
                    };
        
                    (int_str, frac_num)
                } else {
                    (s, 0)
                };
        
                // integer part
                let int_num = parse_int(int_str)?;
        
                let Some(mut inner) = int_num.checked_mul(Self::EXP) else {
                    return Err(ParseError::Overflow);
                };
                inner += frac_num;
                if is_neg {
                    inner = -inner;
                }
                Ok(Self::from_inner(inner))
            }
        }

        fn parse_int(s: &str) -> Result<$inner_type, ParseError> {
            if s.is_empty() {
                Ok(0)
            } else { // TODO check '+' and '-'
                <$inner_type>::from_str(s).map_err(|_|ParseError::Invalid)
            }
        }

        impl<const P: u32> fmt::Debug for $fixdec_type<P> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                write!(f, "Dec({},{})", self.inner, P)
            }
        }

        /// Format the decimal.
        ///
        /// The tailing zeros of fraction are truncated by default, while the
        /// precision can be specified by `{:.N}`.
        ///
        /// # Examples:
        ///
        /// ```
        /// use std::str::FromStr;
        #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
        #[doc = concat!("type Decimal = FixDec", $bits, "::<4>;")]
        /// let fd = Decimal::from_str("1.5670").unwrap();
        /// assert_eq!(&format!("{}", fd), "1.567"); // omit tailing zeros
        /// assert_eq!(&format!("{:.2}", fd), "1.57"); // rounding
        /// ```
        impl<const P: u32> fmt::Display for $fixdec_type<P> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                let intg = self.inner / Self::EXP;
                let mut frac = self.inner % Self::EXP;
                if frac == 0 {
                    return write!(f, "{}", intg);
                }

                if intg == 0 && self.inner < 0 {
                    write!(f, "-0.")?;
                    frac = -frac;
                } else {
                    write!(f, "{}.", intg)?;
                    frac = frac.abs();
                }

                if let Some(precision) = f.precision() {
                    if precision < P as usize {
                        let exp = ALL_EXPS[P as usize - precision];
                        let rem = frac % exp;
                        frac = frac / exp + if rem * 2 >= exp { 1 } else { 0 };
                        write!(f, "{:0width$}", frac, width=precision)
                    } else {
                        write!(f, "{:0width$}", frac, width=P as usize)
                    }
                } else if P > 0 {
                    let mut ie = P as usize - 1;
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

        impl<const P: u32> FromStr for $fixdec_type<P> {
            type Err = ParseError;

            /// Read decimal from string.
            ///
            #[doc = concat!("Equivalent to [`FixDec", $bits, "::from_str_with_rounding`] ")]
            /// with `rounding=Rounding::Round`.
            fn from_str(s: &str) -> Result<Self, ParseError> {
                Self::from_str_with_rounding(s, Rounding::Round)
            }
        }

        macro_rules! convert_from_int {
            ($from_int_type:ty) => {
                impl<const P: u32> TryFrom<$from_int_type> for $fixdec_type<P> {
                    type Error = ();

                    #[doc = concat!("Try to convert ", stringify!($from_int_type), " into FixDec.")]
                    /// Fail if overflow occurred.
                    ///
                    /// # Examples:
                    ///
                    /// ```
                    /// use std::str::FromStr;
                    #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
                    #[doc = concat!("type Decimal = FixDec", $bits, "::<2>;")]
                    #[doc = concat!("assert_eq!(Decimal::try_from(100_", stringify!($from_int_type), ").unwrap(), Decimal::from_str(\"100\").unwrap());")]
                    /// ```
                    fn try_from(i: $from_int_type) -> Result<Self, Self::Error> {
                        let i2: $inner_type = i as $inner_type;
                        if i2 as $from_int_type != i || (i2 > 0) != (i > 0){
                            return Err(());
                        }
                        let inner = i2.checked_mul(ALL_EXPS[P as usize]).ok_or(())?;
                        Ok(Self::from_inner(inner))
                    }
                }
            }
        }
        convert_from_int!(i8);
        convert_from_int!(u8);
        convert_from_int!(i16);
        convert_from_int!(u16);
        convert_from_int!(i32);
        convert_from_int!(u32);
        convert_from_int!(i64);
        convert_from_int!(u64);
        convert_from_int!(i128);
        convert_from_int!(u128);

        macro_rules! convert_from_float {
            ($from_float_type:ty) => {
                impl<const P: u32> TryFrom<$from_float_type> for $fixdec_type<P> {
                    type Error = ();

                    #[doc = concat!("Try to convert ", stringify!($from_float_type), " into FixDec.")]
                    /// Fail if overflow occurred.
                    ///
                    /// # Examples:
                    ///
                    /// ```
                    /// use std::str::FromStr;
                    #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
                    #[doc = concat!("type Decimal = FixDec", $bits, "::<2>;")]
                    #[doc = concat!("assert_eq!(Decimal::try_from(123.456_", stringify!($from_float_type), ").unwrap(), Decimal::from_str(\"123.46\").unwrap());")]
                    /// ```
                    fn try_from(f: $from_float_type) -> Result<Self, Self::Error> {
                        let inner_f = f * Self::EXP as $from_float_type;
                        if !inner_f.is_finite() {
                            return Err(());
                        }
                        let inner_f = inner_f.round();
                        let inner_i = inner_f as $inner_type;
                        if (inner_i as $from_float_type != inner_f) {
                            return Err(());
                        }
                        Ok(Self::from_inner(inner_i))
                    }
                }

                impl<const P: u32> Into<$from_float_type> for $fixdec_type<P> {
                    #[doc = concat!("Convert FixDec into ", stringify!($from_float_type), ".")]
                    ///
                    /// # Examples:
                    ///
                    /// ```
                    #[doc = concat!("use primitive_fixed_point_decimal::FixDec", $bits, ";")]
                    #[doc = concat!("type Decimal = FixDec", $bits, "::<2>;")]
                    #[doc = concat!("let f: ", stringify!($from_float_type), " = Decimal::try_from(123.45).unwrap().into();")]
                    /// assert_eq!(f, 123.45);
                    /// ```
                    fn into(self) -> $from_float_type {
                        (self.inner as $from_float_type) / (Self::EXP as $from_float_type)
                    }
                }
            }
        }

        convert_from_float!(f32);
        convert_from_float!(f64);

        impl<const P: u32> Neg for $fixdec_type<P> {
            type Output = Self;
            fn neg(self) -> Self::Output {
                Self { inner: -self.inner }
            }
        }

        impl<const P: u32> Add for $fixdec_type<P> {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                Self { inner: self.inner + rhs.inner }
            }
        }

        impl<const P: u32> Sub for $fixdec_type<P> {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                Self { inner: self.inner - rhs.inner }
            }
        }

        impl<const P: u32> AddAssign for $fixdec_type<P> {
            fn add_assign(&mut self, rhs: Self) {
                self.inner += rhs.inner;
            }
        }

        impl<const P: u32> SubAssign for $fixdec_type<P> {
            fn sub_assign(&mut self, rhs: Self) {
                self.inner -= rhs.inner;
            }
        }

        #[cfg(feature="serde")]
        use serde::{Serialize, Deserialize, Serializer, Deserializer};

        #[cfg(feature="serde")]
        impl<const P: u32> Serialize for $fixdec_type<P> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
            {
                // XXX how to selete dump type?
                if serializer.is_human_readable() {
                    self.to_string().serialize(serializer)
                } else {
                    Into::<f64>::into((*self)).serialize(serializer)
                }
            }
        }

        #[cfg(feature="serde")]
        impl<'de, const P: u32> Deserialize<'de> for $fixdec_type<P> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>
            {
                use serde::de::{self, Visitor};

                struct FixDecVistor<const P: u32>;

                macro_rules! visit_num {
                    ($func_name:ident, $num_type:ty) => {
                        fn $func_name<E: de::Error>(self, n: $num_type) -> Result<Self::Value, E> {
                            $fixdec_type::<P>::try_from(n)
                                .map_err(|_| E::custom("decimal overflow"))
                        }
                    }
                }

                impl<'de, const P: u32> Visitor<'de> for FixDecVistor<P> {
                    type Value = $fixdec_type<P>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        write!(formatter, "decimal")
                    }

                    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                        $fixdec_type::<P>::from_str(s)
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

                deserializer.deserialize_any(FixDecVistor)
            }
        }
    };
}

use super::{ParseError, Rounding};
pub fn parse_rounding(s: &str, kind: Rounding) -> Result<bool, ParseError> {
    if s.chars().any(|ch| ch.to_digit(10).is_none()) {
        return Err(ParseError::Invalid);
    }

    let is_carry = match kind {
        Rounding::Floor => false,
        Rounding::Ceil => !s.trim_matches('0').is_empty(),
        Rounding::Round => {
            if let Some(first) = s.chars().next() {
                first >= '5'
            } else {
                false
            }
        }
        Rounding::Unexpected => {
            if s.trim_matches('0').is_empty() {
                false
            } else {
                return Err(ParseError::Precision);
            }
        }
    };
    Ok(is_carry)
}

// convert FixDecX to another FixDecY type, where Y > X
macro_rules! convert_into {
    ($from_type:ident, $into_mod:ident, $into_type:ident) => {
        use crate::$into_mod::$into_type;
        impl<const P: u32> Into<$into_type<P>> for $from_type<P> {
            #[doc = concat!("Convert ", stringify!($from_type), " into ", stringify!($into_type))]
            /// with same precision.
            fn into(self) -> $into_type<P> {
                $into_type::<P>::from_inner(self.inner.into())
            }
        }
    }
}

// try to convert FixDecX to another FixDecY type, where Y < X
macro_rules! convert_try_into {
    ($from_type:ident, $into_mod:ident, $into_type:ident) => {
        use crate::$into_mod::$into_type;
        impl<const P: u32> TryInto<$into_type<P>> for $from_type<P> {
            type Error = ();
            #[doc = concat!("Try to convert ", stringify!($from_type), " into ", stringify!($into_type))]
            /// with same precision. Fail if overflow occurred.
            fn try_into(self) -> Result<$into_type<P>, Self::Error> {
                if let Ok(inner) = self.inner.try_into() {
                    Ok($into_type::<P>::from_inner(inner))
                } else {
                    Err(())
                }
            }
        }
    }
}

// define rounding_div_X functions used outside by fixdecX.rs
macro_rules! make_rounding_div {
    ($fn_name:ident, $inner_type:ty) => {
        pub const fn $fn_name(lhs: $inner_type, rhs: $inner_type, kind: Rounding) -> Option<$inner_type> {
            if rhs == 0 {
                return None;
            }
            let d = lhs / rhs;
            let r = lhs % rhs;
            let is_carry = match kind {
                Rounding::Floor => false,
                Rounding::Ceil => r != 0,
                Rounding::Round => r * 2 >= rhs,
                Rounding::Unexpected => if r == 0 { false } else { return None; }
            };
            Some(d + is_carry as $inner_type)
        }
    }
}
make_rounding_div!(rounding_div_i32, i32);
make_rounding_div!(rounding_div_i64, i64);
make_rounding_div!(rounding_div_i128, i128);
make_rounding_div!(rounding_div_u128, u128);

// define convert_opt_X_to_Y functions used outside by fixdecX.rs
macro_rules! make_convert_to_lower {
    ($fn_name:ident, $inner_type:ty, $lower_type:ty) => {
        pub const fn $fn_name(a: Option<$inner_type>) -> Option<$lower_type> {
            match a {
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
}
make_convert_to_lower!(convert_opt_i128_to_i64, i128, i64);
make_convert_to_lower!(convert_opt_i64_to_i32, i64, i32);
make_convert_to_lower!(convert_opt_i32_to_i16, i32, i16);


// export macros
pub(crate) use define_fixdec;
pub(crate) use convert_into;
pub(crate) use convert_try_into;
