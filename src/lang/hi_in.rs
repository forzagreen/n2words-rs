//! Hindi (India) (`hi-IN`).
//!
//! - Indian numbering system (हज़ार, लाख, करोड़)
//! - Devanagari script
//! - 3-2-2 grouping pattern (last 3 digits, then groups of 2)
//! - Complete word forms for 0-99
//! - Ordinals: irregular forms for 1-6, then the -वाँ suffix on the cardinal
//!
//! ```
//! use n2words::hi_in::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "बयालीस");
//! assert_eq!(to_ordinal(10)?, "दसवाँ");
//! assert_eq!(to_currency(42.5)?, "बयालीस रुपये पचास पैसे");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::HI as CURRENCY_VOCAB;
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

const ZERO: &str = "शून्य";
const NEGATIVE: &str = "माइनस";
const DECIMAL_SEP: &str = "दशमलव";
const HUNDRED: &str = "सौ";

// ============================================================================
// Ordinal vocabulary
// ============================================================================

/// Ordinal suffix (adds to cardinal).
const ORDINAL_SUFFIX: &str = "वाँ";

/// Special ordinals for the first few numbers.
const ORDINAL_SPECIAL: [&str; 7] = ["", "पहला", "दूसरा", "तीसरा", "चौथा", "पाँचवाँ", "छठा"];

#[rustfmt::skip]
const BELOW_HUNDRED: [&str; 100] = [
    "शून्य", "एक", "दो", "तीन", "चार", "पाँच", "छह", "सात", "आठ", "नौ",
    "दस", "ग्यारह", "बारह", "तेरह", "चौदह", "पंद्रह", "सोलह", "सत्रह", "अठारह", "उन्नीस",
    "बीस", "इक्कीस", "बाईस", "तेईस", "चौबीस", "पच्चीस", "छब्बीस", "सत्ताईस", "अट्ठाईस", "उनतीस",
    "तीस", "इकतीस", "बत्तीस", "तैंतीस", "चौंतीस", "पैंतीस", "छत्तीस", "सैंतीस", "अड़तीस", "उनतालीस",
    "चालीस", "इकतालीस", "बयालीस", "तेतालीस", "चवालीस", "पैंतालीस", "छियालीस", "सैंतालीस", "अड़तालीस", "उनचास",
    "पचास", "इक्यावन", "बावन", "तिरपन", "चौवन", "पचपन", "छप्पन", "सत्तावन", "अट्ठावन", "उनसठ",
    "साठ", "इकसठ", "बासठ", "तिरसठ", "चौंसठ", "पैंसठ", "छियासठ", "सड़सठ", "अड़सठ", "उनहत्तर",
    "सत्तर", "इकहत्तर", "बहत्तर", "तिहत्तर", "चौहत्तर", "पचहत्तर", "छिहत्तर", "सतहत्तर", "अठहत्तर", "उन्यासी",
    "अस्सी", "इक्यासी", "बयासी", "तिरासी", "चौरासी", "पचासी", "छियासी", "सत्तासी", "अट्ठासी", "नवासी",
    "नब्बे", "इक्यानवे", "बानवे", "तिरानवे", "चौरानवे", "पचानवे", "छियानवे", "सत्तानवे", "अट्ठानवे", "निन्यानवे",
];

/// Scale words: index 0 = units (empty), 1 = thousand, 2 = lakh, 3 = crore, etc.
const SCALE_WORDS: &[&str] = &["", "हज़ार", "लाख", "करोड़", "अरब", "खरब", "नील", "पद्म", "शंख"];

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
// Segment building
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
// Cardinal
// ============================================================================

/// Words for a non-negative integer.
///
/// South Asian 3-2-2 grouping: first 3 digits, then groups of 2.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Fast path: numbers < 1000.
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32);
        }
    }

    // First segment: last 3 digits; remaining segments: 2 digits each
    // (lakh = 100k, crore = 10M, etc.). Least-significant first.
    let segments = split_groups_indian(n, 1000, 100);

    // Build from the most-significant segment down.
    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if !result.is_empty() {
            result.push(' ');
        }
        if i == 0 {
            // The units segment can be 0-999.
            result.push_str(&build_segment(segment));
        } else {
            // Other segments are 0-99.
            result.push_str(BELOW_HUNDRED[segment as usize]);
        }
        // Scale word for every segment but the units one.
        if i > 0 {
            if let Some(scale) = SCALE_WORDS.get(i).filter(|s| !s.is_empty()) {
                result.push(' ');
                result.push_str(scale);
            }
        }
    }
    result
}

/// Words for the digits after the decimal separator: leading zeros one by
/// one, then the rest as a single number.
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

/// Spells a number in Hindi words.
///
/// ```
/// # use n2words::hi_in::to_cardinal;
/// assert_eq!(to_cardinal(123456)?, "एक लाख तेईस हज़ार चार सौ छप्पन");
/// assert_eq!(to_cardinal(-17.42)?, "माइनस सत्रह दशमलव बयालीस");
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

/// Ordinal words for a positive integer: the first 6 are irregular, then the
/// -वाँ suffix is added to the cardinal.
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if (1..=6).contains(&small) {
            return ORDINAL_SPECIAL[small as usize].to_string();
        }
    }
    let mut result = integer_to_words(n);
    result.push_str(ORDINAL_SUFFIX);
    result
}

/// Spells a positive integer as a Hindi ordinal.
///
/// ```
/// # use n2words::hi_in::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "पहला");
/// assert_eq!(to_ordinal(1001)?, "एक हज़ार एकवाँ");
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
/// # use n2words::hi_in::to_currency;
/// assert_eq!(to_currency(1)?, "एक रुपया");
/// assert_eq!(to_currency(0.01)?, "एक पैसा");
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

/// Spells an amount in any currency Hindi has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Hindi has no words for and
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

/// Registry entry for `hi-IN`.
pub static LANGUAGE: Language = Language {
    code: "hi-IN",
    name: "Hindi (India)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
