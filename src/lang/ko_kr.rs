//! Korean (South Korea) (`ko-KR`).
//!
//! - Myriad-based (만) grouping: 4 digits per segment
//! - Implicit "일" (one) omission before scale words
//! - Space separation after 만+ scales
//! - Hangul numerals
//!
//! ```
//! use n2words::ko_kr::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "이십일");
//! assert_eq!(to_cardinal(1000001)?, "백만 일");
//! assert_eq!(to_ordinal(42)?, "제사십이");
//! assert_eq!(to_currency(50000)?, "오만원");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::KO as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{
    Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal, split_groups,
};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

const ONES: [&str; 10] = ["", "일", "이", "삼", "사", "오", "육", "칠", "팔", "구"];

const TEN: &str = "십";
const HUNDRED: &str = "백";
const THOUSAND: &str = "천";

const ZERO: &str = "영";
const NEGATIVE: &str = "마이너스";
const DECIMAL_SEP: &str = "점";

// Myriad scale words (powers of 10,000)
// 만 (10^4), 억 (10^8), 조 (10^12), 경 (10^16), etc.
const SCALES: &[&str] = &["만", "억", "조", "경", "해", "자", "양"];

/// The cardinal ceiling, derived from the scale table: `10^32`. Each scale
/// word covers a power of 10,000, so the first unsupported value is
/// `10^((SCALES.len() + 1) * 4)`.
pub const CARDINAL_MAX: Max = Max::myriad(SCALES.len() as u32);
/// The ordinal ceiling: `10^32` (ordinals prefix the cardinal speller).
pub const ORDINAL_MAX: Max = Max::myriad(SCALES.len() as u32);
/// The currency ceiling: `10^32` (currency builds on the cardinal speller).
pub const CURRENCY_MAX: Max = Max::myriad(SCALES.len() as u32);

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

const ORDINAL_PREFIX: &str = "제";

// ============================================================================
// Segment Building
// ============================================================================

/// Appends `ONES[digit] + scale`, omitting 일 when the digit is 1.
fn push_place(result: &mut String, digit: usize, scale: &str) {
    if digit > 0 {
        if digit != 1 {
            result.push_str(ONES[digit]);
        }
        result.push_str(scale);
    }
}

/// Words for a 0-9999 (4-digit myriad) segment. Korean omits "일" before
/// 십, 백 and 천.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = ((n / 100) % 10) as usize;
    let thousands = (n / 1000) as usize;

    let mut result = String::new();
    push_place(&mut result, thousands, THOUSAND);
    push_place(&mut result, hundreds, HUNDRED);
    push_place(&mut result, tens, TEN);
    if ones > 0 {
        result.push_str(ONES[ones]);
    }
    result
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }
    // Fast path: numbers < 10000
    if let Some(small) = n.to_u64() {
        if small < 10_000 {
            return build_segment(small as u32);
        }
    }
    // For numbers >= 10000, use myriad decomposition
    build_large_number_words(n)
}

/// Words for a number of 10,000 or more, by 4-digit (만) segments, joined
/// with the Korean spacing rules: no spaces within a segment, a space after a
/// scale word before the next number.
fn build_large_number_words(n: &BigUint) -> String {
    // Segments least-significant first (index 0 = units, 1 = 만, etc.)
    let segments = split_groups(n, 10_000);

    let mut result = String::new();
    let mut prev_was_scale = false;
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if i == 0 {
            // Units segment (no scale word)
            if prev_was_scale {
                result.push(' ');
            }
            result.push_str(&build_segment(segment));
            prev_was_scale = false;
        } else {
            // Korean omits the segment when it is 1 before a scale word
            if segment != 1 {
                if prev_was_scale {
                    result.push(' ');
                }
                result.push_str(&build_segment(segment));
            }
            result.push_str(SCALES[i - 1]);
            prev_was_scale = true;
        }
    }

    if result.is_empty() {
        ZERO.to_string()
    } else {
        result
    }
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number, space-separated.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    let zeros = decimal.bytes().take_while(|&b| b == b'0').count();
    for _ in 0..zeros {
        parts.push(ZERO.to_string());
    }
    let remainder = &decimal[zeros..];
    if !remainder.is_empty() {
        parts.push(integer_to_words(&digits_to_biguint(remainder)));
    }
    parts.join(" ")
}

/// Spells a number in Korean (Sino-Korean numerals).
///
/// ```
/// # use n2words::ko_kr::to_cardinal;
/// assert_eq!(to_cardinal(10000)?, "만");
/// assert_eq!(to_cardinal(1000000)?, "백만");
/// assert_eq!(to_cardinal(-17.42)?, "마이너스 십칠 점 사십이");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value, [`Error::OutOfRange`](crate::Error::OutOfRange) at or past
/// [`CARDINAL_MAX`].
pub fn to_cardinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    let number = parse_cardinal(&value.into())?;
    // The fraction's significant digits are spelled via the scale builder,
    // so both parts must clear the ceiling.
    CARDINAL_MAX.check(&number.integer, number.fraction.as_deref())?;

    let mut parts: Vec<String> = Vec::new();
    if number.negative {
        parts.push(NEGATIVE.to_string());
    }
    parts.push(integer_to_words(&number.integer));
    if let Some(fraction) = &number.fraction {
        parts.push(DECIMAL_SEP.to_string());
        parts.push(decimal_part_to_words(fraction));
    }
    Ok(parts.join(" "))
}

// ============================================================================
// Ordinal
// ============================================================================

/// Ordinal words for a positive integer: the 제 prefix plus the Sino-Korean
/// cardinal.
fn integer_to_ordinal(n: &BigUint) -> String {
    format!("{ORDINAL_PREFIX}{}", integer_to_words(n))
}

/// Spells a positive integer as a Korean ordinal (제 + cardinal).
///
/// ```
/// # use n2words::ko_kr::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "제일");
/// assert_eq!(to_ordinal(2)?, "제이");
/// assert_eq!(to_ordinal(10)?, "제십");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidOrdinal`](crate::Error::InvalidOrdinal) for zero, a
/// negative or a fraction; [`Error::OutOfRange`](crate::Error::OutOfRange) at
/// or past [`ORDINAL_MAX`].
pub fn to_ordinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    let n = parse_ordinal(&value.into())?;
    ORDINAL_MAX.check(&n, None)?;
    Ok(integer_to_ordinal(&n))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::KRW, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in South Korean won.
///
/// The won has no everyday subunit (jeon are historical): a fractional
/// amount is an error rather than being silently rounded away.
///
/// ```
/// # use n2words::ko_kr::to_currency;
/// assert_eq!(to_currency(42)?, "사십이원");
/// assert_eq!(to_currency(1000)?, "천원");
/// assert_eq!(to_currency(-5)?, "마이너스 오원");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value, [`Error::OutOfRange`](crate::Error::OutOfRange) at or past
/// [`CURRENCY_MAX`], [`Error::NoMinorUnit`](crate::Error::NoMinorUnit) for a
/// fractional amount.
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency Korean has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Korean has no words for.
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    let words = CURRENCY_VOCAB.words(options.currency)?;
    let money = parse_currency(&value.into(), options.currency.minor_unit_digits())?;
    CURRENCY_MAX.check(&money.major, None)?;
    assert_minor_unit(money.minor, options.currency)?;

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }
    result.push_str(&integer_to_words(&money.major));
    result.push_str(words.major_at(0));
    Ok(result)
}

/// Registry entry for `ko-KR`.
pub static LANGUAGE: Language = Language {
    code: "ko-KR",
    name: "Korean (South Korea)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
