//! Parsing and formatting against an independent oracle, for all ten inner types.
//!
//! `tests/oracle.rs` covers the arithmetic. The text side had no differential
//! test at all: the in-crate tests are a hundred hand-picked strings. Here:
//!
//! * `spec_parse` re-derives what a string means from the grammar
//!   (`[+-]? digits ('.' digits)?`) with `BigInt`, never calling the library.
//! * `spec_display` renders a mantissa at a scale with `BigInt` arithmetic and
//!   `round_div`, never calling the library.
//! * strings are generated from boundary values (MAX/MIN and their neighbours,
//!   powers of ten, ties) and then mangled (signs, leading/trailing zeros,
//!   truncation), because the bugs live at the seams of the grammar.
//!
//! ## Parse policy the spec pins down
//!
//! * Any spelling of zero is zero, at every scale: "0", "-0", "+00", "0.000".
//!   (Also "-0" for unsigned types.)
//! * `try_from_str` is TEXTUAL about precision: "1.50" at scale 1 is
//!   `Precision` although 1.5 fits, while `OobFmt::rescale` is SEMANTIC (it
//!   accepts it). That inconsistency is pre-existing and is pinned, not fixed,
//!   in `trailing_zero_policy_is_textual`.
//! * When a string is both too precise and too big the error kind is not
//!   specified, so the spec allows either.
mod common;
use common::*;

use core::num::ParseIntError;
use num_bigint::BigInt;
use num_traits::{Num, Signed, Zero};
use primitive_fixed_point_decimal::{OobFmt, OobScaleFpdec as D, ParseError, Rounding};

fn scale_mult() -> usize {
    std::env::var("PFPD_ORACLE_SCALE").ok().and_then(|s| s.parse().ok()).unwrap_or(1)
}

// ---------------------------------------------------------------------------
// Parse specification
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Parsed {
    Ok { n: BigInt, raw: i64 }, // mantissa at its own textual scale
    Err(Vec<ParseError>),       // allowed error kinds
}

/// What the string denotes: mantissa `n` and the scale `raw` it is written at
/// (fraction digit count; for an integer string, minus its trailing zeros).
fn spec_lex<T: Int>(s: &str) -> Parsed {
    if s.is_empty() {
        return Parsed::Err(vec![ParseError::Empty]);
    }
    let (neg, rest) = match s.as_bytes()[0] {
        b'-' => (true, &s[1..]),
        b'+' => (false, &s[1..]),
        _ => (false, s),
    };
    let (int, frac) = match rest.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (rest, None),
    };
    let digits = |x: &str| !x.is_empty() && x.bytes().all(|b| b.is_ascii_digit());
    if !digits(int) || frac.is_some_and(|f| !digits(f)) {
        // Which error a malformed string gets depends on where the std integer
        // parser trips first (digits are checked left to right, so "1000 " is an
        // overflow for `u8` before the space is seen). Only "never Ok" is specified.
        return Parsed::Err(vec![
            ParseError::Empty,
            ParseError::Invalid,
            ParseError::Overflow,
            ParseError::Precision,
        ]);
    }
    let all = format!("{int}{}", frac.unwrap_or(""));
    let mag = BigInt::parse_bytes(all.as_bytes(), 10).unwrap();
    if mag.is_zero() {
        // zero, however it is spelled; textual scale is the fraction length
        return Parsed::Ok { n: BigInt::zero(), raw: frac.map_or(0, |f| f.len() as i64) };
    }
    if neg && !T::SIGNED {
        return Parsed::Err(vec![ParseError::Invalid]);
    }
    let n = if neg { -mag } else { mag };
    match frac {
        Some(f) => Parsed::Ok { n, raw: f.len() as i64 },
        None => {
            let tz = int.len() - int.trim_end_matches('0').len();
            Parsed::Ok { n: n / pow10(tz as u32), raw: -(tz as i64) }
        }
    }
}

fn fits<T: Int>(n: &BigInt) -> bool {
    T::from_big(n).is_some()
}

