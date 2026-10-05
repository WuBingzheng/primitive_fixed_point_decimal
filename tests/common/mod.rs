//! Shared test machinery: an arbitrary-precision ORACLE and generators for the
//! boundary values that actually break fixed-point arithmetic.
//!
//! The oracle is deliberately written differently from the library under test:
//! it never splits a product into words, never uses `i128`, and never shares a
//! code path with `FpdecInner`. Everything is a `BigInt` and the rounding rule is
//! written out from the definition of each `Rounding` mode. If the library and
//! the oracle agree on millions of cases, that is evidence; if they were written
//! the same way, it would only be evidence that they share a bug.
#![allow(dead_code)]

use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};
use primitive_fixed_point_decimal::{FpdecInner, Rounding};

pub const ROUNDINGS: [Rounding; 5] = [
    Rounding::Round,
    Rounding::Floor,
    Rounding::Ceiling,
    Rounding::TowardsZero,
    Rounding::AwayFromZero,
];

pub fn pow10(n: u32) -> BigInt {
    BigInt::from(10u8).pow(n)
}

/// `n / d` rounded per `r`, from the definition. `d` must be non-zero.
///
/// `Round` is half-away-from-zero: the library documents "towards the nearest
/// integer" and every tie in its own tests goes away from zero.
pub fn round_div(n: &BigInt, d: &BigInt, r: Rounding) -> BigInt {
    assert!(!d.is_zero());
    let q = n / d; // BigInt `/` truncates toward zero
    let rem = n - &q * d; // same sign as `n`
    if rem.is_zero() {
        return q;
    }
    let neg = n.is_negative() != d.is_negative(); // sign of the exact quotient
    let away = if neg { &q - 1 } else { &q + 1 };
    match r {
        Rounding::TowardsZero => q,
        Rounding::AwayFromZero => away,
        Rounding::Floor => {
            if neg {
                away
            } else {
                q
            }
        }
        Rounding::Ceiling => {
            if neg {
                q
            } else {
                away
            }
        }
        Rounding::Round => {
            // |rem| * 2 >= |d|  <=>  fraction >= 1/2, ties away from zero
            if rem.abs() * 2 >= d.abs() {
                away
            } else {
                q
            }
        }
    }
}

/// An inner integer type, viewed through the oracle.
pub trait Int: FpdecInner + Copy + core::fmt::Debug + core::fmt::Display + PartialEq {
    const NAME: &'static str;
    const BITS: u32;
    const SIGNED: bool;
    fn big(self) -> BigInt;
    fn from_big(b: &BigInt) -> Option<Self>;
    fn min_big() -> BigInt {
        Self::MIN.big()
    }
    fn max_big() -> BigInt {
        Self::MAX.big()
    }
}

macro_rules! int_impl {
    ($t:ty, $signed:expr, $wide:ident) => {
        impl Int for $t {
            const NAME: &'static str = stringify!($t);
            const BITS: u32 = <$t>::BITS;
            const SIGNED: bool = $signed;
            fn big(self) -> BigInt {
                BigInt::from(self)
            }
            fn from_big(b: &BigInt) -> Option<Self> {
                b.$wide().and_then(|x| <$t>::try_from(x).ok())
            }
        }
    };
}
int_impl!(i8, true, to_i128);
int_impl!(i16, true, to_i128);
int_impl!(i32, true, to_i128);
int_impl!(i64, true, to_i128);
int_impl!(i128, true, to_i128);
int_impl!(u8, false, to_u128);
int_impl!(u16, false, to_u128);
int_impl!(u32, false, to_u128);
int_impl!(u64, false, to_u128);
int_impl!(u128, false, to_u128);

/// Deterministic xorshift64*. No `rand` dependency; a failing seed reproduces.
pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    pub fn u128(&mut self) -> u128 {
        (self.next() as u128) << 64 | self.next() as u128
    }
    /// A value of the type with a UNIFORM bit length (so small, medium and huge
    /// magnitudes are equally likely) and a random sign.
    pub fn int<T: Int>(&mut self) -> T {
        let bits = self.below(T::BITS as u64 + 1) as u32;
        let mag = if bits == 0 { 0 } else { self.u128() >> (128 - bits) };
        let b = if T::SIGNED && self.next() & 1 == 1 {
            -BigInt::from(mag)
        } else {
            BigInt::from(mag)
        };
        // Clamp into range (only the MIN edge can fall outside).
        T::from_big(&b).unwrap_or(T::MIN)
    }
}

