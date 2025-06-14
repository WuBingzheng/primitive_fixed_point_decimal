# Comparison to Other Decimal Crates

This document compares `primitive_fixed_point_decimal` crate to
[`rust_decimal`](https://docs.rs/rust_decimal) and
[`bigdecimal`](https://docs.rs/bigdecimal) crates.

Because I have not used these 2 other crates in real project, the following
comparison is superficial and subjective.

Note: `primitive_fixed_point_decimal` supports 2 types, `ConstScaleFpdec`
and `OobPrecFpdec`. They differ only in the way they specify scale.
So only the former is used in the following description.


## Representation

A decimal consists of 3 parts: a scale, a sign, and the mantissa (significant digits).

Now let's look at how the three crates represent these parts respectively.


### `primitive_fixed_point_decimal::ConstScaleFpdec`

The *scale* is specified in the type's constant generics. The *sign*
and *mantissa* are represented by the underlying signed integer.

For example, `ConstScaleFpdec<i64, 4>` means using `i64` as the underlying
representation for sign and mantissa, and `4` is the scale.

The memory layout:

```
+-?--------------- ... ---+
| signed mantissa         |
+----------------- ... ---+
```

The size (`?` above) depends on the type of underlying signed integer,
which supports all Rust primitive signed integers: `i8`, `i16`, `i32`,
`i64` and `i128`. For example, `ConstScaleFpdec<i64, 4>` takes 64 bits
(8 bytes).

The scale is binded on the decimal *type* but not *instance*, so it is not
shown in the memory layout above.

The [definition](https://docs.rs/primitive_fixed_point_decimal/0.7.0/src/primitive_fixed_point_decimal/const_scale_fpdec.rs.html#23):

```rust
pub struct ConstScaleFpdec<I, const S: i32>(I);
```

where `I` is the type of underlying signed integer.


### `rust_decimal::Decimal`

1 bit for *sign*, 8 bits for *scale*, 23 bits unused, and 96 bits for *mantissa*.
So it takes 128 bits (16 bytes) totally.

The memory layout:

```
+-32---------------+-96------------------- ... --+
|sign,scale,unused | mantissa                    |
+------------------+---------------------- ... --+
```

The [definition](https://docs.rs/rust_decimal/1.37.2/src/rust_decimal/decimal.rs.html#115-126):

```rust
pub struct Decimal {
    // Bits 0-15: unused
    // Bits 16-23: Contains "e", a value between 0-28 that indicates the scale
    // Bits 24-30: unused
    // Bit 31: the sign of the Decimal value, 0 meaning positive and 1 meaning negative.
    flags: u32,
    // The lo, mid, hi, and flags fields contain the representation of the
    // Decimal value as a 96-bit integer.
    hi: u32,
    lo: u32,
    mid: u32,
}
```


### `bigdecimal::BigDecimal`

64 bits for *scale*, 8 bits for *sign*, (sizeof(usize) - 8) bits for padding,
and `sizeof(usize)*3` bits for the Vec of mantissa data.

For example, on 64-bits system, it takes 320 bits (40 bytes) for the meta data
and uncertain-size for mantissa data.

Obviously the `BigDecimal` is not `Copy`.

The memory layout:

```
+-64-------+-8--+-?-----+-usize*3----------------------------+
| scale    |sign|padding| mantissa: Vec<u32/u64>             |
+----------+----+-------+---+--------------------------------+
                            |
                            V-?------------- ... ---+
                            | mantissa              |
                            +--------------- ... ---+
```

The [definition](https://docs.rs/bigdecimal/0.4.8/src/bigdecimal/lib.rs.html#206-210):

```rust
pub struct BigDecimal {
    int_val: BigInt,
    // A positive scale means a negative power of 10
    scale: i64,
}
pub struct BigInt {
    sign: Sign,
    data: BigUint,
}
pub struct BigUint {
    data: Vec<BigDigit>,
}
```

### Summary

The biggest difference of `primitive_fixed_point_decimal`, compared with the
other two crates, is that it binds the scale to decimal *type*, while the
other two crates save the scale in decimal *instance*.

Besides, `primitive_fixed_point_decimal` has the most compact memory layout,
while `bigdecimal` has the strongest expression ability (unlimited mantissa).


## Operations

In `primitive_fixed_point_decimal`, the `+`, `-` and comparison operations
work only between decimals of the same scale, which is designed intentionally.
The `*` and `/` operations can work between decimals of different scales
and can also set the result's scale. See the crate's document for details.

Other 2 crates both support `+`, `-` and comparison operations between
decimals of different scales. They first rescale the decimal with smaller
scale into the larger scale, and then execute the operations. And
the result's scale of `*` operation is the sum of 2 oprands. Because
`rust_decimal` has limited mantissa which makes it easy to become overflow,
it will also try to reduce the result's scale if overflow.


## How to Choose

As I understand it, these types can be categorized into the following:

- `primitive_fixed_point_decimal`: fixed-point in base 10

- `rust_decimal` and `bigdecimal`: floating-point in base 10

- Rust floats(`f32` and `f64`): floating-point in base 2

All `primitive_fixed_point_decimal`, `rust_decimal` and `bigdecimal` are in
base 10, which is the reason they are called `*_decimal`.
Therefore, they can represent decimal fractions accurately, and avoid
calculation error such as `0.1 + 0.2 != 0.3`.

The difference is that `primitive_fixed_point_decimal` is *fixed-point*,
while the 2 others are *floating-point*. Thus, the application scenarios
are very clear.

For specific applications, if you know the scales required, such as in
a financial system using 2 scales for balance and 6 for prices, then
it is suitable for `primitive_fixed_point_decimal`.

While for general-purpose applications or libraries, where you don't
know the scale that the end users will need, then it is suitable for
`rust_decimal` and `bigdecimal`.

Note: [The document of `rust_decimal`](https://docs.rs/rust_decimal/1.37.1/rust_decimal/struct.Decimal.html)
says it's "a fixed-precision decimal number".
And [the document of `bigdecimal`](https://docs.rs/bigdecimal/0.4.8/bigdecimal/index.html)
also implies it's fixed-point by: "avoids common floating point errors".
However, I still think they are floating-point, because the scale is carried
in the number and changes during calculation.