/// `Oob::try_from_str(s, scale)`.
fn spec_parse<T: Int>(s: &str, scale: i32) -> Result<BigInt, Vec<ParseError>> {
    match spec_lex::<T>(s) {
        Parsed::Err(k) => Err(k),
        Parsed::Ok { n, .. } if n.is_zero() => Ok(n),
        Parsed::Ok { n, raw } => {
            let scale = scale as i64;
            if raw > scale {
                // too precise; if the digits do not even fit, it is also an overflow
                if fits::<T>(&n) { Err(vec![ParseError::Precision]) } else {
                    Err(vec![ParseError::Precision, ParseError::Overflow])
                }
            } else if scale - raw > 400 {
                Err(vec![ParseError::Overflow]) // non-zero, and 10^400 fits no type here
            } else {
                let m = n * pow10((scale - raw) as u32);
                if fits::<T>(&m) { Ok(m) } else { Err(vec![ParseError::Overflow]) }
            }
        }
    }
}

/// `OobFmt::from_str(s)`: `(mantissa, scale)`.
fn spec_parse_only<T: Int>(s: &str) -> Result<(BigInt, i64), Vec<ParseError>> {
    match spec_lex::<T>(s) {
        Parsed::Err(k) => Err(k),
        Parsed::Ok { n, raw } => {
            if fits::<T>(&n) {
                Ok((n, raw))
            } else {
                Err(vec![ParseError::Precision, ParseError::Overflow])
            }
        }
    }
}

/// `OobFmt::rescale`: semantic -- exact or an error.
fn spec_rescale<T: Int>(n: &BigInt, raw: i64, to: i32) -> Result<BigInt, ParseError> {
    let to = to as i64;
    if n.is_zero() || to == raw {
        return Ok(n.clone());
    }
    if to > raw {
        if to - raw > 400 {
            return Err(ParseError::Overflow);
        }
        let m = n * pow10((to - raw) as u32);
        if fits::<T>(&m) { Ok(m) } else { Err(ParseError::Overflow) }
    } else {
        if raw - to > 400 {
            return Err(ParseError::Precision); // |n| < 10^39, so never a multiple
        }
        let p = pow10((raw - to) as u32);
        if (n % &p).is_zero() { Ok(n / p) } else { Err(ParseError::Precision) }
    }
}

// ---------------------------------------------------------------------------
// Display specification
// ---------------------------------------------------------------------------

/// `(is_negative, digits)` of `m * 10^-scale`, per the `Display` contract:
/// no precision -> shortest form (no trailing fraction zeros); precision `p` ->
/// exactly `p` fraction digits, rounded half away from zero.
fn spec_display(m: &BigInt, scale: i32, prec: Option<usize>) -> (bool, String) {
    let neg = m.is_negative();
    let a = m.abs();
    let p = match prec {
        Some(p) => p as i64,
        None => (scale as i64).max(0),
    };
    let x = if p >= scale as i64 {
        &a * pow10((p - scale as i64) as u32)
    } else {
        round_div(&a, &pow10((scale as i64 - p) as u32), Rounding::Round)
    };
    let mut s = x.to_string();
    if p > 0 {
        while s.len() < p as usize + 1 {
            s.insert(0, '0');
        }
        s.insert(s.len() - p as usize, '.');
    }
    if prec.is_none() && s.contains('.') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    (neg, s)
}

/// Apply sign/width/fill/align the way `std` documents for numbers.
fn spec_pad(neg: bool, body: &str, plus: bool, zero: bool, width: usize, fill: char, align: char) -> String {
    let sign = if neg { "-" } else if plus { "+" } else { "" };
    let len = sign.len() + body.len();
    if width <= len {
        return format!("{sign}{body}");
    }
    let pad = width - len;
    if zero {
        return format!("{sign}{}{body}", "0".repeat(pad));
    }
    let (l, r) = match align {
        '<' => (0, pad),
        '^' => (pad / 2, pad - pad / 2),
        _ => (pad, 0),
    };
    format!("{}{sign}{body}{}", fill.to_string().repeat(l), fill.to_string().repeat(r))
}

// ---------------------------------------------------------------------------
// String generation
// ---------------------------------------------------------------------------

/// A valid spelling of `m` at `scale`: all `scale` fraction digits (trailing
/// zeros kept) for `scale > 0`, the integer `m * 10^-scale` otherwise.
fn canonical_any(m: &BigInt, scale: i32) -> String {
    let body = if scale > 0 {
        spec_display(m, scale, Some(scale as usize)).1
    } else {
        (m.abs() * pow10(scale.unsigned_abs())).to_string()
    };
    if m.is_negative() { format!("-{body}") } else { body }
}

