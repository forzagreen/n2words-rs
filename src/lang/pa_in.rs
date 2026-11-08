//! Punjabi (India) (`pa-IN`).
//!
//! - Indian numbering system (ਹਜ਼ਾਰ, ਲੱਖ, ਕਰੋੜ)
//! - Gurmukhi script
//! - 3-2-2 grouping pattern (last 3 digits, then groups of 2)
//! - Complete word forms for 0-99
//!
//! ```
//! use n2words::pa_in::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(215)?, "ਦੋ ਸੌ ਪੰਦਰਾਂ");
//! assert_eq!(to_ordinal(10)?, "ਦੱਸਵਾਂ");
//! assert_eq!(to_currency(42.5)?, "ਬਿਆਲੀ ਰੁਪਏ ਪੰਜਾਹ ਪੈਸੇ");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::PA as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{
    Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal, split_groups_indian,
};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

const ZERO: &str = "ਸਿਫ਼ਰ";
const NEGATIVE: &str = "ਮਾਇਨਸ";
const DECIMAL_SEP: &str = "ਦਸ਼ਮਲਵ";
const HUNDRED: &str = "ਸੌ";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

/// Ordinal suffix (added to the cardinal for numbers >= 7).
const ORDINAL_SUFFIX: &str = "ਵਾਂ";

/// Special ordinals for the first few numbers (1-6 have irregular forms).
const ORDINAL_SPECIAL: [&str; 7] = ["", "ਪਹਿਲਾ", "ਦੂਜਾ", "ਤੀਜਾ", "ਚੌਥਾ", "ਪੰਜਵਾਂ", "ਛੇਵਾਂ"];

#[rustfmt::skip]
const BELOW_HUNDRED: [&str; 100] = [
    "ਸਿਫ਼ਰ", "ਇੱਕ", "ਦੋ", "ਤਿੰਨ", "ਚਾਰ", "ਪੰਜ", "ਛੇ", "ਸੱਤ", "ਅੱਠ", "ਨੌਂ",
    "ਦੱਸ", "ਗਿਆਰਾਂ", "ਬਾਰਾਂ", "ਤੇਰਾਂ", "ਚੌਦਾਂ", "ਪੰਦਰਾਂ", "ਸੋਲਾਂ", "ਸਤਾਰਾਂ", "ਅਠਾਰਾਂ", "ਉੱਨੀ",
    "ਵੀਹ", "ਇੱਕੀ", "ਬਾਈ", "ਤੇਈ", "ਚੌਬੀ", "ਪੱਚੀ", "ਛੱਬੀ", "ਸਤਾਈ", "ਅਠਾਈ", "ਉਨੱਤੀ",
    "ਤੀਹ", "ਇਕੱਤੀ", "ਬੱਤੀ", "ਤੇਤੀ", "ਚੌਂਤੀ", "ਪੈਂਤੀ", "ਛੱਤੀ", "ਸੈਂਤੀ", "ਅਠੱਤੀ", "ਉਨਤਾਲੀ",
    "ਚਾਲੀ", "ਇਕਤਾਲੀ", "ਬਿਆਲੀ", "ਤਿਰਤਾਲੀ", "ਚੁਵਾਲੀ", "ਪੰਤਾਲੀ", "ਛਿਆਲੀ", "ਸੈਂਤਾਲੀ", "ਅਠਤਾਲੀ", "ਉਨੰਜਾ",
    "ਪੰਜਾਹ", "ਇਕਵੰਜਾ", "ਬਵੰਜਾ", "ਤਰਵੰਜਾ", "ਚੁਰਵੰਜਾ", "ਪੰਜਵੰਜਾ", "ਛਪੰਜਾ", "ਸੱਤਵੰਜਾ", "ਅਠਵੰਜਾ", "ਉਨਾਹਠ",
    "ਸੱਠ", "ਇਕਾਹਠ", "ਬਾਹਠ", "ਤਰਸਠ", "ਚੌਂਸਠ", "ਪੈਂਸਠ", "ਛਿਆਸਠ", "ਸੜਸਠ", "ਅੜਸਠ", "ਉਣਹੱਤਰ",
    "ਸਤੱਰ", "ਇਕਹੱਤਰ", "ਬਹੱਤਰ", "ਤਹੱਤਰ", "ਚੌਹੱਤਰ", "ਪੰਝਹੱਤਰ", "ਛਿਹੱਤਰ", "ਸਤੱਤਰ", "ਅਠੱਤਰ", "ਉਨਾਸੀ",
    "ਅੱਸੀ", "ਇਕਿਆਸੀ", "ਬਿਆਸੀ", "ਤਰਿਆਸੀ", "ਚੌਰਿਆਸੀ", "ਪਚਾਸੀ", "ਛਿਆਸੀ", "ਸੱਤਾਸੀ", "ਅਠਾਸੀ", "ਨਵਾਸੀ",
    "ਨੱਬੇ", "ਇਕਾਨਵੇਂ", "ਬਾਨਵੇਂ", "ਤਰਾਨਵੇਂ", "ਚੁਰਾਨਵੇਂ", "ਪੰਚਾਨਵੇਂ", "ਛਿਆਨਵੇਂ", "ਸਤਾਨਵੇਂ", "ਅਠਾਨਵੇਂ", "ਨਿਨਾਨਵੇਂ",
];

/// Scale words: index 0 = units (empty), 1 = thousand, 2 = lakh, 3 = crore, etc.
const SCALE_WORDS: &[&str] = &["", "ਹਜ਼ਾਰ", "ਲੱਖ", "ਕਰੋੜ", "ਅਰਬ", "ਖਰਬ", "ਨੀਲ", "ਪਦਮ", "ਸ਼ੰਖ"];

