// Implement serde for ConstScaleFpdec and OobFmt.
//
// Enabled by `serde` feature.

use crate::fpdec_inner::FpdecInner;
use crate::{ConstScaleFpdec, OobFmt};

use core::{fmt, num::ParseIntError};

use num_traits::{FromPrimitive, Num};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// == ConstScaleFpdec

impl<I, const S0: i32> Serialize for ConstScaleFpdec<I, S0>
where
    I: FpdecInner + fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // XXX how to selete dump type?
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            Into::<f64>::into(*self).serialize(serializer)
        }
    }
}

impl<'de, I, const S: i32> Deserialize<'de> for ConstScaleFpdec<I, S>
where
    I: FromPrimitive + FpdecInner + Num<FromStrRadixErr = ParseIntError>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use core::marker::PhantomData;
        use core::str::FromStr;
        use serde::de::{self, Visitor};

        struct ConstScaleFpdecVistor<I, const S: i32>(PhantomData<I>);

        macro_rules! visit_num {
            ($func_name:ident, $num_type:ty) => {
                fn $func_name<E: de::Error>(self, n: $num_type) -> Result<Self::Value, E> {
                    ConstScaleFpdec::try_from(n).map_err(|_| E::custom("decimal overflow"))
                }
            };
        }

        impl<'de, I, const S: i32> Visitor<'de> for ConstScaleFpdecVistor<I, S>
        where
            I: FromPrimitive + FpdecInner + Num<FromStrRadixErr = ParseIntError>,
        {
            type Value = ConstScaleFpdec<I, S>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "decimal")
            }

            fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                ConstScaleFpdec::from_str(s).map_err(E::custom)
            }

            visit_num!(visit_f32, f32);
            visit_num!(visit_f64, f64);
            visit_num!(visit_i8, i8);
            visit_num!(visit_i16, i16);
            visit_num!(visit_i32, i32);
            visit_num!(visit_i64, i64);
            visit_num!(visit_i128, i128);
            visit_num!(visit_u8, u8);
            visit_num!(visit_u16, u16);
            visit_num!(visit_u32, u32);
            visit_num!(visit_u64, u64);
            visit_num!(visit_u128, u128);
        }

        deserializer.deserialize_any(ConstScaleFpdecVistor(PhantomData))
    }
}

// == OobFmt for OobScaleFpdec

impl<I> Serialize for OobFmt<I>
where
    I: FpdecInner + fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

/// Because we need to guess the scale, so we can load from
/// string only, but not integer or float numbers.
impl<'de, I> Deserialize<'de> for OobFmt<I>
where
    I: FromPrimitive + FpdecInner + Num<FromStrRadixErr = ParseIntError>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use core::marker::PhantomData;
        use core::str::FromStr;
        use serde::de::{self, Visitor};

        struct OobFmtVistor<I>(PhantomData<I>);

        impl<'de, I> Visitor<'de> for OobFmtVistor<I>
        where
            I: FromPrimitive + FpdecInner + Num<FromStrRadixErr = ParseIntError>,
        {
            type Value = OobFmt<I>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "decimal")
            }

            fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
                OobFmt::from_str(s).map_err(E::custom)
            }
        }

        // TODO:
        // 1. why deserialize_any() works for ConstScaleFpdec?
        // 2. move to serde.rs?
        // 3. more rescale() to fpdec_inner.rs?
        deserializer.deserialize_str(OobFmtVistor(PhantomData))
    }
}
