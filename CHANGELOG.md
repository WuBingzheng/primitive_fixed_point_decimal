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

- **Fix: `u64` multiplication with a scale difference of 19 gave wrong results.**
  `calc_mul_div_exp` took its fast path whenever `a * b < 2^64`, then added the
  rounding extra `10^19 - 1` in `u64`, which can overflow: a debug-build panic,
  and in release a wrapped (wrong) quotient -- `3_000_000_000 * 3_000_000_000`
  at diff 19 with `Ceiling` returned `0` instead of `1`. The fast path now
  requires `a * b < 2^63`, and checks the add where it can still overflow (only
  at a scale difference of 19). The check is keyed on the scale difference, a
  constant for `ConstScaleFpdec`, so it is free: a first version that tested the
  sum itself made `u64` multiplication ~10% slower.

- **Fix: `i128`/`u128` multiplication at a scale difference of 38 violated the
  precondition of `div-pow10`'s `unchecked_div_single_r1b`** (`n <= 2^127`; a
  `debug_assert!`, and undefined for an `unsafe` function) when
  `a * b + rounding extra` exceeded it. Debug builds panicked. The other
  division routine is now used for those dividends -- chosen by the scale
  difference (below 38 the dividend cannot reach `2^127`), not by testing the
  dividend, which cost ~40% of an `i128` multiplication (2.8 -> 4.0 ns).

- **Fix: `round`, `round_ext`, `round_diff` and `round_diff_ext` returned a wrong
  value or wrapped silently when the rounded result does not fit.** E.g. rounding
  `1_i8` up to a multiple of 1000 returned `0`; `u16` 65535 at diff 5 (0.65535)
  returned `0`, not "one unit"; larger values wrapped in release. They now panic
  (`"overflow in decimal rounding"`; documented under `# Panics`).
  **Add `checked_round`, `checked_round_ext`, `checked_round_diff` and
  `checked_round_diff_ext`**, returning `None` instead.

- **Fix: `checked_rem` panicked for `MIN % -1`** (it is `0`). Also through
  `num_traits::CheckedRem`.

- **Fix: string parsing.**
  - A sign was accepted inside the fraction: `"1.-5"` parsed as `0.95`, `"1.+5"`
    as `1.05`, `"0.-5"` as `-0.05`. All are now `ParseError::Invalid`.
  - `"00"` (any run of zeros) was `ParseError::Empty`; every spelling of zero
    (`"0"`, `"-0"`, `"+00"`, `"0.000"`) now parses to zero for every inner type,
    including `"-0"` for the unsigned ones.
  - A number too big to hold its own digits (`"1000"` as `i8`, `"1.001"` at scale 3)
    was reported as `Precision`; it is `Overflow` unless it also has more fraction
    digits than the scale (then still `Precision`, as documented).

- **Fix: `Display` with a precision rounded wrongly when `10^scale` does not fit the
  type** (`u16` 65535 at scale 5 with `{:.0}` printed `0` for 0.65535; now `1`).

- **Fix: scale arithmetic overflowed `i32`** for scales near the type's limits:
  `round_diff(i32::MIN)`, `OobFmt::rescale`, `try_from_str(_, i32::MAX)` and the
  `(float, scale)` conversions panicked in debug builds and misbehaved in release.
  They now return the right error / value.

- **Fix: zero was rejected by integer conversion and `OobFmt::rescale` whenever
  `10^scale` is not representable** in the type (`ConstScaleFpdec<i8, 3>::try_from(0)`
  was `Overflow`, `ConstScaleFpdec<i8, -3>::try_from(0)` was `Precision`).

- **Fix (`num-traits-impls`): `ToPrimitive` returned `None` for representable values**
  -- `ConstScaleFpdec<i8, -3>` of value 1000 (it multiplied in the inner type), and
  anything below 1 when `10^S` does not fit (`u16` at scale 5). The integer methods now
  compute in `i128`/`u128`. `FromPrimitive::from_i128`/`from_u128` went through
  `i64`/`u64` and rejected every value above 2^63 even for 128-bit types.

- **Fix: serde reported "decimal overflow" for every failed integer/float input**,
  including precision loss at a negative scale; it now reports the actual `ParseError`.

- **Fix: `fpdec!` expanded to a hard-coded `primitive_fixed_point_decimal::` path**;
  it uses `$crate`, so it works when the crate is renamed or re-exported.

- Fix: dividing an `OobScaleFpdec` by a `ConstScaleFpdec` panicked with the message
  "overflow in decimal multiplication".

- **Fix: float conversions failed when `10^scale` does not fit the float** (`f32`: scale >= 39,
  `f64`: scale >= 309): `f * inf` is `inf` (or NaN for `0.0`), so even zero was `Overflow`, and
  decimal -> float divided by infinity and returned `0`. The power of ten is now applied in
  chunks.

- **Fix: `ConstScaleFpdec` `checked_mul`/`checked_div` computed the scale difference `S + S2 - SR`
  in `i32`**, which overflows for scales near the limits (panic in debug, wrong value in release);
  it is computed in `i64` and is `None` when out of range.

- Docs: `Rounding::Round` ties go away from zero (it always did).

- **Fix: `checked_mul*` / `checked_div*` with an `rhs` of a different inner type returned `None`
  when `rhs` did not fit the result's type**, even if the result did (`0.0001 * 1000.00` as an
  `i16` times an `i64` holding 100000 is `0.1`). The product is now exact for every pair of inner
  types; the same-type path is unchanged.

- **Fix (`pyo3`): a hostile `Decimal` could force a gigabyte allocation** --
  `Decimal("1E+999999999")` is formatted in full (`format(x, "f")`) before the range check, and
  `Decimal(10**1_000_000)` takes seconds. The exponent / bit length is checked first; such values
  are a `ValueError`.

- Docs: the operators (`+`, `-`, `abs`, ...) panic on overflow in debug builds and wrap in release,
  like the primitive integers; `checked_*` is the profile-independent form.

- **Tests.** An arbitrary-precision oracle (`tests/common`, built on `num-bigint`,
  written from the definition of each rounding mode and sharing no code with the
  crate) now checks every arithmetic, rounding, parsing, formatting and conversion
  entry point for all ten inner types, with boundary-value generation (powers of
  ten and two and their neighbours, exact ties, `MIN`/`MAX`, products straddling
  word boundaries); the 8-bit types are exhaustive. Run with `cargo test` **and**
  `cargo test --release` (debug builds panic on overflow, release builds wrap), and
  scale the sampling with `PFPD_ORACLE_SCALE=10`.

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
