//! Numeric input and the parsers each form uses.
//!
//! Every conversion function takes `impl Into<Number>`, so integers, floats,
//! numeric strings and big integers are all accepted. Strings keep arbitrary
//! precision (`"12345678901234567890"` is exact) and accept scientific
//! notation; floats go through their shortest round-trip representation, so
//! `0.1` spells "zero point one", not its binary expansion.

use crate::error::{Error, Result};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{ToPrimitive, Zero};
use std::borrow::Cow;

/// A numeric value accepted by every form.
///
/// Build one through the `From` conversions rather than directly:
/// `to_cardinal(42)`, `to_cardinal("42.5")`, `to_cardinal(1e21)`,
/// `to_cardinal(BigInt::from(7))` all work.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Number<'a> {
    /// Any primitive integer except `u128`.
    Int(i128),
    /// A finite `f64`, spelled from its shortest round-trip decimal form.
    Float(f64),
    /// An arbitrary-precision integer.
    Big(BigInt),
    /// A decimal string: optional sign, digits, optional fraction, optional
    /// exponent (`"-12.5"`, `"1e21"`, `".5"`).
    Text(Cow<'a, str>),
}

macro_rules! from_int {
    ($($t:ty),*) => {$(
        impl From<$t> for Number<'static> {
            fn from(value: $t) -> Self {
                Number::Int(value as i128)
            }
        }
    )*};
}
from_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, usize);

impl From<u128> for Number<'static> {
    fn from(value: u128) -> Self {
        Number::Big(BigInt::from(value))
    }
}

impl From<f64> for Number<'static> {
    fn from(value: f64) -> Self {
        Number::Float(value)
    }
}

impl From<f32> for Number<'static> {
    /// Goes through the `f32`'s own shortest decimal form, so `0.1f32` is
    /// "zero point one" rather than the widened `0.100000001490116...`.
    fn from(value: f32) -> Self {
        Number::Text(Cow::Owned(value.to_string()))
    }
}

impl From<BigInt> for Number<'static> {
    fn from(value: BigInt) -> Self {
        Number::Big(value)
    }
}

impl From<&BigInt> for Number<'static> {
    fn from(value: &BigInt) -> Self {
        Number::Big(value.clone())
    }
}

impl From<BigUint> for Number<'static> {
    fn from(value: BigUint) -> Self {
        Number::Big(BigInt::from(value))
    }
}

impl From<&BigUint> for Number<'static> {
    fn from(value: &BigUint) -> Self {
        Number::Big(BigInt::from(value.clone()))
    }
}

impl<'a> From<&'a str> for Number<'a> {
    fn from(value: &'a str) -> Self {
        Number::Text(Cow::Borrowed(value))
    }
}

impl From<String> for Number<'static> {
    fn from(value: String) -> Self {
        Number::Text(Cow::Owned(value))
    }
}

impl<'a> From<&'a String> for Number<'a> {
    fn from(value: &'a String) -> Self {
        Number::Text(Cow::Borrowed(value))
    }
}

impl<'a> From<Cow<'a, str>> for Number<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Number::Text(value)
    }
}

impl<'a, 'b: 'a> From<&'a Number<'b>> for Number<'a> {
    fn from(value: &'a Number<'b>) -> Self {
        match value {
            Number::Int(i) => Number::Int(*i),
            Number::Float(f) => Number::Float(*f),
            Number::Big(b) => Number::Big(b.clone()),
            Number::Text(t) => Number::Text(Cow::Borrowed(t)),
        }
    }
}

// ============================================================================
// Parsed shapes
// ============================================================================

/// A cardinal input: sign, integer magnitude and the fraction's digits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Cardinal {
    /// Whether the value is negative (`"-0"` counts, as in the JS library).
    pub negative: bool,
    /// The integer part's magnitude.
    pub integer: BigUint,
    /// The digits after the decimal point, exactly as written (`"007"`), or
    /// `None` when there are none.
    pub fraction: Option<String>,
}

