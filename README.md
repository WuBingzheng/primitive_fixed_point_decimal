# primitive_fixed_point_decimal

Primitive fixed-point decimal types.

It's necessary to represent decimals accurately in some scenarios,
such as financial field. Primitive floating-point types (`f32`
and `f64`) can not accurately represent decimal fractions because
they represent values in binary. Here we use integer types to
represent values, and handle fractions in base 10.

Primitive integers `i8`, `i16`, `i32`, `i64` and `i128` are used to represent
values, which can represent about 2, 4, 9, 18 and 38 decimal significant
digits respectively. So the number `12.345` is stored as `123450`
for decimal with precision `4`. See below to find how to specify
the precision.

In addition, these scenarios generally require *fraction precision*,
rather than the *significant digits* like in scientific calculations,
so fixed-point is more suitable than floating-point.

So here are the *primitive fixed-point* decimal types.

## Specify Precision

There are 2 ways to specify the precision: *static* and *out-of-band*:

- For the *static* type, [`StaticPrecFpdec`], we use Rust's *const generics*
  to specify the precision. For example, `StaticPrecFpdec<i64, 4>` means
  4 precision.

- For the *out-of-band* type, [`OobPrecFpdec`], we do NOT save the
  precision with our decimal types, so it's your job to save it somewhere
  and apply it in the following operations later. For example,
  `OobPrecFpdec<i64>` takes no precision information.

Generally, the *static* type is more convenient and suitable for most
scenarios. For example, in traditional currency exchange, you can use
`StaticPrecFpdec<i64, 2>` to represent balance, e.g. `1234.56` USD and
`8888800.00` JPY. And use `StaticPrecFpdec<i32, 6>` to represent all
market prices since 6-digit-precision is big enough for all currency
pairs, e.g. `146.4730` JPY/USD and `0.006802` USD/JPY:

```rust
use primitive_fixed_point_decimal::StaticPrecFpdec;
type Balance = StaticPrecFpdec<i64, 2>;
type Price = StaticPrecFpdec<i32, 6>; // 6 is big enough for all markets

let usd = Balance::try_from(1234.56).unwrap();
let price = Price::try_from(146.4730).unwrap();

let jpy: Balance = usd.checked_mul(price).unwrap();
assert_eq!(jpy, Balance::try_from(180829.70688).unwrap());
```

However in some scenarios, such as in cryptocurrency exchange, the
price differences across various markets are very significant. For
example `81234.0` in BTC/USDT and `0.000004658` in PEPE/USDT. Here
we need to select different precisions for each market. So it's
the *Out-of-band* type:

```rust
use primitive_fixed_point_decimal::OobPrecFpdec;
type Balance = OobPrecFpdec<i64>;
type Price = OobPrecFpdec<i32>; // no precision set

struct Market {
    base_asset_precision: i32,
    quote_asset_precision: i32,
    price_precision: i32,
}

let btc_usdt = Market {
    base_asset_precision: 8,
    quote_asset_precision: 6,
    price_precision: 1,
};

// we need tell the precisions to `try_from_float()` method
let btc = Balance::try_from_float(0.34, btc_usdt.base_asset_precision).unwrap();
let price = Price::try_from_float(81234.0, btc_usdt.price_precision).unwrap();

// we need tell the precision difference to `checked_mul()` method
let diff = btc_usdt.base_asset_precision + btc_usdt.price_precision - btc_usdt.quote_asset_precision;
let usdt = btc.checked_mul(price, diff).unwrap();
assert_eq!(usdt, Balance::try_from_float(27619.56, btc_usdt.quote_asset_precision).unwrap());
```

Obviously it's verbose to use, but offers greater flexibility.

You can even use the 2 types at same time. For example, use *out-of-band*
type for balance which have different precisions for different assets; and
use *static* type for fee-rate which has a fixed precision:

```rust
use primitive_fixed_point_decimal::{StaticPrecFpdec, OobPrecFpdec};
type Balance = OobPrecFpdec<i64>; // out-of-band type
type FeeRate = StaticPrecFpdec<i16, 6>; // static type

let btc_precision = 8;

let btc = Balance::try_from_float(0.34, btc_precision).unwrap();
let fee_rate = FeeRate::try_from(0.0002).unwrap();

let fee = btc.checked_mul_static(fee_rate).unwrap();
assert_eq!(fee, Balance::try_from_float(0.000068, btc_precision).unwrap());
```

## Characteristics

It is a common idea to use integers to represent decimals. But we have
some specialties.

The `+`, `-` and comparison operations only perform between same types in
same precision. There is no implicitly type or precision conversion.
This makes sence. For example, you do not want to add balance type by
exchange rate type. This also makes the operations very fast.

However, the `*` and `/` operations accept operand with different
precisions. Certainly we need to multiply between balance type
and exchange rates type.

## Features

- `serde` enables serde traits integration (`Serialize`/`Deserialize`)
  for *static* precision type. While the *out-of-band* type does not
  support serde at all.

## Status

More tests are need before ready for production.

License: MIT
