// Implement bytemuck for ConstScaleFpdec and OobScaleFpdec.
//
// Enabled by `bytemuck` feature.

use crate::{ConstScaleFpdec, OobScaleFpdec};

// == ConstScaleFpdec

// SAFETY: `ConstScaleFpdec` is `#[repr(transparent)]` over `I` and has no
// invariant on its value, so it is `Zeroable`/`Pod` exactly when `I` is.
unsafe impl<I: bytemuck::Zeroable, const S: i32> bytemuck::Zeroable for ConstScaleFpdec<I, S> {}
unsafe impl<I: bytemuck::Pod, const S: i32> bytemuck::Pod for ConstScaleFpdec<I, S> {}
unsafe impl<I, const S: i32> bytemuck::TransparentWrapper<I> for ConstScaleFpdec<I, S> {}

// == OobScaleFpdec

// SAFETY: `OobScaleFpdec` is `#[repr(transparent)]` over `I` and has no
// invariant on its value, so it is `Zeroable`/`Pod` exactly when `I` is.
unsafe impl<I: bytemuck::Zeroable> bytemuck::Zeroable for OobScaleFpdec<I> {}
unsafe impl<I: bytemuck::Pod> bytemuck::Pod for OobScaleFpdec<I> {}
unsafe impl<I> bytemuck::TransparentWrapper<I> for OobScaleFpdec<I> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as primitive_fixed_point_decimal;
    use crate::fpdec;

    type Dec32 = OobScaleFpdec<i32>;

    #[test]
    fn test_const_scale_fpdec() {
        type Dec = ConstScaleFpdec<i64, 4>;

        let d: Dec = bytemuck::cast(12345_i64);
        assert_eq!(d, Dec::from_mantissa(12345));
        assert_eq!(bytemuck::cast::<Dec, i64>(d), 12345);

        let ds: &[Dec] = bytemuck::cast_slice(&[1_i64, -2, 3]);
        assert_eq!(ds[1], fpdec!(-0.0002));

        assert_eq!(<Dec as bytemuck::Zeroable>::zeroed(), Dec::ZERO);

        use bytemuck::TransparentWrapper;
        assert_eq!(Dec::wrap(7), Dec::from_mantissa(7));
        assert_eq!(Dec::peel_slice(ds), &[1, -2, 3]);
    }

    #[test]
    fn test_oob_scale_fpdec() {
        let d: Dec32 = bytemuck::cast(12345_i32);
        assert_eq!(d, Dec32::from_mantissa(12345));
        assert_eq!(bytemuck::cast::<Dec32, i32>(d), 12345);

        let ds: &[Dec32] = bytemuck::cast_slice(&[1_i32, -2, 3]);
        assert_eq!(ds[1], Dec32::from_mantissa(-2));

        assert_eq!(<Dec32 as bytemuck::Zeroable>::zeroed(), Dec32::ZERO);

        use bytemuck::TransparentWrapper;
        assert_eq!(Dec32::wrap(7), Dec32::from_mantissa(7));
        assert_eq!(Dec32::peel_slice(ds), &[1, -2, 3]);
    }
}
