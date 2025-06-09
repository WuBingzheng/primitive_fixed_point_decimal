# Comparation to Other Decimal Crates

This document compares `primitive_fixed_point_decimal` crate to
[`rust_decimal`](https://docs.rs/rust_decimal) and
[`bigdecimal`](https://docs.rs/bigdecimal) crates.

Because I have not used these 2 other crates in real project, the following
comparison is superficial and subjective.

Note: `primitive_fixed_point_decimal` supports 2 types, `StaticPrecFpdec`
and `OobPrecFpdec`. They differ only in the way they specify precision.
So only the former is used in the following description.


## Representation

A decimal consists of 3 parts: a scale (precision), a sign, and the
mantissa (significant digits).

Now let's look at how the three crates represent these parts respectively.


### `primitive_fixed_point_decimal::StaticPrecFpdec`

The *precision* is specified in the type's constant generics. The *sign*
and *mantissa* are represented by the underlying signed integer.

For example, `StaticPrecFpdec<i64, 4>` means using `i64` as the underlying
representation for sign and mantissa, and `4` is the precision.

The memory layout:

```
+-?--------------- ... ---+
| signed mantissa         |
+----------------- ... ---+
```

The size (`?` above) depends on the type of underlying signed integer,
which supports all Rust primitive signed integers: `i8`, `i16`, `i32`,
`i64` and `i128`. For example, `StaticPrecFpdec<i64, 4>` takes 64 bits
(8 bytes).

The [definition](https://docs.rs/primitive_fixed_point_decimal/latest/src/primitive_fixed_point_decimal/static_prec_fpdec.rs.html#18):

```rust
pub struct StaticPrecFpdec<I, const P: i32>(I);
```

where `I` is the type of underlying signed integer.


### `rust_decimal::Decimal`

1 bit for *sign*, 8 bits for *scale*, 23 bits unused, and 96 bits for *mantissa*.
So it takes 128 bits (16 bytes) totally.

The memory layout:

```
+-32---------------+-96------------- ... --+
|sign,scale,unused | mantissa              |
+------------------+---------------- ... --+
```

The [definition](https://docs.rs/rust_decimal/latest/src/rust_decimal/decimal.rs.html#115-126):

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

The memory layout:

```
+-64-----+-8----+-?-----+-usize*3----------------------------+
| scale  | sign |padding| mantissa: Vec<u32/u64>             |
+--------+------+-------+---+--------------------------------+
                            |
                            V-?------- ... ---+
                            | mantissa        |
                            +--------- ... ---+
```

The [definition](https://docs.rs/bigdecimal/latest/src/bigdecimal/lib.rs.html#206-210):

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

Obviously the `BigDecimal` is not `Copy`.


In summary, `primitive_fixed_point_decimal` has the most compact memory layout,
while `bigdecimal` has the strongest expression ability (unlimited mantissa).


## Operations

In `primitive_fixed_point_decimal`, the `+`, `-` and comparation operations
work only between decimals of the same precision, which is designed intentionally.
The `*` and `/` operations can work between decimals of different precisions
and can also set the result's precision. See the crate's document for details.

Other 2 crates both support `+`, `-` and comparation operations between
decimals of different precisions. They first rescale the decimal with smaller
precision into the larger precision, and then execute the operations. And
the result's precision of `*` operation is the sum of 2 oprands. Because
`rust_decimal` has limited mantissa which makes it easy to become overflow,
it will also try to reduce the result's precision if overflow.


## How to Choose

The biggest characteristic of `primitive_fixed_point_decimal` is real
fixed-point. Thus, its application scenarios are very clear.

For specific applications, if you know the precisions required, such as in
a financial system using 2 precisions for balance and 6 for prices, then
it is suitable for this crate.

While for general-purpose applications or libraries, where you don't know the
precision that the end users will need, such as in storage systems like
Redis, then it is not suitable for this crate.
