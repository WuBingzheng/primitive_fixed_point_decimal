macro_rules! define_both_fpdecs {
    (
        $static_type:ident,
        $oob_type:ident,
        $inner_type:ty,
        $digits:expr,

        // These are used only in doc comments.
        $bits:literal,
        $bits_minus_one:literal
    ) => {
        use std::fmt;
        use crate::{ParseError, Rounding};
        use crate::utils::*;

        crate::define_calculations::define_calculations!($inner_type, $digits);

        crate::define_static_prec_fpdec::define_static_prec_fpdec!($static_type, $inner_type, $digits, $bits, $bits_minus_one);

        crate::define_oob_prec_fpdec::define_oob_prec_fpdec!($oob_type, $inner_type, $digits, $bits, $bits_minus_one);

        crate::define_oob_prec_fpdec::define_oob_mul_static!($oob_type, $static_type, $inner_type);

        crate::define_convert::define_convert_each_other!($static_type, $oob_type);
    }
}

pub(crate) use define_both_fpdecs;
