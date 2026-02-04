use criterion::{criterion_group, criterion_main, Criterion};
use std::fmt::Write;

// primitive_fixed_point_decimal
use primitive_fixed_point_decimal::{fpdec, ConstScaleFpdec};
type PrimPrice = ConstScaleFpdec<u32, 4>;
type PrimFeeRate = ConstScaleFpdec<u16, 6>;

type PrimDec64 = ConstScaleFpdec<i64, 6>;
type PrimDec64Mul = ConstScaleFpdec<i64, 12>;
type PrimDec64Div = ConstScaleFpdec<i64, 0>;

fn prim64_add(a: PrimDec64, b: PrimDec64) -> PrimDec64 {
    a + b
}
fn prim64_mul_rescale(a: PrimDec64, b: PrimDec64) -> PrimDec64 {
    a * b
}
fn prim64_mul_pure(a: PrimDec64, b: PrimDec64) -> PrimDec64Mul {
    a.checked_mul(b).unwrap()
}
fn prim64_div_rescale(a: PrimDec64, b: PrimDec64) -> PrimDec64 {
    a / b
}
fn prim64_div_pure(a: PrimDec64, b: PrimDec64) -> PrimDec64Div {
    a.checked_div(b).unwrap()
}
fn prim64_load(s: &str) -> PrimDec64 {
    PrimDec64::from_str(s).unwrap()
}
fn prim64_dump(a: PrimDec64, buf: &mut String) {
    buf.clear();
    write!(buf, "{a}").unwrap();
}

#[derive(Clone)]
struct Prim64AppAccount {
    base: PrimDec64,
    quota: PrimDec64,
}
#[derive(Clone)]
struct Prim64AppContext {
    bider: Prim64AppAccount,
    asker: Prim64AppAccount,
    sys_fee: Prim64AppAccount,
    fee_rate: PrimFeeRate,
}
fn prim64_app(ctx: &Prim64AppContext, amount: PrimDec64, price: PrimPrice) {
    let mut ctx = ctx.clone();

    let money = amount * price;
    let quota_fee = money * ctx.fee_rate;
    let base_fee = amount * ctx.fee_rate;

    ctx.bider.base += amount;
    ctx.bider.quota -= money;
    ctx.bider.base -= base_fee;
    ctx.sys_fee.base += base_fee;

    ctx.asker.base -= amount;
    ctx.asker.quota += money;
    ctx.asker.quota -= quota_fee;
    ctx.sys_fee.quota += quota_fee;

    std::hint::black_box(&ctx);
}

type PrimDec128 = ConstScaleFpdec<i128, 6>;
type PrimDec128Mul = ConstScaleFpdec<i128, 12>;
type PrimDec128Div = ConstScaleFpdec<i128, 0>;

fn prim128_add(a: PrimDec128, b: PrimDec128) -> PrimDec128 {
    a + b
}
fn prim128_mul_rescale(a: PrimDec128, b: PrimDec128) -> PrimDec128 {
    a * b
}
fn prim128_mul_pure(a: PrimDec128, b: PrimDec128) -> PrimDec128Mul {
    a.checked_mul(b).unwrap()
}
fn prim128_div_rescale(a: PrimDec128, b: PrimDec128) -> PrimDec128 {
    a / b
}
fn prim128_div_pure(a: PrimDec128, b: PrimDec128) -> PrimDec128Div {
    a.checked_div(b).unwrap()
}
fn prim128_load(s: &str) -> PrimDec128 {
    PrimDec128::from_str(s).unwrap()
}
fn prim128_dump(a: PrimDec128, buf: &mut String) {
    buf.clear();
    write!(buf, "{a}").unwrap();
}

#[derive(Clone)]
struct Prim128AppAccount {
    base: PrimDec128,
    quota: PrimDec128,
}
#[derive(Clone)]
struct Prim128AppContext {
    bider: Prim128AppAccount,
    asker: Prim128AppAccount,
    sys_fee: Prim128AppAccount,
    fee_rate: PrimFeeRate,
}
fn prim128_app(ctx: &Prim128AppContext, amount: PrimDec128, price: PrimPrice) {
    let mut ctx = ctx.clone();

    let money = amount * price;
    let quota_fee = money * ctx.fee_rate;
    let base_fee = amount * ctx.fee_rate;

    ctx.bider.base += amount;
    ctx.bider.quota -= money;
    ctx.bider.base -= base_fee;
    ctx.sys_fee.base += base_fee;

    ctx.asker.base -= amount;
    ctx.asker.quota += money;
    ctx.asker.quota -= quota_fee;
    ctx.sys_fee.quota += quota_fee;

    std::hint::black_box(&ctx);
}