/// The text variants worth throwing at a parser for a value that already has
/// one valid spelling `c`.
fn mangle(c: &str) -> Vec<String> {
    let (sign, body) = match c.strip_prefix('-') {
        Some(b) => ("-", b),
        None => ("", c),
    };
    let mut v = vec![
        c.to_string(),
        format!("+{body}"),          // explicit plus (negated if it was negative)
        format!("{sign}000{body}"),  // leading zeros
        format!("{sign}{body}0"),    // one more trailing zero
        format!("{sign}{body}.0"),   // ".0" appended (or a second '.' if there is one)
        format!("{sign}{body}00"),
        format!("{c}1"),             // one more digit: usually overflows
        format!("{c}9"),
        format!("{sign}1{body}"),
        format!("{sign}9{body}"),
        format!("-{c}"),             // double sign / sign flip
        format!("+{c}"),
        format!(" {c}"),
        format!("{c} "),
        format!("{c}e2"),
        format!("{}.", c),
        format!(".{body}"),
    ];
    if let Some((i, f)) = body.split_once('.') {
        v.push(format!("{sign}{i}.+{f}")); // sign inside the fraction
        v.push(format!("{sign}{i}.-{f}"));
        v.push(format!("{sign}{i}.{}", &f[..f.len().saturating_sub(1)])); // truncated fraction
        v.push(format!("{sign}{i}{f}")); // point dropped
        v.push(format!("{sign}0.{i}{f}")); // point moved
    } else if body.len() > 1 {
        v.push(format!("{sign}{}.{}", &body[..1], &body[1..])); // point inserted
        v.push(format!("{sign}{}", &body[..body.len() - 1]));
    }
    v
}

const FIXED_STRINGS: &[&str] = &[
    "", "+", "-", ".", "-.", "+.", "0", "-0", "+0", "00", "-00", "+00", "000", "0.", ".0", "0.0", "-0.0", "+0.0",
    "0.00", "-0.000", "00.00", "-00.00", "1", "-1", "+1", "01", "10", "100", "-100", "1.", ".1", "1.0", "1.5",
    "-1.5", "+1.5", "1.50", "0.5", "-0.5", "+0.5", "0.05", "-0.05", ".5", "-.5", "1..5", "1.2.3", "--1", "++1",
    "+-1", "-+1", "1-", "1+", "1.-5", "1.+5", "0.-5", "-0.-5", "-0.+5", "+0.-5", "0.+5", "1.5.", "1e5", "1E5",
    "0x10", "1_000", "1,5", "١٢٣", "１２３", "1.٥", "1 .5", "1. 5", "\t1", "1\n", "0.0000000000000000000000000001",
    "-0.0000000000000000000000000001", "1.0000000000000000000000000000000000000000000000001",
    "340282366920938463463374607431768211455", "340282366920938463463374607431768211456",
    "-170141183460469231731687303715884105728", "-170141183460469231731687303715884105729",
    "18446744073709551615", "18446744073709551616", "9223372036854775807", "9223372036854775808",
    "-9223372036854775808", "-9223372036854775809", "1000000000000000000000000000000000000000000000000",
    "123456789012345678901234567890123456789012345678901234567890",
];

/// Scales to render strings at: modest (a string at scale 10^6 is a million digits).
fn gen_scales<T: Int>() -> Vec<i32> {
    let mut v = parse_scales::<T>();
    v.retain(|s| s.unsigned_abs() <= 100);
    v
}

fn parse_scales<T: Int>() -> Vec<i32> {
    let d = T::DIGITS as i32;
    // The last four are the i32 limits: scale arithmetic there must not overflow.
    let mut v = vec![
        -40, -d - 1, -d, -3, -2, -1, 0, 1, 2, 3, 5, d - 1, d, d + 1, d + 2, 40, 100, 500, 1_000_000,
        -1_000_000, i32::MIN + 1, i32::MIN, i32::MAX - 1, i32::MAX,
    ];
    v.sort();
    v.dedup();
    v
}

// ---------------------------------------------------------------------------
// The differential tests
// ---------------------------------------------------------------------------

fn kinds_msg(k: &[ParseError]) -> String {
    format!("{k:?}")
}

