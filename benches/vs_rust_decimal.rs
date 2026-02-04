use criterion::{criterion_group, criterion_main, Criterion};

// primitive_fixed_point_decimal
use primitive_fixed_point_decimal::{fpdec, ConstScaleFpdec};
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
    group.bench_with_input("rust-dec-diff", &(rust_n1, rust_n5), |b, i| {
        b.iter(|| rust_add(i.0, i.1))
    });
    group.bench_with_input("rust-dec-diff-big", &(rust_n1, rust_n6), |b, i| {
        b.iter(|| rust_add(i.0, i.1))
    });
    group.bench_with_input("rust-dec-diff-huge", &(rust_n1, rust_n7), |b, i| {
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

// entry
fn criterion_benchmark(c: &mut Criterion) {
    // bench_int(c);
    bench_add(c);
    bench_mul_pure(c);
    bench_mul_rescale(c);
    bench_mul_rescale_big(c);
    bench_div_pure(c);
    bench_div_rescale(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