// rust_decimal
use rust_decimal::prelude::*;
type RustDec = Decimal;

fn rust_add(a: RustDec, b: RustDec) -> RustDec {
    a + b
}
fn rust_mul(a: RustDec, b: RustDec) -> RustDec {
    a * b
}
fn rust_div(a: RustDec, b: RustDec) -> RustDec {
    a / b
}
fn rust_load(s: &str) -> RustDec {
    RustDec::from_str(s).unwrap()
}
fn rust_dump(a: RustDec, buf: &mut String) {
    buf.clear();
    write!(buf, "{a}").unwrap();
}

#[derive(Clone)]
struct RustAppAccount {
    base: RustDec,
    quota: RustDec,
}
#[derive(Clone)]
struct RustAppContext {
    bider: RustAppAccount,
    asker: RustAppAccount,
    sys_fee: RustAppAccount,
    fee_rate: RustDec,
}
fn rust_app(ctx: &RustAppContext, amount: RustDec, price: RustDec) {
    let mut ctx = ctx.clone();

    let money = amount * price;
    let quota_fee = money * ctx.fee_rate;
    let base_fee = amount * ctx.fee_rate;

    ctx.bider.base += amount;
    ctx.bider.quota -= money;
    ctx.bider.base -= base_fee;
    ctx.sys_fee.base += base_fee;

    ctx.asker.base -= amount;
    ctx.asker.quota += money;
    ctx.asker.quota -= quota_fee;
    ctx.sys_fee.quota += quota_fee;

    std::hint::black_box(&ctx);
}

// benches

fn _bench_int(c: &mut Criterion) {
    let mut group = c.benchmark_group("int-as-base");

    // int64
    group.bench_with_input("int64:add", &(1234567_i64, 3456789_i64), |b, i| {
        b.iter(|| i.0 + i.1)
    });
    group.bench_with_input("int64:mul", &(1234567_i64, 3456789_i64), |b, i| {
        b.iter(|| i.0 * i.1)
    });
    group.bench_with_input("int64:div", &(1234567_i64, 3456789_i64), |b, i| {
        b.iter(|| i.0 / i.1)
    });
    // int64
    group.bench_with_input("int128:add", &(1234567_i128, 3456789_i128), |b, i| {
        b.iter(|| i.0 + i.1)
    });
    group.bench_with_input("int128:mul", &(1234567_i128, 3456789_i128), |b, i| {
        b.iter(|| i.0 * i.1)
    });
    group.bench_with_input("int128:div", &(1234567_i128, 3456789_i128), |b, i| {
        b.iter(|| i.0 / i.1)
    });

    // done
    group.finish();
}

fn bench_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("add");

    // prim-dec64
    let prim64_n1: PrimDec64 = fpdec!(12.345678);
    let prim64_n2: PrimDec64 = fpdec!(34.567809);
    group.bench_with_input("prim-dec64", &(prim64_n1, prim64_n2), |b, i| {
        b.iter(|| prim64_add(i.0, i.1))
    });

    // prim-dec128
    let prim128_n1: PrimDec128 = fpdec!(12.345678);
    let prim128_n2: PrimDec128 = fpdec!(34.567809);
    group.bench_with_input("prim-dec128", &(prim128_n1, prim128_n2), |b, i| {
        b.iter(|| prim128_add(i.0, i.1))
    });

    // rust-dec
    let rust_n1 = dec!(12.345678);
    let rust_n2 = dec!(34.567809);
    let rust_n5 = dec!(12.3);
    let rust_n6 = dec!(1e12);
    let rust_n7 = dec!(1e28);
    group.bench_with_input("rust-dec", &(rust_n1, rust_n2), |b, i| {
        b.iter(|| rust_add(i.0, i.1))
    });
    group.bench_with_input("rust-dec-rescale", &(rust_n1, rust_n5), |b, i| {
        b.iter(|| rust_add(i.0, i.1))
    });
    group.bench_with_input("rust-dec-rescale-big", &(rust_n1, rust_n6), |b, i| {
        b.iter(|| rust_add(i.0, i.1))
    });
    group.bench_with_input("rust-dec-rescale-huge", &(rust_n1, rust_n7), |b, i| {
        b.iter(|| rust_add(i.0, i.1))
    });

    // done
    group.finish();
}

