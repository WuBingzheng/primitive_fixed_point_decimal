I benchmarked 2 decimal crates: `rust_decimal` and `primitive_fixed_point_decimal`.
The benchmark results are listed below, along with an analysis of the possible
reasons for their performance differences.

[`rust_decimal`](https://docs.rs/rust_decimal) is currently the most popular
decimal crate. It is floating-point.

I developed this crate [`primitive_fixed_point_decimal`](https://docs.rs/primitive_fixed_point_decimal)
for a previous project where I needed a fixed-point decimal implementation.

Floating-point versus fixed-point is the fundamental difference between these 2 crates.


# Representation

Before listing the benchmark results, I'd like to explain how each crate represent a decimal.

`rust_decimal` uses a floating-point design: each instance stores a scale value.
Its decimal type [Decimal](https://docs.rs/rust_decimal/1.40.0/rust_decimal/struct.Decimal.html)
is defined as:

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

As said in the comment, the decimal type includes a 8-bit scale, 1-bit sign,
and 96 bits of mantissa.

In contrast, this crate `primitive_fixed_point_decimal` uses a fixed-point design,
where the scale is bound to the type rather than stored per instance.
Its decimal type
[`ConstScaleFpdec`](https://docs.rs/primitive_fixed_point_decimal/latest/primitive_fixed_point_decimal/struct.ConstScaleFpdec.html)
is defined as:

```rust
#[repr(transparent)]
pub struct ConstScaleFpdec<I, const S: i32>(I);
```

where `I` is the underlying integer which holds the mantissa, and `S` is the
constant scale.

The underlying integer could be any primitive integer type. To be close to
`rust_decimal`'s 96-bit mantissa, we test `i64` and `i128` here. So here
we have 3 types to benchmark and compare:

- `primitive_fixed_point_decimal::ConstScaleFpdec<i64, 6>`
- `primitive_fixed_point_decimal::ConstScaleFpdec<i128, 6>`
- `rust_decimal::Decimal`

They have 18, 28, and 38 decimal significant digits respectively.
Therefore, this is destined not to be a fair comparison.

NOTE: As implied by this type name, this crate also supports another type
[`OobScaleFpdec`](https://docs.rs/primitive_fixed_point_decimal/latest/primitive_fixed_point_decimal/struct.OobScaleFpdec.html)
where the scale is not constant, but out-of-band. It provides greater flexibility,
but the API is more verbose.
See [the documentation](https://docs.rs/primitive_fixed_point_decimal/latest/primitive_fixed_point_decimal/index.html#specify-scale) for details.
The performance of both is the same. The benchmarks here only use `ConstScaleFpdec`.


# Environment

Versions:

- Rust: `cargo 1.93.0 (083ac5135 2025-12-15)`
- `criterion`: `0.7`
- `rust_decimal`: `1.40.0`
- `primitive_fixed_point_decimal`: `1.3.0`

Machines:

- Ubuntu 22.04 @AMD EPYC 9754
- Ubuntu 16.04 @Intel Xeon, 2500 MHZ
- MacOS 13.5 @M1

The results varied considerably at different Machines.
You are welcome to run the benchmark on your own computer:

```bash
git clone https://github.com/WuBingzheng/primitive_fixed_point_decimal.git
cd primitive_fixed_point_decimal
cargo bench
```


# Results

```
                |     Ubuntu 22.04      ||     Ubuntu 16.04      ||     MacOS 13.5
                |     AMD EPYC 9754     ||     Intel Xeon        ||     M1
----------------+-----------------------++-----------------------++----------------------
                |  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
add             |  0.32 |  0.32 |  9.16 ||  0.31 |  0.63 | 10.43 ||  0.31 |  0.31 |  3.21
add-rescale     |    \  |    \  | 10.10 ||    \  |    \  | 12.35 ||    \  |    \  |  3.62
add-rescale-big |    \  |    \  | 14.13 ||    \  |    \  | 17.77 ||    \  |    \  |  6.38
add-rescale-huge|    \  |    \  | 42.79 ||    \  |    \  |109.95 ||    \  |    \  | 17.12
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
mul-pure        |  0.32 |  0.32 |  9.40 ||  0.31 |  0.63 | 10.74 ||  0.31 |  0.31 |  2.88
mul-rescale     |  2.89 | 11.98 |    \  || 12.18 | 22.40 |    \  ||  1.75 |  4.78 |    \
mul-rescale-big |  9.38 | 31.61 | 37.54 || 38.25 | 76.37 |104.34 ||  6.32 | 20.06 | 15.90
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
div-pure        |  2.26 | 14.22 | 10.35 ||  4.75 | 32.39 | 14.54 ||  1.91 |  6.88 |  4.19
div-rescale     |  4.54 | 12.95 | 57.52 || 14.40 | 24.98 | 59.36 ||  2.88 |  6.40 | 28.81
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
load-short      |  9.37 | 15.95 |  9.70 || 12.73 | 26.27 | 11.37 ||  7.55 | 10.76 |  3.83
load-long       | 20.80 | 31.13 | 17.41 || 28.59 | 43.31 | 24.61 || 17.64 | 26.76 | 14.36
dump-short      | 25.66 | 56.98 | 38.72 || 39.07 | 87.33 | 51.14 || 17.03 | 51.09 | 21.05
dump-long       | 32.46 | 50.56 |103.97 || 47.92 |138.05 |126.73 || 21.73 | 44.36 | 70.75
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
app             | 10.81 | 33.06 |129.82 || 33.69 | 66.27 |153.79 ||  5.96 | 16.56 | 49.61
```

- `P-64`: `primitive_fixed_point_decimal::ConstScaleFpdec<i64, 6>`
- `P-128`: `primitive_fixed_point_decimal::ConstScaleFpdec<i128, 6>`
- `R-dec`: `rust_decimal::Decimal`

All the numbers are in nanosecond.

Next, let's explore the reasons behind these numbers.


# Analysis

## BENCH: addition

Let's copy the results about addition here,

```
                |     Ubuntu 22.04      ||     Ubuntu 16.04      ||     MacOS 13.5
                |     AMD EPYC 9754     ||     Intel Xeon        ||     M1
----------------+-----------------------++-----------------------++----------------------
                |  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
add             |  0.32 |  0.32 |  9.16 ||  0.31 |  0.63 | 10.43 ||  0.31 |  0.31 |  3.21
add-rescale     |    \  |    \  | 10.10 ||    \  |    \  | 12.35 ||    \  |    \  |  3.62
add-rescale-big |    \  |    \  | 14.13 ||    \  |    \  | 17.77 ||    \  |    \  |  6.38
add-rescale-huge|    \  |    \  | 42.79 ||    \  |    \  |109.95 ||    \  |    \  | 17.12
----------------+-----------------------++-----------------------++----------------------
                |                       ||                       ||
```

One of the [main features](https://docs.rs/primitive_fixed_point_decimal/latest/primitive_fixed_point_decimal/index.html#features)
of this crate `primitive_fixed_point_decimal` is that, the `+` and `-` operations
only perform between same types in same scale. There is no implicitly type or
scale conversion. This also makes the operations super fast, roughly equivalent
to one single CPU instruction. As shown in the results, most of them are 0.31 ns,
which is really fast.

While for floating-point `rust_decimal`, implicit rescaling is the most basic
feature. So we constructed 4 test cases:

1. `add`, both operands have the same scale, no rescaling needed. From the
   benchmark results, it can be seen that even without rescaling, `rust_decimal`'s
   addition is still very slow. Much slower than `primitive_fixed_point_decimal`.
   I suspect this may be because it needs to compare
   the scales and handle the signs of the two operands, and these conditional
   branches reduce the speed.

2. `add-rescale`, the two operands have different scales, where the operand
   with the smaller scale needs to be rescaled to match the scale of the
   other operand. Although rescaling is required, it only adds a small
   proportion of time. So for relatively small numbers, the rescale itself is
   not slow.

3. `add-rescale-big`, the operand is large, requiring more time for rescaling;

4. `add-rescale-huge`, the operand is so huge that it overflows after rescaling,
   so it can only rescale the other operand downward instead. This is very slow.
   Also this may lose precision.

See the `bench_add()` function in the benchmark
[source code](https://github.com/WuBingzheng/primitive_fixed_point_decimal/blob/master/benches/vs_rust_decimal.rs)
for more details.


## BENCH: multiplication

Let's copy the results about multiplication here,

```
                |     Ubuntu 22.04      ||     Ubuntu 16.04      ||     MacOS 13.5
                |     AMD EPYC 9754     ||     Intel Xeon        ||     M1
----------------+-----------------------++-----------------------++----------------------
                |  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
add             |  0.32 |  0.32 |  9.16 ||  0.31 |  0.63 | 10.43 ||  0.31 |  0.31 |  3.21
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
mul-pure        |  0.32 |  0.32 |  9.40 ||  0.31 |  0.63 | 10.74 ||  0.31 |  0.31 |  2.88
mul-rescale     |  2.89 | 11.98 |    \  || 12.18 | 22.40 |    \  ||  1.75 |  4.78 |    \
mul-rescale-big |  9.38 | 31.61 | 37.54 || 38.25 | 76.37 |104.34 ||  6.32 | 20.06 | 15.90
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
                |                       ||                       ||
```

Another of the [main features](https://docs.rs/primitive_fixed_point_decimal/latest/primitive_fixed_point_decimal/index.html#features)
of this crate `primitive_fixed_point_decimal` is that, the `*` and `/` operations
accept operand with different types and scales, and allow the result's scale
specified.

If the scale of the result type equals the sum of the scales of the two operands,
then it only need to multiply the underlying integers without any rescaling.
In this case, `mul-pure`, it is extremely fast. As fast as the addition.

If they are not equal, rescaling is required, which slows things down. For smaller
numbers (the product of underlying integers does not overflow, which is also the
case for most situations), there are optimizations in the implementation, so
`mul-rescale` is faster than `mul-rescale-big`.

While for floating-point `rust_decimal`, the rescaling is implicit. Whether to
rescale depends on the size of the product. If there is no overflow within 96
bits, then no rescaling is needed. It's fast. If overflow occurs, rescaling is
required. It's slow.
Therefore, rescaling is not performed for small numbers, so there is no
benchmark result in `mul-rescale` for `rust_decimal`.

For the first case, `mul-pure`, all 3 types are as fast as their addition operation.
This shows that the speed of addition and multiplication CPU instruction is
roughly the same.

See the `bench_mul()` function in the benchmark
[source code](https://github.com/WuBingzheng/primitive_fixed_point_decimal/blob/master/benches/vs_rust_decimal.rs)
for more details.


## BENCH: division

Let's copy the results about division here,

```
                |     Ubuntu 22.04      ||     Ubuntu 16.04      ||     MacOS 13.5
                |     AMD EPYC 9754     ||     Intel Xeon        ||     M1
----------------+-----------------------++-----------------------++----------------------
                |  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
div-pure        |  2.26 | 14.22 | 10.35 ||  4.75 | 32.39 | 14.54 ||  1.91 |  6.88 |  4.19
div-rescale     |  4.54 | 12.95 | 57.52 || 14.40 | 24.98 | 59.36 ||  2.88 |  6.40 | 28.81
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
                |                       ||                       ||
```

For the `primitive_fixed_point_decimal`, similar to the multiplication mentioned
earlier, whether division requires rescaling also depends on the scale specified
for the result type. However, because this crate's division supports
[multiple rounding strategies](https://docs.rs/primitive_fixed_point_decimal/latest/primitive_fixed_point_decimal/enum.Rounding.html),
the logic is relatively complex and cannot be mapped to a single integer division
instruction. Therefore, even without rescaling, it is relatively slow. And needing
rescaling does not make it much slower. (Some are even faster; the reason is unclear.)

While for floating-point `rust_decimal`, whether to rescale depends on whether
the underlying integer can divide evenly. If it cannot, a very slow rescale is required.

See the `bench_div()` function in the benchmark
[source code](https://github.com/WuBingzheng/primitive_fixed_point_decimal/blob/master/benches/vs_rust_decimal.rs)
for more details.


## BENCH: load and dump

Let's copy the results about load and dump here,

```
                |     Ubuntu 22.04      ||     Ubuntu 16.04      ||     MacOS 13.5
                |     AMD EPYC 9754     ||     Intel Xeon        ||     M1
----------------+-----------------------++-----------------------++----------------------
                |  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
load-short      |  9.37 | 15.95 |  9.70 || 12.73 | 26.27 | 11.37 ||  7.55 | 10.76 |  3.83
load-long       | 20.80 | 31.13 | 17.41 || 28.59 | 43.31 | 24.61 || 17.64 | 26.76 | 14.36
dump-short      | 25.66 | 56.98 | 38.72 || 39.07 | 87.33 | 51.14 || 17.03 | 51.09 | 21.05
dump-long       | 32.46 | 50.56 |103.97 || 47.92 |138.05 |126.73 || 21.73 | 44.36 | 70.75
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
                |                       ||                       ||
```

The `load` means parsing a number from string. The `dump` means formatting the
number into string.

The `short` means a short string `"12.34"`. The `long` means a long string
`"123456789012.123456"`.

There is nothing to discuss about this part.


## BENCH: app

From the comparison and explanation above, due to the differences between
fixed-point and floating-point, it is difficult to accurately compare these
two crates. Ultimately, which one is faster depends on the specific application
scenario.

Here, we constructed a foreign exchange trading application scenario. For
a single trade, it involves fund changes for both buyer and seller accounts
and execution fees. In total, it involves 3 multiplications, 4 additions,
and 4 subtractions.

See the `bench_app()` function in the benchmark
[source code](https://github.com/WuBingzheng/primitive_fixed_point_decimal/blob/master/benches/vs_rust_decimal.rs)
for more details.

Let's see the results:

```
                |     Ubuntu 22.04      ||     Ubuntu 16.04      ||     MacOS 13.5
                |     AMD EPYC 9754     ||     Intel Xeon        ||     M1
----------------+-----------------------++-----------------------++----------------------
                |  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec ||  P-64 | P-128 | R-dec
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
add             |  0.32 |  0.32 |    \  ||  0.31 |  0.63 |   \   ||  0.31 |  0.31 |    \ 
add-rescale     |    \  |    \  | 10.10 ||    \  |    \  | 12.35 ||    \  |    \  |  3.62
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
mul-pure        |    \  |    \  |  9.40 ||    \  |    \  | 10.74 ||    \  |    \  |  2.88
mul-rescale     |  2.89 | 11.98 |    \  || 12.18 | 22.40 |    \  ||  1.75 |  4.78 |    \
----------------+-------+-------+-------++-------+-------+-------++-------+-------+------
app             | 10.81 | 33.06 |129.82 || 33.69 | 66.27 |153.79 ||  5.96 | 16.56 | 49.61
```

The results roughly meet expectations:

    TIME(app) := TIME(add) * 8 + TIME(mul) * 3


# Conclusion

`primitive_fixed_point_decimal` is faster than `rust_decimal` in most cases.

`primitive_fixed_point_decimal` delivers more stable performance. `rust_decimal`
experiences significant performance fluctuations due to implicit rescaling.
