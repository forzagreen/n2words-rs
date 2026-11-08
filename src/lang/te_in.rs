//! Telugu (India) (`te-IN`).
//!
//! - Indian numbering system (వెయ్యి, లక్ష, కోటి)
//! - Telugu script
//! - 3-2-2 grouping pattern
//! - Complete word forms for 0-99
//! - Per-digit decimal reading
//!
//! ```
//! use n2words::te_in::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(1234)?, "ఒక వెయ్యి రెండు వందలు ముప్పై నాలుగు");
//! assert_eq!(to_ordinal(10)?, "పదివ");
//! assert_eq!(to_currency(42.5)?, "నలభై రెండు రూపాయలు యాభై పైసలు");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::TE as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, parse_cardinal, parse_currency, parse_ordinal, split_groups_indian};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

const ZERO: &str = "సున్నా";
const NEGATIVE: &str = "మైనస్";
const DECIMAL_SEP: &str = "పాయింట్";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

/// Ordinal suffix (added to the cardinal for numbers >= 7).
const ORDINAL_SUFFIX: &str = "వ";

/// Special ordinals for the first few numbers.
const ORDINAL_SPECIAL: [&str; 7] = ["", "మొదటి", "రెండవ", "మూడవ", "నాలుగవ", "ఐదవ", "ఆరవ"];

#[rustfmt::skip]
const BELOW_HUNDRED: [&str; 100] = [
    "సున్నా", "ఒకటి", "రెండు", "మూడు", "నాలుగు", "ఐదు", "ఆరు", "ఏడు", "ఎనిమిది", "తొమ్మిది",
    "పది", "పదకొండు", "పన్నెండు", "పదమూడు", "పద్నాలుగు", "పదిహేను", "పదహారు", "పదిహేడు", "పద్దెనిమిది", "పంతొమ్మిది",
    "ఇరవై", "ఇరవై ఒక్కటి", "ఇరవై రెండు", "ఇరవై మూడు", "ఇరవై నాలుగు", "ఇరవై ఐదు", "ఇరవై ఆరు", "ఇరవై ఏడు", "ఇరవై ఎనిమిది", "ఇరవై తొమ్మిది",
    "ముప్పై", "ముప్పై ఒకటి", "ముప్పై రెండు", "ముప్పై మూడు", "ముప్పై నాలుగు", "ముప్పై ఐదు", "ముప్పై ఆరు", "ముప్పై ఏడు", "ముప్పై ఎనిమిది", "ముప్పై తొమ్మిది",
    "నలభై", "నలభై ఒకటి", "నలభై రెండు", "నలభై మూడు", "నలభై నాలుగు", "నలభై ఐదు", "నలభై ఆరు", "నలభై ఏడు", "నలభై ఎనిమిది", "నలభై తొమ్మిది",
    "యాభై", "యాభై ఒకటి", "యాభై రెండు", "యాభై మూడు", "యాభై నాలుగు", "యాభై ఐదు", "యాభై ఆరు", "యాభై ఏడు", "యాభై ఎనిమిది", "యాభై తొమ్మిది",
    "అరవై", "అరవై ఒకటి", "అరవై రెండు", "అరవై మూడు", "అరవై నాలుగు", "అరవై ఐదు", "అరవై ఆరు", "అరవై ఏడు", "అరవై ఎనిమిది", "అరవై తొమ్మిది",
    "డెబ్బై", "డెబ్బై ఒకటి", "డెబ్బై రెండు", "డెబ్బై మూడు", "డెబ్బై నాలుగు", "డెబ్బై ఐదు", "డెబ్బై ఆరు", "డెబ్బై ఏడు", "డెబ్బై ఎనిమిది", "డెబ్బై తొమ్మిది",
    "ఎనభై", "ఎనభై ఒకటి", "ఎనభై రెండు", "ఎనభై మూడు", "ఎనభై నాలుగు", "ఎనభై ఐదు", "ఎనభై ఆరు", "ఎనభై ఏడు", "ఎనభై ఎనిమిది", "ఎనభై తొమ్మిది",
    "తొంభై", "తొంభై ఒకటి", "తొంభై రెండు", "తొంభై మూడు", "తొంభై నాలుగు", "తొంభై ఐదు", "తొంభై ఆరు", "తొంభై ఏడు", "తొంభై ఎనిమిది", "తొంభై తొమ్మిది",
];

const HUNDREDS: [&str; 10] = [
    "",
    "వంద",
    "రెండు వందలు",
    "మూడు వందలు",
    "నాలుగు వందలు",
    "ఐదు వందలు",
    "ఆరు వందలు",
    "ఏడు వందలు",
    "ఎనిమిది వందలు",
    "తొమ్మిది వందలు",
];

/// Ones for decimal reading.
const ONES: [&str; 9] = [
    "ఒకటి",
    "రెండు",
    "మూడు",
    "నాలుగు",
    "ఐదు",
    "ఆరు",
    "ఏడు",
    "ఎనిమిది",
    "తొమ్మిది",
];

/// The word for 1 at a scale position ("one thousand", "one lakh", ...).
const ONE_AT_SCALE: &str = "ఒక";

/// Scale words: index 0 = units (empty), 1 = thousand, 2 = lakh, etc.
const SCALE_WORDS: &[&str] = &["", "వెయ్యి", "లక్ష", "కోటి", "అరబ్", "ఖరబ్", "నిల్", "పడ్మ", "శంకు"];

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
        return HUNDREDS[hundreds].to_string();
    }
    format!("{} {}", HUNDREDS[hundreds], BELOW_HUNDRED[remainder])
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
        } else if segment == 1 {
            // Use "ఒక" for 1 at scale positions.
            result.push_str(ONE_AT_SCALE);
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

/// Words for the decimal fraction, read digit by digit.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut result = String::new();
    for byte in decimal.bytes() {
        let word = match byte {
            b'0' => ZERO,
            b'1'..=b'9' => ONES[(byte - b'1') as usize],
            _ => continue,
        };
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    }
    result
}

/// Spells a number in Telugu words.
///
/// ```
/// # use n2words::te_in::to_cardinal;
/// assert_eq!(to_cardinal(999)?, "తొమ్మిది వందలు తొంభై తొమ్మిది");
/// assert_eq!(to_cardinal(3.14)?, "మూడు పాయింట్ ఒకటి నాలుగు");
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
    // The fraction is spelled digit by digit, so only the integer part has a ceiling.
    CARDINAL_MAX.check(&number.integer, None)?;

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
/// Telugu ordinals: the first 6 are irregular, then add the -వ suffix.
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

/// Spells a positive integer as a Telugu ordinal.
///
/// ```
/// # use n2words::te_in::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "మొదటి");
/// assert_eq!(to_ordinal(2)?, "రెండవ");
/// assert_eq!(to_ordinal(1001)?, "ఒక వెయ్యి ఒకటివ");
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
/// # use n2words::te_in::to_currency;
/// assert_eq!(to_currency(1)?, "ఒకటి రూపాయి");
/// assert_eq!(to_currency(0.01)?, "ఒకటి పైసా");
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

/// Spells an amount in any currency Telugu has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Telugu has no words for and
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

/// Registry entry for `te-IN`.
pub static LANGUAGE: Language = Language {
    code: "te-IN",
    name: "Telugu (India)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
