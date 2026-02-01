Primitive fixed-point decimal types.

Floating-point for flexibility, fixed-point for efficiency. We are fixed-point
by binding scale to *type* but not each value.

Binary for machines, decimal for humans. We represent and calculate
decimal fractions accurately by scaling integers in base-10.

For example, `ConstScaleFpdec<i64, 4>` means using `i64` as the underlying
representation, and the static scale is `4`.


# Features

Important:

- The `+` and `-` operations only perform between same types in same scale.
  There is no implicitly type or scale conversion. This makes sense, for we
  do not want to add `Balance` type by `Price` type.

- The `*` and `/` operations accept operand with different types and scales,
  and allow the result's scale specified. Certainly we need to multiply
  between `Balance` type and `Price` type.

- Supports 2 ways to specify the scale: *const* and *out-of-band*. See
  the [Specify Scale](#specify-scale) section for details.

- Supports both signed and unsigned types.

Less important, yet might also be what you need:

- Supports scale larger than the significant digits of the underlying integer
  type. For example `ConstScaleFpdec<i8, 4>` represents numbers in range
  [-0.0128, 0.0127].

- Supports negative scale. For example `ConstScaleFpdec<i8, -2>` represents
  numbers in range [-12800, 12700] with step 100.

- Supports serde traits integration (`Serialize`/`Deserialize`) by optional
  `serde` feature flag.

- `no-std` and `no-alloc`.


# Usage

Here we take `ConstScaleFpdec` as example. The other type `OobScaleFpdec`
is similar.

There are several ways to construct the decimal:

```rust
use primitive_fixed_point_decimal::{ConstScaleFpdec, fpdec};

// We choose `i64` as the underlying integer, and keep `6` precision.
type Balance = ConstScaleFpdec<i64, 6>;

// From float number.
let b = Balance::try_from(12.345).unwrap();
assert_eq!(b.to_string(), "12.345");

// From integer number.
let b = Balance::try_from(123).unwrap();
assert_eq!(b.to_string(), "123");

// This macro wraps above 2 TryFrom methods. It panics if fail in convert.
let _b1: Balance = fpdec!(12.345);
let _b2: Balance = fpdec!(123);

// From string.
use std::str::FromStr;
let b = Balance::from_str("12.345").unwrap();
assert_eq!(b.to_string(), "12.345");

// From mantissa, which is the underlying integer.
// This is low-level, but also the only `const` construction method.
const TWENTY: Balance = Balance::from_mantissa(20_000_000);
assert_eq!(TWENTY, fpdec!(20));
```

Addition and substraction operations only perform between same types in
same scale. There is no implicitly type or scale conversion. This make
them super fast, roughly equivalent to one single CPU instruction.

```rust
use primitive_fixed_point_decimal::{ConstScaleFpdec, fpdec};
type Balance = ConstScaleFpdec<i64, 6>;

let b1: Balance = fpdec!(12.345);
let b2: Balance = fpdec!(0.001);

assert_eq!(b1 + b2, fpdec!(12.346));

// If you want to check the overflow, use `checked_add()`:
assert_eq!(b1.checked_add(b2), Some(fpdec!(12.346)));
```

Multiplication and division operations accept operand with different types
and scales, and allow the result's scale specified.

```rust
use primitive_fixed_point_decimal::{ConstScaleFpdec, fpdec, Rounding};
type Balance = ConstScaleFpdec<i64, 6>;
type Price = ConstScaleFpdec<u32, 4>; // different integer type and precision

let stock: Balance = fpdec!(12.345);
let price: Price = fpdec!(10);

// `money` has the same type.
let money = stock * price;
assert_eq!(money, fpdec!(123.45));

// If you want to check overflow, or want to specify new decimal type for
// the result, use `checked_mul()`:
type AnotherBalance = ConstScaleFpdec<i64, 10>; // longer precision
let price: Price = fpdec!(1e-5);
let money: AnotherBalance = stock.checked_mul(price).unwrap();
assert_eq!(money, fpdec!(0.00012345));

// Multiplication operations can result in loss of precision. The default
// behavior is round, though custom rounding strategies are supported by
// `*_ext()` methods:
let money: Balance = stock.checked_mul_ext(price, Rounding::Ceiling).unwrap();
assert_eq!(money, fpdec!(0.000124));

// Also multiply by integer:
let double = stock.checked_mul_int(2).unwrap();
assert_eq!(double, fpdec!(12.345 * 2.0));

// However can not by float numbers.
```

Decimal can be converted into some types:

```rust
use primitive_fixed_point_decimal::{ConstScaleFpdec, fpdec};
type Balance = ConstScaleFpdec<i64, 6>;
let b: Balance = fpdec!(12.345);

// Format into string.
assert_eq!(format!("{:+.4}", b), "+12.3450");

// Convert into float numbers.
let f: f32 = b.into();
assert_eq!(f, 12.345);

// Get the mantissa, which is the underlying integer.
let m = b.mantissa();
assert_eq!(m, 12_345_000);
```


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

In summary,

- if you know the scale (decimal precision) at compile time, choose [`ConstScaleFpdec`];
- if you know it at runtime, choose [`OobScaleFpdec`];
- if you have no idea about it (maybe because the scale is variable rather
  than fixed, e.g. in a general-purpose decimal math library), you need a
  *floating-point* decimal crate, such as `bigdecimal` or `rust_decimal`.

You can also use these two types in combination.
[For example](OobScaleFpdec::checked_mul_const_scale_ext),
use `OobScaleFpdec` as Balance and `ConstScaleFpdec` as FeeRate.


# License

MIT