// 3-2-2 Indian grouping: a 3-digit base segment, then 2 digits per scale word
// (SCALE_WORDS[0] = "" is the units slot). Past the table the scale word is
// dropped, which collapses the magnitude — so cap there.

/// The cardinal ceiling, derived from the scale table: `10^19`.
pub const CARDINAL_MAX: Max = Max::indian(SCALE_WORDS.len() as u32);
/// The ordinal ceiling: `10^19`.
pub const ORDINAL_MAX: Max = Max::indian(SCALE_WORDS.len() as u32);
/// The currency ceiling: `10^19`.
pub const CURRENCY_MAX: Max = Max::indian(SCALE_WORDS.len() as u32);

// ============================================================================
// Segment Building
// ============================================================================

/// Words for a 0-999 segment.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    if n < 100 {
        return BELOW_HUNDRED[n as usize].to_string();
    }

    let hundreds = (n / 100) as usize;
    let remainder = (n % 100) as usize;

    if remainder == 0 {
        return format!("{} {HUNDRED}", BELOW_HUNDRED[hundreds]);
    }
    format!(
        "{} {HUNDRED} {}",
        BELOW_HUNDRED[hundreds], BELOW_HUNDRED[remainder]
    )
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Words for a non-negative integer.
///
/// South Asian 3-2-2 grouping: first 3 digits, then groups of 2.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Fast path: numbers < 1000 (direct lookup).
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32);
        }
    }

    // Segments least-significant first: the 3-digit base, then 2 digits each.
    let segments = split_groups_indian(n, 1000, 100);

    // Build the result from most-significant to least.
    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if !result.is_empty() {
            result.push(' ');
        }

        if i == 0 {
            result.push_str(&build_segment(segment));
        } else {
            result.push_str(BELOW_HUNDRED[segment as usize]);
        }

        if i > 0 {
            if let Some(scale) = SCALE_WORDS.get(i).filter(|s| !s.is_empty()) {
                result.push(' ');
                result.push_str(scale);
            }
        }
    }

    result
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut result = String::new();
    let zeros = decimal.bytes().take_while(|&b| b == b'0').count();
    for _ in 0..zeros {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(ZERO);
    }

    let remainder = &decimal[zeros..];
    if !remainder.is_empty() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&digits_to_biguint(remainder)));
    }

    result
}

/// Spells a number in Punjabi words.
///
/// ```
/// # use n2words::pa_in::to_cardinal;
/// assert_eq!(to_cardinal(1000000)?, "ਦੱਸ ਲੱਖ");
/// assert_eq!(to_cardinal(-17)?, "ਮਾਇਨਸ ਸਤਾਰਾਂ");
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
    // Both the integer part and the decimal's significant digits are spelled
    // via the scale builder, so both must clear the ceiling.
    CARDINAL_MAX.check(&number.integer, number.fraction.as_deref())?;

    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }
    result.push_str(&integer_to_words(&number.integer));
    if let Some(fraction) = &number.fraction {
        result.push(' ');
        result.push_str(DECIMAL_SEP);
        result.push(' ');
        result.push_str(&decimal_part_to_words(fraction));
    }
    Ok(result)
}

// ============================================================================
// Ordinal
// ============================================================================

/// Ordinal words for a positive integer.
///
/// Punjabi ordinals: the first 6 are irregular, then add the -ਵਾਂ suffix.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special ordinals for 1-6.
    if let Some(small) = n.to_u64() {
        if (1..=6).contains(&small) {
            return ORDINAL_SPECIAL[small as usize].to_string();
        }
    }

    // For 7 and above, add the suffix to the cardinal.
    let mut result = integer_to_words(n);
    result.push_str(ORDINAL_SUFFIX);
    result
}

/// Spells a positive integer as a Punjabi ordinal.
///
/// ```
/// # use n2words::pa_in::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "ਪਹਿਲਾ");
/// assert_eq!(to_ordinal(3)?, "ਤੀਜਾ");
/// assert_eq!(to_ordinal(1001)?, "ਇੱਕ ਹਜ਼ਾਰ ਇੱਕਵਾਂ");
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
    // Ordinals build on the cardinal speller, so they share its ceiling.
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
        currency: Currency = Currency::INR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Indian rupees.
///
/// ```
/// # use n2words::pa_in::to_currency;
/// assert_eq!(to_currency(1)?, "ਇੱਕ ਰੁਪਇਆ");
/// assert_eq!(to_currency(0.01)?, "ਇੱਕ ਪੈਸਾ");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value, [`Error::OutOfRange`](crate::Error::OutOfRange) at or past
/// [`CURRENCY_MAX`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency Punjabi has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Punjabi has no words for and
/// [`Error::NoMinorUnit`](crate::Error::NoMinorUnit) for a fractional amount
/// in a currency without one.
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

    // Rupees part: shown if non-zero, or if there are no paise.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        // Singular for 1 rupee, plural otherwise.
        result.push_str(words.major_at(if money.major.is_one() { 0 } else { 1 }));
    }

    // Paise part.
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        // Singular for 1 paisa, plural otherwise.
        result.push_str(words.minor_at(if money.minor == 1 { 0 } else { 1 }));
    }

    Ok(result)
}

/// Registry entry for `pa-IN`.
pub static LANGUAGE: Language = Language {
    code: "pa-IN",
    name: "Punjabi (India)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