/// A currency amount: sign, major units and minor units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Money {
    /// Whether the amount is negative.
    pub negative: bool,
    /// Whole major units (dollars, euros, ...).
    pub major: BigUint,
    /// Minor units (cents, ...), at the precision the parser was asked for.
    pub minor: u32,
}

/// Parses a value for the cardinal form: any finite number, negatives and
/// decimals included.
pub(crate) fn parse_cardinal(value: &Number<'_>) -> Result<Cardinal> {
    let decimal = decompose(value)?;
    Ok(Cardinal {
        negative: decimal.negative,
        integer: decimal.integer,
        fraction: decimal.fraction.filter(|f| !f.is_empty()),
    })
}

/// Parses a value for the ordinal form: positive integers only.
pub(crate) fn parse_ordinal(value: &Number<'_>) -> Result<BigUint> {
    const POSITIVE: &str = "Ordinals must be positive integers";
    const WHOLE: &str = "Ordinals must be whole numbers";
    match value {
        Number::Float(f) if !f.is_finite() => {
            return Err(Error::InvalidOrdinal(
                "Ordinals must be finite numbers".into(),
            ));
        }
        Number::Float(f) if f.fract() != 0.0 => return Err(Error::InvalidOrdinal(WHOLE.into())),
        Number::Text(t) => {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                return Err(Error::InvalidOrdinal(
                    "Ordinals cannot be empty strings".into(),
                ));
            }
            if trimmed.starts_with('-') {
                return Err(Error::InvalidOrdinal("Ordinals cannot be negative".into()));
            }
            if trimmed.contains('.') && !trimmed.contains(['e', 'E']) {
                return Err(Error::InvalidOrdinal(WHOLE.into()));
            }
        }
        _ => {}
    }
    let decimal = decompose(value)?;
    if decimal.negative && !decimal.integer.is_zero() {
        return Err(Error::InvalidOrdinal(POSITIVE.into()));
    }
    if decimal
        .fraction
        .is_some_and(|f| f.bytes().any(|b| b != b'0'))
    {
        return Err(Error::InvalidOrdinal(WHOLE.into()));
    }
    if decimal.integer.is_zero() {
        return Err(Error::InvalidOrdinal(POSITIVE.into()));
    }
    Ok(decimal.integer)
}

/// Parses a value for the currency form, keeping `minor_digits` decimal
/// digits (2, or 3 for a thousandth-unit currency) and truncating anything
/// finer: at 2 digits `"1.004"` is one major unit and zero minor units.
///
/// Truncating rather than rejecting is deliberate: float input would
/// otherwise be unusable, since `0.1 + 0.2` is `0.30000000000000004`.
pub(crate) fn parse_currency(value: &Number<'_>, minor_digits: u32) -> Result<Money> {
    let decimal = decompose(value)?;
    let minor = match decimal.fraction {
        None => 0,
        Some(fraction) => {
            let mut digits: String = fraction.chars().take(minor_digits as usize).collect();
            while digits.len() < minor_digits as usize {
                digits.push('0');
            }
            digits.parse().expect("minor digits are ASCII digits")
        }
    };
    Ok(Money {
        negative: decimal.negative,
        major: decimal.integer,
        minor,
    })
}

// ============================================================================
// Decomposition
// ============================================================================

/// The common shape every parser starts from.
struct Decimal {
    negative: bool,
    integer: BigUint,
    /// Fraction digits exactly as written; `Some("")` is possible for `"1."`.
    fraction: Option<String>,
}

/// Largest exponent accepted in scientific notation, to bound the digit
/// string a pathological `"1e999999999"` would otherwise expand to.
const MAX_EXPONENT: i64 = 10_000;

