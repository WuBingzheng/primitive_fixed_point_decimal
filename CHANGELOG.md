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