/// Values that sit exactly on the edges of the algorithms: powers of 10 and
/// their neighbours, half-way ties, powers of two and their neighbours, the
/// type's own limits, and square-root-of-range products.
pub fn interesting<T: Int>() -> Vec<T> {
    let (lo, hi) = (T::min_big(), T::max_big());
    let mut set: Vec<BigInt> = Vec::new();
    let mut push = |b: BigInt| {
        for v in [b.clone(), -b] {
            if v >= lo && v <= hi {
                set.push(v);
            }
        }
    };
    for k in 0..=2u32 {
        push(BigInt::from(k));
    }
    for v in [3, 5, 7, 9, 11, 49, 50, 51, 99, 101, 127, 128, 129, 255, 256, 257] {
        push(BigInt::from(v));
    }
    for k in 0..=(T::DIGITS + 1) {
        let p = pow10(k);
        for d in -2..=2i32 {
            push(&p + d);
            push(&p * 5 + d); // 5 * 10^k: the half-way tie for 10^(k+1)
            push(&p * 3 + d);
        }
    }
    for k in 0..T::BITS {
        let p = BigInt::from(1u8) << k;
        for d in -2..=2i32 {
            push(&p + d);
        }
    }
    // squares: products of two of these straddle the 2^(BITS) boundary
    let h = T::BITS / 2;
    for d in -3..=3i32 {
        push((BigInt::from(1u8) << h) + d);
        push((BigInt::from(1u8) << (h.saturating_sub(1))) + d);
    }
    for d in 0..=3i32 {
        push(&hi - d);
        push(&hi / 2 + d);
        push(&hi / 3 + d);
        push(&hi / 10 + d);
    }
    set.push(lo.clone());
    set.push(&lo + 1);
    set.push(&lo + 2);
    set.push(&lo / 2);
    set.push(&lo / 2 + 1);
    set.push(BigInt::from(0));
    set.sort();
    set.dedup();
    set.into_iter()
        .filter(|v| *v >= lo && *v <= hi)
        .filter_map(|v| T::from_big(&v))
        .collect()
}

/// The `diff_scale` values worth trying for a type: everything near zero and
/// the edges of `[-DIGITS, DIGITS]`, plus just outside it.
pub fn diffs<T: Int>() -> Vec<i32> {
    let d = T::DIGITS as i32;
    let mut v: Vec<i32> = (-(d + 1)..=(d + 1)).collect();
    if d > 6 {
        v = vec![
            -(d + 1),
            -d,
            -(d - 1),
            -(d / 2 + 1),
            -(d / 2),
            -3,
            -2,
            -1,
            0,
            1,
            2,
            3,
            d / 2,
            d / 2 + 1,
            d - 1,
            d,
            d + 1,
        ];
    }
    v.sort();
    v.dedup();
    v
}

/// Pairs `(a, b)` whose PRODUCT lands next to `2^k` for each machine-word
/// boundary `k`, because every fast path in the library is guarded by a
/// leading-zeros test on the operands and an overflow of that test is exactly
/// where a wrong answer would hide.
pub fn product_edge_pairs<T: Int>(rng: &mut Rng, per_edge: usize) -> Vec<(T, T)> {
    let mut out = Vec::new();
    let mut ks: Vec<u32> = vec![31, 32, 61, 62, 63, 64, 65, 125, 126, 127, 128];
    ks.retain(|&k| k <= 2 * T::BITS);
    for k in ks {
        let target = BigInt::from(1u8) << k;
        for _ in 0..per_edge {
            let a: T = rng.int::<T>();
            let a_abs = a.big().abs();
            if a_abs.is_zero() {
                continue;
            }
            for delta in -2..=2i32 {
                let b_abs = &target / &a_abs + delta;
                for sa in [1i32, -1] {
                    for sb in [1i32, -1] {
                        if let (Some(x), Some(y)) =
                            (T::from_big(&(&a_abs * sa)), T::from_big(&(&b_abs * sb)))
                        {
                            out.push((x, y));
                        }
                    }
                }
            }
        }
    }
    out
}

