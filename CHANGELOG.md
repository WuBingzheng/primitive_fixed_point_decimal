# Unreleased

- **Add `num-traits-impls` feature**: `Zero`, `One`, `Num`, `Bounded`, `Signed`,
  `CheckedAdd`/`Sub`/`Mul`/`Div`/`Rem`/`Neg`, `ToPrimitive`, `FromPrimitive`.
  `NumCast`/`PrimInt` are deliberately not implemented -- a decimal is not an
  integer, and `NumCast::from` would have to silently choose a rounding mode.

- **Add `zerocopy` feature**: `IntoBytes`, `FromBytes`, `Immutable`,
  `KnownLayout` for `ConstScaleFpdec` and `OobScaleFpdec`.

- **Add `pyo3` feature**: `IntoPyObject`/`FromPyObject` for `ConstScaleFpdec`
  and `OobFmt`, converting to and from Python's `decimal.Decimal` without loss.
  Accepts `Decimal`, `str` and `int`; rejects `float` and `bool` (the latter is
  an `int` subclass in Python). Implies the new `std` feature.

- **Add `std` and `full` features.** The crate is still `no_std` by default;
  `std` exists because PyO3 requires it.

- **Fix: serde was lossy and broken on binary formats.** `Serialize` wrote an
  `f64`, which holds ~15.9 significant digits where `ConstScaleFpdec<i128, 12>`
  holds up to 39 -- so 123456789.123456789012 came back as 123456789.12345679.
  `Deserialize` called `deserialize_any`, which non-self-describing formats
  reject outright, so bincode and friends failed with
  `DeserializeAnyNotSupported`. Binary formats now write and read the raw
  mantissa: exact, 16 bytes for an `i128`, with the scale carried by the type.
  Human-readable formats are unchanged (exact decimal string).

  **This changes the binary wire format** (`f64` -> mantissa). Anything that
  round-tripped through a *self-describing* binary format such as CBOR or
  MessagePack must be re-encoded; non-self-describing formats could not
  round-trip at all before, so nothing working is broken.

  `Serialize` now requires `I: Serialize` and `Deserialize` requires
  `I: Deserialize`. Every primitive integer satisfies both.

- **Add `ConstScaleFpdec::checked_rem` and the `%` operator.** Required by
  `num_traits::NumOps`, and useful on its own; both operands share a scale, so
  the remainder is exact.

- **Change: the integer `Mul`/`Div`/`MulAssign`/`DivAssign` impls now bound
  `J: Into<I> + PrimInt` instead of `J: Into<I> + Num`.** `Num` was only ever a
  disjointness marker ("the `Num` to avoid conflicting implementations only"),
  and it worked because nothing implemented `Num` for a decimal --
  `num-traits-impls` now does. `PrimInt` is already this crate's marker for
  "primitive integer" (see `IntoRatioInt`) and states the intent directly. All
  primitive integers still work; a custom `Num` type that was also `Into<I>`
  would no longer.

# v1.5.0 (2026-08-08)

- Use div-pow10 crate.
- Fix rounding bug for negative quotient.
- Rewrite fuzz.

# v1.4.2 (2026-04-18)

- Fix bug in division for i128 type.

# v1.4.1 (2026-04-17)

- Optimize multiplication of 64-bit and 128-bit for small numbers.
- Fix Display if the fraction part rounding up to exceed exp.
- Add fuzz tests.

# v1.4.0 (2026-03-14)

- Optimize multiplication of 64-bit and 128-bit.

# v1.3.1 (2026-03-05)

- Update the module document only.

# v1.3.0 (2026-02-05)

- Implement `core::iter::Sum`.
- Fix and optimize `rounding_div()`.
- Add benchmark.

# v1.2.1 (2026-01-29)

- Optimize Display.

# v1.2.0 (2026-01-18)

- Add formatting options supporting, such as width and precision.

# v1.1.0 (2026-01-01)

- Add `OobScaleFpdec::checked_mul_const_scale` and more.

# v1.0.1 (2025-12-31)

- Add `#[must_use]` for many APIs.
- Add `#[repr(transparent)]` for the type.
- Add `TryFrom` for unsigned integers.

# v1.0.0 (2025-12-01)

- Just update to v1.0.0!

# v0.11.0 (2025-10-20)

- Remove the *Cumulative Error*.
- Remove the `CumErr` argument for all `*_ext` methods.
- Rename `round_with_rounding` method to `round_ext`.
- Add CHANGELOG.md.

# v0.10.1 (2025-10-18)

- Add `no-std::no-alloc` category in Cargo.toml.

# v0.10.0 (2025-10-13)

- Add `checked_mul_ratio` method.
- Remove deprecated `to_float` method.

# v0.9.1 (2025-09-22)

- Update README.

# v0.9.0 (2025-09-12)

- Add unsigned support.

# v0.8.1 (2025-07-08)

- Implement `Mul` and `Div` traits.
- Add `to_f32` and `to_f64` to replace `to_float`.
- Fix division without rounding.

# v0.8.0 (2025-06-30)

- Fix division !!!
- Export `FpdecInner` if some one want to implement trait for Decimal.
- Some other optimization.

# v0.7.2 (2025-06-19)

- Add more test.

# v0.7.1 (2025-06-17)

- Fix serde.
 
# v0.7.0 (2025-06-16)

- Rename to `ConstScaleFpdec` and `OobScaleFpdec`.
- Make into `no-std`.