fn bench_mul_pure(c: &mut Criterion) {
    let mut group = c.benchmark_group("mul-pure");

    // prim-dec64
    let prim64_n1: PrimDec64 = fpdec!(12.345678);
    let prim64_n2: PrimDec64 = fpdec!(34.567809);
    group.bench_with_input("prim-dec64", &(prim64_n1, prim64_n2), |b, i| {
        b.iter(|| prim64_mul_pure(i.0, i.1))
    });

    // prim-dec128
    let prim128_n1: PrimDec128 = fpdec!(12.345678);
    let prim128_n2: PrimDec128 = fpdec!(34.567809);
    group.bench_with_input("prim-dec128", &(prim128_n1, prim128_n2), |b, i| {
        b.iter(|| prim128_mul_pure(i.0, i.1))
    });

    // rust-dec
    let rust_n1 = dec!(12.345678);
    let rust_n2 = dec!(34.567809);
    group.bench_with_input("rust-dec", &(rust_n1, rust_n2), |b, i| {
        b.iter(|| rust_mul(i.0, i.1))
    });

    // done
    group.finish();
}

fn bench_mul_rescale(c: &mut Criterion) {
    let mut group = c.benchmark_group("mul-rescale");

    // prim-dec64
    let prim64_n1: PrimDec64 = fpdec!(12.345678);
    let prim64_n2: PrimDec64 = fpdec!(34.567809);
    group.bench_with_input("prim-dec64", &(prim64_n1, prim64_n2), |b, i| {
        b.iter(|| prim64_mul_rescale(i.0, i.1))
    });

    // prim-dec128
    let prim128_n1: PrimDec128 = fpdec!(12.345678);
    let prim128_n2: PrimDec128 = fpdec!(34.567809);
    group.bench_with_input("prim-dec128", &(prim128_n1, prim128_n2), |b, i| {
        b.iter(|| prim128_mul_rescale(i.0, i.1))
    });

    // done
    group.finish();
}

fn bench_mul_rescale_big(c: &mut Criterion) {
    let mut group = c.benchmark_group("mul-rescale-big");

    // prim-dec64
    // (6 + 6) + (6 + 6) > i64::DIGITS
    let prim64_n1: PrimDec64 = fpdec!(1e6);
    let prim64_n2: PrimDec64 = fpdec!(1e6 + 1.0);
    group.bench_with_input("prim-dec64", &(prim64_n1, prim64_n2), |b, i| {
        b.iter(|| prim64_mul_rescale(i.0, i.1))
    });

    // prim-dec128
    // (16 + 6) + (16 + 6) > i128::DIGITS
    let prim128_n1: PrimDec128 = fpdec!(1e16);
    let prim128_n2: PrimDec128 = fpdec!(1e16 + 1.0);
    group.bench_with_input("prim-dec128", &(prim128_n1, prim128_n2), |b, i| {
        b.iter(|| prim128_mul_rescale(i.0, i.1))
    });

    // rust-dec
    // (12 + 6) + (12 + 6) > u96::DIGITS
    let mut rust_n1 = RustDec::from_scientific("1e12").unwrap();
    rust_n1.rescale(6);
    let rust_n2 = rust_n1 + dec!(0.1);

    group.bench_with_input("rust-dec", &(rust_n1, rust_n2), |b, i| {
        b.iter(|| rust_mul(i.0, i.1))
    });

    // done
    group.finish();
}

fn bench_div_pure(c: &mut Criterion) {
    let mut group = c.benchmark_group("div-pure");

    // prim-dec64
    let prim64_n1: PrimDec64 = fpdec!(12.3456);
    let prim64_n2: PrimDec64 = fpdec!(0.123456);
    group.bench_with_input("prim-dec64", &(prim64_n1, prim64_n2), |b, i| {
        b.iter(|| prim64_div_pure(i.0, i.1))
    });

    // prim-dec128
    let prim128_n1: PrimDec128 = fpdec!(12.3456);
    let prim128_n2: PrimDec128 = fpdec!(0.123456);
    group.bench_with_input("prim-dec128", &(prim128_n1, prim128_n2), |b, i| {
        b.iter(|| prim128_div_pure(i.0, i.1))
    });

    // rust-dec
    // n1 % n2 = 0
    let rust_n1 = dec!(12.345600);
    let rust_n2 = dec!(0.123456);
    group.bench_with_input("rust-dec", &(rust_n1, rust_n2), |b, i| {
        b.iter(|| rust_div(i.0, i.1))
    });

    // done
    group.finish();
}