/// Pairs whose product is an exact HALF-WAY tie for `10^i` for every `i`:
/// `a = 5 * 10^(i-1) * k` with `k` and `b` odd, so `a * b mod 10^i == 5 * 10^(i-1)`.
pub fn tie_pairs<T: Int>(rng: &mut Rng, per_i: usize) -> Vec<(T, T)> {
    let mut out = Vec::new();
    for i in 1..=T::DIGITS {
        let half = pow10(i - 1) * 5;
        for _ in 0..per_i {
            let k = BigInt::from(2 * rng.below(20) + 1); // odd, so k*b stays odd
            let b = BigInt::from(rng.u128() >> 64 | 1); // odd
            for (sa, sb) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
                if let (Some(x), Some(y)) = (
                    T::from_big(&(&half * &k * sa)),
                    T::from_big(&(&b * sb)),
                ) {
                    out.push((x, y));
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The specification. Each function says what the DOCUMENTED contract returns.
// ---------------------------------------------------------------------------

/// `checked_mul_ext`: `a * b / 10^diff`, `None` on overflow or if `|diff|` is
/// outside `[-DIGITS, DIGITS]` (documented: "or the scale difference is out of range").
pub fn spec_mul<T: Int>(a: T, b: T, diff: i32, r: Rounding) -> Option<T> {
    if diff.unsigned_abs() > T::DIGITS {
        return None;
    }
    let p = a.big() * b.big();
    let q = if diff > 0 { round_div(&p, &pow10(diff as u32), r) } else { p * pow10(diff.unsigned_abs()) };
    T::from_big(&q)
}

/// `checked_div_ext`: `a / b * 10^-diff`; `None` on `b == 0`, overflow or bad `diff`.
pub fn spec_div<T: Int>(a: T, b: T, diff: i32, r: Rounding) -> Option<T> {
    if b.big().is_zero() || diff.unsigned_abs() > T::DIGITS {
        return None;
    }
    let q = if diff > 0 {
        round_div(&a.big(), &(b.big() * pow10(diff as u32)), r)
    } else {
        round_div(&(a.big() * pow10(diff.unsigned_abs())), &b.big(), r)
    };
    T::from_big(&q)
}

/// `a * b` at scale difference `diff`, in the type of `a`: the scale difference is bounded by
/// that type, the value by its range.
pub fn spec_mul_mixed<I: Int, J: Int>(a: I, b: J, diff: i32, r: Rounding) -> Option<I> {
    if diff.unsigned_abs() > I::DIGITS {
        return None;
    }
    let p = a.big() * b.big();
    I::from_big(&if diff > 0 { round_div(&p, &pow10(diff as u32), r) } else { p * pow10(diff.unsigned_abs()) })
}

pub fn spec_div_mixed<I: Int, J: Int>(a: I, b: J, diff: i32, r: Rounding) -> Option<I> {
    if b.big() == 0.into() || diff.unsigned_abs() > I::DIGITS {
        return None;
    }
    I::from_big(&if diff > 0 {
        round_div(&a.big(), &(b.big() * pow10(diff as u32)), r)
    } else {
        round_div(&(a.big() * pow10(diff.unsigned_abs())), &b.big(), r)
    })
}

pub fn spec_rounding_div<T: Int>(a: T, b: T, r: Rounding) -> Option<T> {
    if b.big().is_zero() {
        return None;
    }
    T::from_big(&round_div(&a.big(), &b.big(), r))
}

pub fn spec_ratio<T: Int>(a: T, n: T, d: T, r: Rounding) -> Option<T> {
    if d.big().is_zero() {
        return None;
    }
    T::from_big(&round_div(&(a.big() * n.big()), &d.big(), r))
}

/// `checked_round_diff_ext`: drop the `diff` lowest decimal digits, rounding per `r`; the
/// result keeps its scale, so it is `round(a / 10^diff) * 10^diff`. `None` if THAT
/// does not fit (e.g. `i8` 127 rounded up to a multiple of 1000). `diff <= 0` has
/// nothing to drop. Valid for any `diff`: `10^diff` is a `BigInt`.
pub fn spec_round<T: Int>(a: T, diff: i32, r: Rounding) -> Option<T> {
    if diff <= 0 {
        return Some(a);
    }
    let p = pow10(diff as u32);
    T::from_big(&(round_div(&a.big(), &p, r) * p))
}

/// The same for a `diff` so large that `10^diff` cannot be built (`i32::MAX`):
/// every non-zero mantissa is below one unit, so it rounds to 0 or to 10^diff.
pub fn spec_round_huge<T: Int>(a: T, r: Rounding) -> Option<T> {
    let neg = a.big().is_negative();
    let to_unit = match r {
        Rounding::TowardsZero | Rounding::Round => false,
        Rounding::AwayFromZero => true,
        Rounding::Floor => neg,
        Rounding::Ceiling => !neg,
    };
    if a.big().is_zero() || !to_unit {
        Some(T::from_big(&BigInt::from(0)).unwrap())
    } else {
        None
    }
}

/// `spec_round` for any `diff`, including ones too big to build `10^diff` from.
pub fn spec_round_any<T: Int>(a: T, diff: i64, r: Rounding) -> Option<T> {
    if diff <= 0 {
        Some(a)
    } else if diff <= 400 {
        spec_round(a, diff as i32, r)
    } else {
        spec_round_huge(a, r)
    }
}

/// `C<I, S>::try_from(v)`: `v * 10^S` must be an integer (precision) and fit (overflow).
/// Zero is representable at every scale.
pub fn spec_from_int<I: Int>(v: &BigInt, s: i32) -> Result<BigInt, primitive_fixed_point_decimal::ParseError> {
    if v.is_zero() {
        return Ok(v.clone());
    }
    let m = if s >= 0 {
        v * pow10(s as u32)
    } else {
        let p = pow10(s.unsigned_abs());
        if !(v % &p).is_zero() {
            return Err(primitive_fixed_point_decimal::ParseError::Precision);
        }
        v / p
    };
    if I::from_big(&m).is_some() { Ok(m) } else { Err(primitive_fixed_point_decimal::ParseError::Overflow) }
}

// ---------------------------------------------------------------------------
// Failure aggregation. A single run should show the SCOPE of a bug (how many
// inputs, which types, which roundings), not just the first assertion to trip.
// ---------------------------------------------------------------------------

use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Once;

thread_local!(static QUIET: Cell<bool> = const { Cell::new(false) });

/// Silences the panic message ONLY while this thread is inside `guarded`, so a
/// library panic is recorded as a failure instead of flooding stderr, while a
/// genuine test assertion elsewhere still prints normally.
fn install_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !QUIET.with(|q| q.get()) {
                prev(info)
            }
        }));
    });
}

