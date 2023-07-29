# primitive_fixed_point_decimal

Primitive fixed-point decimal types.

It's necessary to represent decimals accurately in some scenarios,
such as financial field. Primitive floating-point types (`f32`
and `f64`) can not accurately represent decimal fractions because
they use binary to represent values. Here we use integer types to
represent values, and handle fractions in base 10.

Primitive integers `i16`, `i32`, `i64` and `i128` are used to represent
values, corresponding to `FixDec16<P>`, `FixDec32<P>`, `FixDec64<P>`,
and `FixDec128<P>` types, respectively, which can represent about 4,
9, 18 and 38 decimal significant digits.

In addition, these scenarios generally require *fraction precision*,
rather than the *significant digits* like in scientific calculations,
so fixed-point is more suitable than floating-point.

We use Rust's *const generics* to specify the precision. For example,
`FixDec16<2>` represents `2` decimal precision and its range represented
is `-327.68` ~ `327.67`.

## Characteristics

It is a common idea to use integers and const generics to represent
decimals. We have some specialties.

The `+`, `-` and comparison operations only perform between same types in
same precision. There is no implicitly type or precision conversion.
This makes sence. For example, if you use `FixDec64<2>` to represent
balance and `FixDec64<6>` to represent exchange rates, there should be
no above operations between balance `FixDec64<2>` and exchange rates
`FixDec64<6>`.

However, the `*` and `/` operations accept operand with different
precisions. Certainly we need to multiply between balance `FixDec64<2>`
and exchange rates `FixDec64<6>` to get another balance.

Besides, the `*` and `/` operations can specify the precision of the
results. For example, the product of balance and exchange rate is still
balance, which of another asset, so the result should be `FixDec64<2>`
too, but not `FixDec64<2+6>`. Another example, you want to get the
exchange rate `FixDec64<6>` by dividing two balance `FixDec64<2>`.

## Conversion

Meanwhile the conversion can be made explicitly.

Different types are converted into each other by `Into` and `TryInto`
trait. Use `Into` to convert from less-bit-type to more-bit-type, and
use `TryInto` for the opposite direction because it may overflow.
The conversion keeps the precision.

Different precisions of same type are converted into each other by
`rescale()` function.

### Features

- `serde` enables serde traits integration (`Serialize`/`Deserialize`)

## Example

Let's see an example of foreign exchange trading.

```rust
use std::str::FromStr;
use primitive_fixed_point_decimal::{FixDec64, FixDec16};

type Balance = FixDec64<2>;
type Price = FixDec64<6>;
type FeeRate = FixDec16<4>;

// I have 30000 USD and none CNY in my account at first.
let mut account_usd = Balance::from_str("30000").unwrap();
let mut account_cny = Balance::ZERO;

// I want to exchange 10000 USD to CNY at price 7.17, with 0.0015 fee-rate.
let pay_usd = Balance::from_str("10000").unwrap();
let price = Price::from_str("7.17").unwrap();
let fee_rate = FeeRate::from_str("0.0015").unwrap();

// Calculate the get_cny = pay_usd * price.
// Because `checked_mul()` accepts operand with different precision,
// it's not need to convert the `Price` from `FixDec64<8>` to `FixDec64<2>`.
// Besides we want get `Balance` as result, so it's need to declare the
// `get_cny` as `Balance` explicitly.
let get_cny: Balance = pay_usd.checked_mul(price).unwrap();

// Calculate the fee_cny = get_cny * fee_rate.
// Because `checked_mul()` accepts same type operand only, so the
// `FeeRate` is converted from `FixDec16<4>` into `FixDec64<4>`.
let fee_cny: Balance = get_cny.checked_mul(fee_rate.into()).unwrap();

// Update the trading result.
account_usd -= pay_usd;
account_cny += get_cny - fee_cny;

// Check the result:
//   USD = 20000 = 30000 - 10000
//   CNY = 71592.45 = 10000 * 7.17 - 10000 * 7.17 * 0.0015
assert_eq!(account_usd, Balance::from_str("20000").unwrap());
assert_eq!(account_cny, Balance::from_str("71592.45").unwrap());
```

## Status

More tests are need before ready for production.

License: MIT