fn decompose(value: &Number<'_>) -> Result<Decimal> {
    match value {
        Number::Int(i) => Ok(Decimal {
            negative: *i < 0,
            integer: BigUint::from(i.unsigned_abs()),
            fraction: None,
        }),
        Number::Big(b) => Ok(Decimal {
            negative: b.sign() == Sign::Minus,
            integer: b.magnitude().clone(),
            fraction: None,
        }),
        Number::Float(f) => {
            if !f.is_finite() {
                return Err(Error::InvalidNumber(
                    "Number must be finite (NaN and Infinity are not supported)".into(),
                ));
            }
            // Fast path for integers that are exact in an f64.
            if f.fract() == 0.0 && f.abs() <= 9_007_199_254_740_992.0 {
                let magnitude = f.abs().to_u64().expect("checked to fit");
                return Ok(Decimal {
                    negative: *f < 0.0,
                    integer: BigUint::from(magnitude),
                    fraction: None,
                });
            }
            // `Display` for f64 is the shortest round-trip form and never uses
            // an exponent, which is exactly the expanded form we want.
            decompose_text(&f.to_string())
        }
        Number::Text(t) => decompose_text(t),
    }
}

fn invalid(text: &str) -> Error {
    Error::InvalidNumber(format!("Invalid number format: {text:?}"))
}

/// Parses a decimal string: optional sign, digits with an optional fraction,
/// optional exponent. Hex, `Infinity` and `NaN` are rejected.
fn decompose_text(text: &str) -> Result<Decimal> {
    let trimmed = text.trim();
    let bytes = trimmed.as_bytes();
    let mut pos = 0;

    let negative = match bytes.first() {
        Some(b'-') => {
            pos += 1;
            true
        }
        Some(b'+') => {
            pos += 1;
            false
        }
        _ => false,
    };

    let int_start = pos;
    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
        pos += 1;
    }
    let int_digits = &trimmed[int_start..pos];

    let mut frac_digits: Option<&str> = None;
    if pos < bytes.len() && bytes[pos] == b'.' {
        pos += 1;
        let frac_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        frac_digits = Some(&trimmed[frac_start..pos]);
    }

    if int_digits.is_empty() && frac_digits.is_none_or(str::is_empty) {
        return Err(invalid(text));
    }

    let mut exponent: i64 = 0;
    if pos < bytes.len() && (bytes[pos] == b'e' || bytes[pos] == b'E') {
        pos += 1;
        let exp_negative = match bytes.get(pos) {
            Some(b'-') => {
                pos += 1;
                true
            }
            Some(b'+') => {
                pos += 1;
                false
            }
            _ => false,
        };
        let exp_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        let exp_digits = &trimmed[exp_start..pos];
        if exp_digits.is_empty() {
            return Err(invalid(text));
        }
        let magnitude: i64 = exp_digits
            .trim_start_matches('0')
            .parse()
            .ok()
            .filter(|&m| m <= MAX_EXPONENT)
            .ok_or_else(|| Error::InvalidNumber(format!("Exponent too large in {text:?}")))?;
        exponent = if exp_negative { -magnitude } else { magnitude };
    }

    if pos != bytes.len() {
        return Err(invalid(text));
    }

    let (integer, fraction) = if exponent == 0 {
        (int_digits.to_string(), frac_digits.map(str::to_string))
    } else {
        shift_point(int_digits, frac_digits.unwrap_or(""), exponent)
    };

    let integer = if integer.is_empty() {
        BigUint::zero()
    } else {
        BigUint::parse_bytes(integer.as_bytes(), 10).expect("validated ASCII digits")
    };
    Ok(Decimal {
        negative,
        integer,
        fraction,
    })
}

/// Moves the decimal point of `int.frac` by `exponent` places.
fn shift_point(int_digits: &str, frac_digits: &str, exponent: i64) -> (String, Option<String>) {
    let digits = format!("{int_digits}{frac_digits}");
    let point = int_digits.len() as i64 + exponent;
    if point >= digits.len() as i64 {
        let zeros = "0".repeat((point - digits.len() as i64) as usize);
        (format!("{digits}{zeros}"), None)
    } else if point <= 0 {
        let zeros = "0".repeat((-point) as usize);
        (String::new(), Some(format!("{zeros}{digits}")))
    } else {
        let (i, f) = digits.split_at(point as usize);
        (i.to_string(), Some(f.to_string()))
    }
}