/// Run `f`, turning a panic into `Err(())`. Debug builds panic on arithmetic
/// overflow and on `debug_assert!`s (including those inside dependencies); a
/// release build wraps silently instead. Either is a defect when the contract
/// is "return `None`", so both must be visible.
pub fn guarded<R>(f: impl FnOnce() -> R) -> Result<R, ()> {
    install_hook();
    QUIET.with(|q| q.set(true));
    let r = catch_unwind(AssertUnwindSafe(f));
    QUIET.with(|q| q.set(false));
    r.map_err(|_| ())
}

#[derive(Default)]
pub struct Failures {
    pub count: usize,
    pub shown: Vec<String>,
    pub by_kind: std::collections::BTreeMap<String, usize>,
}

impl Failures {
    /// `kind` groups failures ("mul i=19 Ceiling"); `detail` is built lazily so
    /// passing cases cost nothing.
    pub fn check<X: PartialEq + core::fmt::Debug>(
        &mut self,
        kind: impl FnOnce() -> String,
        detail: impl FnOnce() -> String,
        got: Result<X, ()>,
        want: X,
    ) {
        let ok = matches!(&got, Ok(g) if *g == want);
        if ok {
            return;
        }
        self.count += 1;
        let kind = kind();
        let n = self.by_kind.entry(kind.clone()).or_insert(0);
        *n += 1;
        if *n <= 3 && self.shown.len() < 40 {
            let got = match got {
                Ok(g) => format!("{g:?}"),
                Err(()) => "PANIC".to_string(),
            };
            self.shown
                .push(format!("[{kind}] {}  got {got}, want {want:?}", detail()));
        }
    }

    pub fn finish(self, what: &str) {
        if self.count == 0 {
            return;
        }
        eprintln!("\n{what}: {} FAILURES", self.count);
        for (k, n) in &self.by_kind {
            eprintln!("  {n:>7} x {k}");
        }
        eprintln!("first examples:");
        for s in &self.shown {
            eprintln!("  {s}");
        }
        panic!("{what}: {} oracle mismatches (see stderr)", self.count);
    }
}