fn check_parse<T>(f: &mut Failures, s: &str, scales: &[i32])
where
    T: Int + Num<FromStrRadixErr = ParseIntError>,
{
    let name = T::NAME;
    // try_from_str, at every scale
    for &sc in scales {
        let got = guarded(|| D::<T>::try_from_str(s, sc).map(|v| v.mantissa().big()));
        let Ok(got) = got else {
            f.check(|| format!("{name} try_from_str PANIC"), || format!("{s:?} @{sc}"), Err::<(), ()>(()), ());
            continue;
        };
        match spec_parse::<T>(s, sc) {
            Ok(want) => f.check(|| format!("{name} try_from_str Ok"), || format!("{s:?} @{sc}"),
                Ok(got.clone().map_err(|e| format!("{e:?}"))), Ok(want)),
            Err(kinds) => {
                let ok = matches!(&got, Err(e) if kinds.contains(e));
                f.check(|| format!("{name} try_from_str Err"), || format!("{s:?} @{sc} allowed {}", kinds_msg(&kinds)),
                    Ok(ok), true);
            }
        }
    }
    // OobFmt::from_str, then rescale to a few scales
    let got = guarded(|| s.parse::<OobFmt<T>>().map(|o| (o.0.mantissa().big(), o.1 as i64)));
    let Ok(got) = got else {
        f.check(|| format!("{name} OobFmt::from_str PANIC"), || format!("{s:?}"), Err::<(), ()>(()), ());
        return;
    };
    match spec_parse_only::<T>(s) {
        Ok(want) => {
            f.check(|| format!("{name} OobFmt::from_str Ok"), || format!("{s:?}"),
                Ok(got.clone().map_err(|e| format!("{e:?}"))), Ok(want.clone()));
            // (a parse failure was already reported above)
            let Ok(o) = s.parse::<OobFmt<T>>() else { return };
            for &sc in scales {
                let got = guarded(|| o.rescale(sc).map(|v| v.mantissa().big()));
                let want = spec_rescale::<T>(&want.0, want.1, sc);
                f.check(|| format!("{name} rescale"), || format!("{s:?} -> @{sc}"),
                    got.map(|r| r.map_err(|e| format!("{e:?}"))), want.map_err(|e| format!("{e:?}")));
            }
        }
        Err(kinds) => {
            let ok = matches!(&got, Err(e) if kinds.contains(e));
            f.check(|| format!("{name} OobFmt::from_str Err"), || format!("{s:?} allowed {}", kinds_msg(&kinds)),
                Ok(ok), true);
        }
    }
}

fn run_parse<T>()
where
    T: Int + Num<FromStrRadixErr = ParseIntError>,
{
    let mut f = Failures::default();
    let scales = parse_scales::<T>();
    let gens = gen_scales::<T>();
    let mut rng = Rng(0xDEC1_0000 ^ T::BITS as u64 ^ ((T::SIGNED as u64) << 33));
    let k = scale_mult();

    for s in FIXED_STRINGS {
        check_parse::<T>(&mut f, s, &scales);
    }

    // every boundary value, rendered at several scales and mangled
    let all = interesting::<T>();
    let stride = (all.len() / (60 * k)).max(1);
    for &m in all.iter().step_by(stride) {
        for &sc in &gens {
            // parse each at the scale it was written for, its neighbours and 0
            let near = [sc.saturating_sub(1), sc, sc.saturating_add(1), 0];
            for s in mangle(&canonical_any(&m.big(), sc)) {
                check_parse::<T>(&mut f, &s, &near);
            }
        }
    }
    // the type's own limits and their neighbours, at EVERY scale: the exact edge of Overflow
    for &sc in &gens {
        for d in -2..=2i32 {
            for edge in [T::max_big() + d, T::min_big() + d, BigInt::from(d)] {
                for s in mangle(&canonical_any(&edge, sc)) {
                    check_parse::<T>(&mut f, &s, &scales);
                }
            }
        }
    }
    // random magnitudes
    for _ in 0..(400 * k) {
        let m: T = rng.int();
        let sc = gens[rng.below(gens.len() as u64) as usize];
        let near = [sc.saturating_sub(1), sc, sc.saturating_add(1), 0];
        for s in mangle(&canonical_any(&m.big(), sc)) {
            check_parse::<T>(&mut f, &s, &near);
        }
    }
    f.finish(&format!("parse {}", T::NAME));
}