// ============================================================================
// Helpers for language implementations
// ============================================================================

/// Splits `n` into base-`group` digit groups, least significant first
/// (`split_groups(1_234_567, 1000)` is `[567, 234, 1]`). Zero yields `[]`.
pub(crate) fn split_groups(n: &BigUint, group: u32) -> Vec<u32> {
    if let Some(small) = n.to_u64() {
        let mut out = Vec::new();
        let mut rest = small;
        let group = u64::from(group);
        while rest > 0 {
            out.push((rest % group) as u32);
            rest /= group;
        }
        return out;
    }
    let mut out = Vec::new();
    let mut rest = n.clone();
    while !rest.is_zero() {
        out.push((&rest % group).to_u32().expect("remainder below group"));
        rest /= group;
    }
    out
}

/// Splits `n` South Asian style: a first group of `head` (1000) then groups
/// of `rest` (100), least significant first. Zero yields `[]`.
pub(crate) fn split_groups_indian(n: &BigUint, head: u32, rest: u32) -> Vec<u32> {
    if n.is_zero() {
        return Vec::new();
    }
    let mut out = vec![(n % head).to_u32().expect("remainder below head")];
    out.extend(split_groups(&(n / head), rest));
    out
}

/// Parses a string of ASCII digits (a fraction part) into a big integer.
pub(crate) fn digits_to_biguint(digits: &str) -> BigUint {
    BigUint::parse_bytes(digits.as_bytes(), 10).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn big(s: &str) -> BigUint {
        BigUint::parse_bytes(s.as_bytes(), 10).unwrap()
    }

    #[test]
    fn cardinal_from_integers_and_bigints() {
        let c = parse_cardinal(&Number::from(-42)).unwrap();
        assert_eq!((c.negative, c.integer, c.fraction), (true, big("42"), None));
        let c = parse_cardinal(&Number::from(u128::MAX)).unwrap();
        assert_eq!(c.integer, big("340282366920938463463374607431768211455"));
        let c = parse_cardinal(&Number::from(
            BigInt::parse_bytes(b"-123456789012345678901234567890", 10).unwrap(),
        ))
        .unwrap();
        assert!(c.negative);
        assert_eq!(c.integer, big("123456789012345678901234567890"));
    }

    #[test]
    fn cardinal_from_floats_uses_shortest_repr() {
        let c = parse_cardinal(&Number::from(17.42)).unwrap();
        assert_eq!((c.integer, c.fraction.as_deref()), (big("17"), Some("42")));
        let c = parse_cardinal(&Number::from(0.007)).unwrap();
        assert_eq!((c.integer, c.fraction.as_deref()), (big("0"), Some("007")));
        let c = parse_cardinal(&Number::from(1e21)).unwrap();
        assert_eq!(c.integer, big("1000000000000000000000"));
        let c = parse_cardinal(&Number::from(-0.0)).unwrap();
        assert!(!c.negative);
        let c = parse_cardinal(&Number::from(0.1f32)).unwrap();
        assert_eq!(c.fraction.as_deref(), Some("1"));
        assert!(parse_cardinal(&Number::from(f64::NAN)).is_err());
        assert!(parse_cardinal(&Number::from(f64::INFINITY)).is_err());
    }

    #[test]
    fn cardinal_from_strings() {
        let c = parse_cardinal(&Number::from("  -1.500 ")).unwrap();
        assert_eq!(
            (c.negative, c.integer, c.fraction.as_deref()),
            (true, big("1"), Some("500"))
        );
        let c = parse_cardinal(&Number::from(".5")).unwrap();
        assert_eq!((c.integer, c.fraction.as_deref()), (big("0"), Some("5")));
        let c = parse_cardinal(&Number::from("7.")).unwrap();
        assert_eq!((c.integer, c.fraction), (big("7"), None));
        let c = parse_cardinal(&Number::from("+12")).unwrap();
        assert_eq!(c.integer, big("12"));
        let c = parse_cardinal(&Number::from("-0")).unwrap();
        assert!(c.negative);
        for bad in [
            "", "   ", "abc", "1.2.3", "0x1A", "Infinity", "NaN", "1e", "1_000", "--1", "1 2",
        ] {
            assert!(
                parse_cardinal(&Number::from(bad)).is_err(),
                "{bad:?} should be rejected"
            );
        }
    }

    #[test]
    fn scientific_notation_expands_exactly() {
        let c = parse_cardinal(&Number::from("1e21")).unwrap();
        assert_eq!(c.integer, big("1000000000000000000000"));
        let c = parse_cardinal(&Number::from("1.5e3")).unwrap();
        assert_eq!((c.integer, c.fraction), (big("1500"), None));
        let c = parse_cardinal(&Number::from("1.5e-3")).unwrap();
        assert_eq!((c.integer, c.fraction.as_deref()), (big("0"), Some("0015")));
        let c = parse_cardinal(&Number::from("-2.5E+1")).unwrap();
        assert_eq!((c.negative, c.integer, c.fraction), (true, big("25"), None));
        let c = parse_cardinal(&Number::from("12345e-2")).unwrap();
        assert_eq!((c.integer, c.fraction.as_deref()), (big("123"), Some("45")));
        assert!(parse_cardinal(&Number::from("1e99999999")).is_err());
    }

    #[test]
    fn ordinal_rejects_non_positive_and_fractional() {
        assert_eq!(parse_ordinal(&Number::from(42)).unwrap(), big("42"));
        assert_eq!(parse_ordinal(&Number::from(3.0)).unwrap(), big("3"));
        assert_eq!(parse_ordinal(&Number::from("1e3")).unwrap(), big("1000"));
        for bad in [
            Number::from(0),
            Number::from(-1),
            Number::from(1.5),
            Number::from("0"),
            Number::from("-5"),
            Number::from("1.0"),
            Number::from(""),
            Number::from("x"),
        ] {
            assert!(parse_ordinal(&bad).is_err(), "{bad:?} should be rejected");
        }
        assert!(parse_ordinal(&Number::from(f64::INFINITY)).is_err());
    }

    #[test]
    fn currency_truncates_to_minor_digits() {
        let m = parse_currency(&Number::from(42.5), 2).unwrap();
        assert_eq!((m.negative, m.major, m.minor), (false, big("42"), 50));
        let m = parse_currency(&Number::from("1.004"), 2).unwrap();
        assert_eq!((m.major, m.minor), (big("1"), 0));
        let m = parse_currency(&Number::from("1.500"), 3).unwrap();
        assert_eq!((m.major, m.minor), (big("1"), 500));
        let m = parse_currency(&Number::from("0.001"), 3).unwrap();
        assert_eq!((m.major, m.minor), (big("0"), 1));
        let m = parse_currency(&Number::from(-1), 2).unwrap();
        assert_eq!((m.negative, m.major, m.minor), (true, big("1"), 0));
        let m = parse_currency(&Number::from(0.1 + 0.2), 2).unwrap();
        assert_eq!(m.minor, 30);
        let m = parse_currency(&Number::from(BigInt::from(5)), 2).unwrap();
        assert_eq!(m.major, big("5"));
    }

    #[test]
    fn group_splitting() {
        assert_eq!(split_groups(&big("1234567"), 1000), vec![567, 234, 1]);
        assert_eq!(split_groups(&big("0"), 1000), Vec::<u32>::new());
        assert_eq!(
            split_groups(&big("100000000000000000000000"), 1000),
            vec![0, 0, 0, 0, 0, 0, 0, 100]
        );
        assert_eq!(split_groups(&big("123456789"), 10000), vec![6789, 2345, 1]);
        assert_eq!(
            split_groups_indian(&big("12345678"), 1000, 100),
            vec![678, 45, 23, 1]
        );
        assert_eq!(digits_to_biguint("007"), big("7"));
    }
}
