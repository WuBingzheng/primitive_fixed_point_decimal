macro_rules! define_convert_into_longer {
    (
        $static_type:ident,
        $oob_type:ident,

        $(
            ($into_mod:ident, $into_static_type:ident, $into_oob_type:ident),
        )*
    ) => {
        $(
            use crate::$into_mod::$into_static_type;
            impl<const P: i32> From<$static_type<P>> for $into_static_type<P> {
                fn from(d: $static_type<P>) -> Self {
                    $into_static_type::<P>::from_inner(d.inner.into())
                }
            }

            use crate::$into_mod::$into_oob_type;
            impl From<$oob_type> for $into_oob_type {
                fn from(d: $oob_type) -> Self {
                    $into_oob_type::from_inner(d.inner.into())
                }
            }
        )*
    }
}

macro_rules! define_convert_try_into_shorter {
    (
        $static_type:ident,
        $oob_type:ident,

        $(
            ($into_mod:ident, $into_static_type:ident, $into_oob_type:ident),
        )*
    ) => {
        $(
            use crate::$into_mod::$into_static_type;
            impl<const P: i32> TryFrom<$static_type<P>> for $into_static_type<P> {
                type Error = std::num::TryFromIntError;
                fn try_from(d: $static_type<P>) -> Result<Self, Self::Error> {
                    Ok($into_static_type::<P>::from_inner(d.inner.try_into()?))
                }
            }

            use crate::$into_mod::$into_oob_type;
            impl TryFrom<$oob_type> for $into_oob_type {
                type Error = std::num::TryFromIntError;
                fn try_from(d: $oob_type) -> Result<Self, Self::Error> {
                    Ok($into_oob_type::from_inner(d.inner.try_into()?))
                }
            }
        )*
    }
}

pub(crate) use define_convert_into_longer;
pub(crate) use define_convert_try_into_shorter;