/// `Display` with every precision and many scales, and `parse(display(x)) == x`.
fn run_display<T>()
where
    T: Int + Num<FromStrRadixErr = ParseIntError>,
{
    let mut f = Failures::default();
    let d = T::DIGITS as i32;
    let mut scales = vec![-d - 2, -d, -d / 2, -2, -1, 0, 1, 2, 3, d / 2, d - 1, d, d + 1, d + 2, 2 * d, 60, 200];
    scales.retain(|&s| s >= -45);
    scales.sort();
    scales.dedup();
    let precs: [Option<usize>; 12] = [
        None, Some(0), Some(1), Some(2), Some(3), Some(5), Some(10), Some(20), Some(39), Some(40), Some(100), Some(200),
    ];
    let mut rng = Rng(0xD15B_1A70 ^ T::BITS as u64 ^ ((T::SIGNED as u64) << 33));
    let mut vals = interesting::<T>();
    for _ in 0..(300 * scale_mult()) {
        vals.push(rng.int());
    }
    let name = T::NAME;
    for &m in &vals {
        let dec = D::<T>::from_mantissa(m);
        for &sc in &scales {
            let fmt = OobFmt(dec, sc);
            for &p in &precs {
                // documented panics: |scale| > 200, or precision + (-scale) > 200 for scale <= 0
                if p.is_some_and(|p| sc <= 0 && p as i64 + (-sc) as i64 > 200) {
                    continue;
                }
                let (neg, body) = spec_display(&m.big(), sc, p);
                let want = if neg { format!("-{body}") } else { body.clone() };
                let got = guarded(|| match p {
                    None => format!("{fmt}"),
                    Some(p) => format!("{fmt:.p$}"),
                });
                f.check(|| format!("{name} display prec={p:?}"), || format!("{m:?} @{sc}"), got, want.clone());

                // Padding / sign flags, on a few precisions only (they are orthogonal to the digits).
                if matches!(p, None | Some(0) | Some(2)) && sc.abs() <= d + 2 {
                    let w = body.len() + 4;
                    let cases = [
                        ("{:+}", spec_pad(neg, &body, true, false, 0, ' ', '>')),
                        ("{:>w}", spec_pad(neg, &body, false, false, w, ' ', '>')),
                        ("{:<w}", spec_pad(neg, &body, false, false, w, '*', '<')),
                        ("{:^w}", spec_pad(neg, &body, false, false, w, '*', '^')),
                        ("{:0w}", spec_pad(neg, &body, false, true, w, '0', '>')),
                        ("{:+0w}", spec_pad(neg, &body, true, true, w, '0', '>')),
                        ("{:w}", spec_pad(neg, &body, false, false, w, ' ', '>')),
                        ("{:3}", spec_pad(neg, &body, false, false, 3, ' ', '>')),
                    ];
                    for (spec, want) in cases {
                        let got = guarded(|| render(&fmt, spec, w, p));
                        f.check(|| format!("{name} display flags {spec} prec={p:?}"), || format!("{m:?} @{sc}"), got, want);
                    }
                }
            }
            // parse(display(x)) == x
            if sc.abs() <= 45 {
                let text = format!("{fmt}");
                let back = guarded(|| D::<T>::try_from_str(&text, sc).map(|v| v.mantissa()));
                f.check(|| format!("{name} roundtrip display->parse"), || format!("{m:?} @{sc} text {text:?}"),
                    back.map(|r| r.map_err(|e| format!("{e:?}"))), Ok(m));
            }
        }
    }
    f.finish(&format!("display {}", T::NAME));
}

/// `format!` with a runtime width and precision (std takes only literal specs).
fn render<T: Int>(v: &OobFmt<T>, spec: &str, w: usize, p: Option<usize>) -> String {
    match (spec, p) {
        ("{:+}", None) => format!("{v:+}"),
        ("{:+}", Some(p)) => format!("{v:+.p$}"),
        ("{:>w}", None) => format!("{v:>w$}"),
        ("{:>w}", Some(p)) => format!("{v:>w$.p$}"),
        ("{:<w}", None) => format!("{v:*<w$}"),
        ("{:<w}", Some(p)) => format!("{v:*<w$.p$}"),
        ("{:^w}", None) => format!("{v:*^w$}"),
        ("{:^w}", Some(p)) => format!("{v:*^w$.p$}"),
        ("{:0w}", None) => format!("{v:0w$}"),
        ("{:0w}", Some(p)) => format!("{v:0w$.p$}"),
        ("{:+0w}", None) => format!("{v:+0w$}"),
        ("{:+0w}", Some(p)) => format!("{v:+0w$.p$}"),
        ("{:w}", None) => format!("{v:w$}"),
        ("{:w}", Some(p)) => format!("{v:w$.p$}"),
        ("{:3}", None) => format!("{v:3}"),
        ("{:3}", Some(p)) => format!("{v:3.p$}"),
        _ => unreachable!("{spec}"),
    }
}

