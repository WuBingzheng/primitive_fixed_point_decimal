use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

// primitive_fixed_point_decimal
use primitive_fixed_point_decimal::{fpdec, ConstScaleFpdec};
type PrimDec64 = ConstScaleFpdec<i64, 6>;
type PrimDec64Mul = ConstScaleFpdec<i64, 12>;
type PrimDec64Div = ConstScaleFpdec<i64, 0>;

// add
fn prim64_add(a: PrimDec64, b: PrimDec64) -> PrimDec64 {
    a + b
}

// mul
fn prim64_mul_rescale(a: PrimDec64, b: PrimDec64) -> PrimDec64 {
    a * b
}
fn prim64_mul_pure(a: PrimDec64, b: PrimDec64) -> PrimDec64Mul {
    a.checked_mul(b).unwrap()
}

// div
fn prim64_div_rescale(a: PrimDec64, b: PrimDec64) -> PrimDec64 {
    a / b
}
fn prim64_div_pure(a: PrimDec64, b: PrimDec64) -> PrimDec64Div {
    a.checked_div(b).unwrap()
}

type PrimDec128 = ConstScaleFpdec<i128, 6>;
type PrimDec128Mul = ConstScaleFpdec<i128, 12>;
type PrimDec128Div = ConstScaleFpdec<i128, 0>;

// add
fn prim128_add(a: PrimDec128, b: PrimDec128) -> PrimDec128 {
    a + b
}

// mul
fn prim128_mul_rescale(a: PrimDec128, b: PrimDec128) -> PrimDec128 {
    a * b
}
fn prim128_mul_pure(a: PrimDec128, b: PrimDec128) -> PrimDec128Mul {
    a.checked_mul(b).unwrap()
}

// div
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
fn rust_mul_pure(a: RustDec, b: RustDec) -> RustDec {
    a * b
}
fn rust_div(a: RustDec, b: RustDec) -> RustDec {
    a / b
}

fn bench_add(c: &mut Criterion) {
    // make sure not const, so we do not need black_box()
    let prim64_n1: PrimDec64 = fpdec!(12.345678);
    let prim64_n2: PrimDec64 = fpdec!(34.567809);
    let prim128_n1: PrimDec128 = fpdec!(12.345678);
    let prim128_n2: PrimDec128 = fpdec!(34.567809);
    let rust_n1 = RustDec::from_str("12.345678").unwrap();
    let rust_n2 = RustDec::from_str("34.567809").unwrap();
    let rust_n5 = RustDec::from_str("12.34").unwrap();
    let rust_n6 = RustDec::from_scientific("1e12").unwrap();
    let rust_n7 = RustDec::from_scientific("1e28").unwrap();

    c.bench_function("prim-dec64:add", |b| {
        b.iter(|| prim64_add(prim64_n1, prim64_n2))
    });
    c.bench_function("prim-dec128:add", |b| {
        b.iter(|| prim128_add(prim128_n1, prim128_n2))
    });
    c.bench_function("rust-dec:add", |b| b.iter(|| rust_add(rust_n1, rust_n2)));
    c.bench_function("rust-dec:add-diff", |b| {
        b.iter(|| rust_add(rust_n1, rust_n5))
    });
    c.bench_function("rust-dec:add-diff-big", |b| {
        b.iter(|| rust_add(rust_n1, rust_n6))
    });
    c.bench_function("rust:add-diff-bigbig", |b| {
        b.iter(|| rust_add(rust_n1, rust_n7))
    });
}

fn bench_mul(c: &mut Criterion) {
    let prim64_n1: PrimDec64 = fpdec!(12.345678);
    let prim64_n2: PrimDec64 = fpdec!(34.567891);
    let prim128_n1: PrimDec128 = fpdec!(12.345678);
    let prim128_n2: PrimDec128 = fpdec!(34.567891);
    let rust_n1 = RustDec::from_str("12.345678").unwrap();
    let rust_n2 = RustDec::from_str("34.567891").unwrap();

    // mul with pure scale
    c.bench_function("prim64:mul-pure", |b| {
        b.iter(|| black_box(prim64_mul_pure(prim64_n1, prim64_n2)))
    });
    c.bench_function("prim128:mul-pure", |b| {
        b.iter(|| black_box(prim128_mul_pure(prim128_n1, prim128_n2)))
    });
    c.bench_function("rust:mul-pure", |b| {
        b.iter(|| black_box(rust_mul_pure(rust_n1, rust_n2)))
    });

    // mul with rescale scale
    c.bench_function("prim64:mul-rescale", |b| {
        b.iter(|| black_box(prim64_mul_rescale(prim64_n1, prim64_n2)))
    });
    c.bench_function("prim128:mul-rescale", |b| {
        b.iter(|| black_box(prim128_mul_rescale(prim128_n1, prim128_n2)))
    });

    // mul with rescale scale big
    let prim64_big_n1: PrimDec64 = fpdec!(1e6);
    let prim64_big_n2: PrimDec64 = fpdec!(1e6 + 1.0);
    let prim128_big_n1: PrimDec128 = fpdec!(1e16);
    let prim128_big_n2: PrimDec128 = fpdec!(1e16 + 1.0);
    let mut rust_big_n1 = RustDec::from_scientific("1e12").unwrap();
    rust_big_n1.rescale(6);
    let rust_big_n2 = rust_big_n1 + dec!(0.1);

    c.bench_function("prim64:mul-rescale-big", |b| {
        b.iter(|| black_box(prim64_mul_rescale(prim64_big_n1, prim64_big_n2)))
    });
    c.bench_function("prim128:mul-rescale-big", |b| {
        b.iter(|| black_box(prim128_mul_rescale(prim128_big_n1, prim128_big_n2)))
    });
    c.bench_function("rust:mul-rescale-big", |b| {
        b.iter(|| black_box(rust_mul_pure(rust_big_n1, rust_big_n2)))
    });
    // load
    // dump
}

// bench
fn criterion_benchmark(c: &mut Criterion) {
    bench_add(c);
    bench_mul(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
