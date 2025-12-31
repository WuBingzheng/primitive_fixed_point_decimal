Primitive fixed-point decimal types.

For example, `ConstScaleFpdec<i64, 4>` means using `i64` as the underlying
representation, and the static scale is `4`.


# Features

- Fixed-point. The scale is bound to the *type* but not each *value*.

- Decimal. Using integer types to represent numbers with a scaling factor
  (also called as "scale") in base 10 to achieve the accuracy. This is a
  [common idea](https://en.wikipedia.org/wiki/Fixed-point_arithmetic#Representation).

- The `+` and `-` operations only perform between same types in same scale.
  There is no implicitly type or scale conversion. This makes sense, for we
  do not want to add `Balance` type by `Price` type.

- The `*` and `/` operations accept operand with different types and scales,
  and allow the result's scale specified. Certainly we need to multiply
  between `Balance` type and `Price` type.

- Supports 2 ways to specify the scale: *const* and *out-of-band*. See
  the [Specify Scale](#specify-scale) section for details.

- Supports both signed and unsigned types.

- Supports scale larger than the significant digits of the underlying integer
  type. For example `ConstScaleFpdec<i8, 4>` represents numbers in range
  [-0.0128, 0.0127].

- Supports negative scale. For example `ConstScaleFpdec<i8, -2>` represents
  numbers in range [-12800, 12700] with step 100.

- Supports serde traits integration (`Serialize`/`Deserialize`) by optional
  `serde` feature flag.

- `no-std` and `no-alloc`.


# Specify Scale

There are 2 ways to specify the scale: *const* and *out-of-band*:

- For the *const* type [`ConstScaleFpdec`], we use Rust's *const generics*
  to specify the scale. For example, `ConstScaleFpdec<i64, 4>` means
  scale is 4.

- For the *out-of-band* type [`OobScaleFpdec`], we do NOT save the
  scale with decimal types, so it's your job to save it somewhere
  and apply it in the following operations later. For example,
  `OobScaleFpdec<i64>` takes no scale information.

Generally, the *const* type is more convenient and suitable for most
scenarios. For example, in traditional currency exchange, you can use
`ConstScaleFpdec<i64, 2>` to represent balance, e.g. `1234.56` USD and
`8888800.00` JPY. And use `ConstScaleFpdec<u32, 6>` to represent all
market prices since 6-digit-scale is big enough for all currency
pairs, e.g. `146.4730` JPY/USD and `0.006802` USD/JPY:

```rust
use primitive_fixed_point_decimal::{ConstScaleFpdec, fpdec};
type Balance = ConstScaleFpdec<i64, 2>; // 2 is enough for all currencies
type Price = ConstScaleFpdec<u32, 6>; // 6 is enough for all markets

let usd: Balance = fpdec!(1234.56);
let price: Price = fpdec!(146.4730);

let jpy: Balance = usd * price;
assert_eq!(jpy, fpdec!(180829.71));
```

However in some scenarios, such as in cryptocurrency exchange, the
price differences across various markets are very significant. For
example `81234.0` in BTC/USDT and `0.000004658` in PEPE/USDT. Here
we need to select different scales for each market. So it's
the *Out-of-band* type:

```rust
use primitive_fixed_point_decimal::{OobScaleFpdec, fpdec};
type Balance = OobScaleFpdec<i64>; // no global scale set
type Price = OobScaleFpdec<u32>; // no global scale set

// each market has its own scale configuration
struct Market {
    base_asset_scale: i32,
    quote_asset_scale: i32,
    price_scale: i32,
}

// let's take BTC/USDT market as example
let btc_usdt = Market {
    base_asset_scale: 8,
    quote_asset_scale: 6,
    price_scale: 1,
};

// we need tell the scale to `fpdec!`
let btc: Balance = fpdec!(0.34, btc_usdt.base_asset_scale);
let price: Price = fpdec!(81234.0, btc_usdt.price_scale);

// we need tell the scale difference to `checked_mul()` method
let diff = btc_usdt.base_asset_scale + btc_usdt.price_scale - btc_usdt.quote_asset_scale;
let usdt = btc.checked_mul(price, diff).unwrap();
assert_eq!(usdt, fpdec!(27619.56, btc_usdt.quote_asset_scale));
```

Obviously it's verbose to use, but offers greater flexibility.

Another example is the SQL `Decimal` data type.
In the server end, the scale of each decimal column is fixed on created
(at runtime), so it fits `OobScaleFpdec`.
While in the client end, the application knows the business logical and
the scale of each decimal column ahead (at compilation time), so it fits
`ConstScaleFpdec`.


# How to Choose Your Number Type

Here are 2 questions:

Q1: Do you want *Binary*(base-2) or *Decimal*(base-10)?

*Binary* for machines, *Decimal* for humans.

If you want to accurately represent decimal fractions from the real world,
such as avoiding the issue where 0.1 + 0.2 != 0.3, then you should choose
*Decimal*.

Q2: Do you want *Floating-point* or *Fixed-point*?

*Floating-point* for flexible, *Fixed-point* for performance.

If you have no idea about the required decimal precision, use *Floating-point*.
For example, if you are implementing a general-purpose decimal math library,
the precision requirements of the end users are variable and unknown.

If you know the decimal precision, use *Fixed-point*. If it is known at
compile time, refer to the `ConstScaleFpdec` example above. If it is
known at runtime, refer to the `OobScaleFpdec` example above.

Generally speaking, use *Floating-point* for library, and *Fixed-point*
for application.

Then make a choice:

- Floating-point Binary: primitive `f32`, `f64`
- Fixed-point Binary: crate `fixed`
- Floating-point Decimal: crate `bigdecimal` and `rust_decimal`
- Fixed-point Decimal: __THIS__ crate `primitive_fixed_point_decimal` !!!


# License

MIT