macro_rules! per_type {
    ($($t:ty: $parse:ident, $display:ident);* $(;)?) => { $(
        #[test] fn $parse() { run_parse::<$t>(); }
        #[test] fn $display() { run_display::<$t>(); }
    )* };
}
per_type!(i8: parse_i8, display_i8; i16: parse_i16, display_i16; i32: parse_i32, display_i32;
          i64: parse_i64, display_i64; i128: parse_i128, display_i128;
          u8: parse_u8, display_u8; u16: parse_u16, display_u16; u32: parse_u32, display_u32;
          u64: parse_u64, display_u64; u128: parse_u128, display_u128);

// ---------------------------------------------------------------------------
// Self-checks of the specification, so a wrong oracle cannot hide a wrong library
// ---------------------------------------------------------------------------

#[test]
fn spec_display_hand_computed_table() {
    let b = |n: i64| BigInt::from(n);
    // (mantissa, scale, precision, expected "-"? , expected body)
    let t: &[(i64, i32, Option<usize>, bool, &str)] = &[
        (1234500, 4, None, false, "123.45"),
        (1234500, 4, Some(4), false, "123.4500"),
        (1234500, 4, Some(1), false, "123.5"), // .45 -> tie rounds away
        (-1234500, 4, Some(1), true, "123.5"),
        (1234400, 4, Some(1), false, "123.4"),
        (5, 1, Some(0), false, "1"),           // 0.5 -> 1 (tie away)
        (4, 1, Some(0), false, "0"),
        (0, 3, None, false, "0"),
        (0, 3, Some(2), false, "0.00"),
        (1, 3, None, false, "0.001"),
        (1, -3, None, false, "1000"),
        (1, -3, Some(2), false, "1000.00"),
        (0, -3, None, false, "0"),
        (-1, 3, Some(2), true, "0.00"),         // negative zero keeps its sign, like f64
        (999, 3, Some(2), false, "1.00"),       // carry across the point
        (9996, 4, Some(3), false, "1.000"),
    ];
    for (m, sc, p, neg, body) in t {
        assert_eq!(spec_display(&b(*m), *sc, *p), (*neg, body.to_string()), "{m} @{sc} {p:?}");
    }
}

#[test]
fn spec_parse_hand_computed_table() {
    use ParseError::*;
    let s = |x: &str, sc: i32| spec_parse::<i8>(x, sc);
    assert_eq!(s("1.5", 1), Ok(BigInt::from(15)));
    assert_eq!(s("1.2", 2), Ok(BigInt::from(120)));
    assert_eq!(s("1.5", 2), Err(vec![Overflow]));    // 150 > i8::MAX
    assert_eq!(spec_parse::<i16>("1.50", 1), Err(vec![Precision])); // textual policy
    assert_eq!(s("1.50", 1), Err(vec![Precision, Overflow])); // 150 does not fit i8 either
    assert_eq!(s("0.00", 1), Ok(BigInt::from(0)));   // zero is zero
    assert_eq!(s("12.8", 1), Err(vec![Overflow]));   // 128 > i8::MAX
    assert_eq!(s("-12.8", 1), Ok(BigInt::from(-128)));
    assert_eq!(s("1000", 0), Err(vec![Overflow]));
    assert_eq!(s("100", 0), Ok(BigInt::from(100)));
    assert_eq!(s("100", -2), Ok(BigInt::from(1)));
    assert_eq!(s("10", -2), Err(vec![Precision]));
    assert!(s("1.+5", 3).is_err_and(|k| k.contains(&Invalid)));
    assert_eq!(s("", 0), Err(vec![Empty]));
    assert_eq!(spec_parse::<u8>("-0", 2), Ok(BigInt::from(0)));
    assert_eq!(spec_parse::<u8>("-1", 2), Err(vec![Invalid]));
}