fn bench_div_rescale(c: &mut Criterion) {
    let mut group = c.benchmark_group("div-rescale");

    // prim-dec64
    let prim64_n1: PrimDec64 = fpdec!(12.345678);
    let prim64_n2: PrimDec64 = fpdec!(34.567809);
    group.bench_with_input("prim-dec64", &(prim64_n1, prim64_n2), |b, i| {
        b.iter(|| prim64_div_rescale(i.0, i.1))
    });

    // prim-dec128
    let prim128_n1: PrimDec128 = fpdec!(12.345678);
    let prim128_n2: PrimDec128 = fpdec!(34.567809);
    group.bench_with_input("prim-dec128", &(prim128_n1, prim128_n2), |b, i| {
        b.iter(|| prim128_div_rescale(i.0, i.1))
    });

    // rust-dec
    // n1 % n2 != 0
    let rust_n1 = dec!(12.345678);
    let rust_n2 = dec!(34.567809);
    group.bench_with_input("rust-dec", &(rust_n1, rust_n2), |b, i| {
        b.iter(|| rust_div(i.0, i.1))
    });

    // done
    group.finish();
}

fn bench_load(c: &mut Criterion, name: &str, input: &str) {
    let mut group = c.benchmark_group(name);

    // prim-dec64
    group.bench_with_input("prim-dec64", &input, |b, i| b.iter(|| prim64_load(i)));
    // prim-dec128
    group.bench_with_input("prim-dec128", &input, |b, i| b.iter(|| prim128_load(i)));
    // rust-dec
    group.bench_with_input("rust-dec", &input, |b, i| b.iter(|| rust_load(i)));

    // done
    group.finish();
}

fn bench_dump(c: &mut Criterion, name: &str, input: &str) {
    let mut group = c.benchmark_group(name);

    let mut buf = String::with_capacity(200);

    // prim-dec64
    let prim64_n1 = PrimDec64::from_str(input).unwrap();
    group.bench_with_input("prim-dec64", &prim64_n1, |b, i| {
        b.iter(|| prim64_dump(*i, &mut buf))
    });
    // prim-dec128
    let prim128_n1 = PrimDec128::from_str(input).unwrap();
    group.bench_with_input("prim-dec128", &prim128_n1, |b, i| {
        b.iter(|| prim128_dump(*i, &mut buf))
    });
    // rust-dec
    let rust_n1 = RustDec::from_str(input).unwrap();
    group.bench_with_input("rust-dec", &rust_n1, |b, i| {
        b.iter(|| rust_dump(*i, &mut buf))
    });

    // done
    group.finish();
}

fn bench_app(c: &mut Criterion) {
    let mut group = c.benchmark_group("app");

    // prim-dec64
    let account = Prim64AppAccount {
        base: fpdec!(10000),
        quota: fpdec!(10000),
    };
    let ctx = Prim64AppContext {
        bider: account.clone(),
        asker: account.clone(),
        sys_fee: account,
        fee_rate: fpdec!(0.000012),
    };
    let amount: PrimDec64 = fpdec!(56.789);
    let price: PrimPrice = fpdec!(1234.5);
    group.bench_with_input("prim-dec64", &(amount, price), |b, i| {
        b.iter(|| prim64_app(&ctx, i.0, i.1))
    });

    // prim-dec128
    let account = Prim128AppAccount {
        base: fpdec!(10000),
        quota: fpdec!(10000),
    };
    let ctx = Prim128AppContext {
        bider: account.clone(),
        asker: account.clone(),
        sys_fee: account,
        fee_rate: fpdec!(0.000012),
    };
    let amount: PrimDec128 = fpdec!(56.789);
    let price: PrimPrice = fpdec!(1234.5);
    group.bench_with_input("prim-dec128", &(amount, price), |b, i| {
        b.iter(|| prim128_app(&ctx, i.0, i.1))
    });

    // rust-dec
    let account = RustAppAccount {
        base: dec!(10000),
        quota: dec!(10000),
    };
    let ctx = RustAppContext {
        bider: account.clone(),
        asker: account.clone(),
        sys_fee: account,
        fee_rate: dec!(0.000012),
    };
    let amount: RustDec = dec!(56.789);
    let price: RustDec = dec!(1234.5);
    group.bench_with_input("rust-dec", &(amount, price), |b, i| {
        b.iter(|| rust_app(&ctx, i.0, i.1))
    });

    // done
    group.finish();
}

// entry
fn criterion_benchmark(c: &mut Criterion) {
    // bench_int(c);
    bench_add(c);
    bench_mul_pure(c);
    bench_mul_rescale(c);
    bench_mul_rescale_big(c);
    bench_div_pure(c);
    bench_div_rescale(c);
    bench_load(c, "load-short", "12.34");
    bench_load(c, "load-long", "123456789012.123456");
    bench_dump(c, "dump-short", "12.34");
    bench_dump(c, "dump-long", "123456789012.123456");
    bench_app(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
