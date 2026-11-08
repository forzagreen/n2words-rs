//! Georgian (Georgia) (`ka-GE`).
//!
//! Georgian-specific rules:
//! - Vigesimal (base-20) system for 20-99
//! - 40 = ორმოცი (2×20), 60 = სამოცი (3×20), 80 = ოთხმოცი (4×20)
//! - 30/50/70/90 use "და" + "ათი": ოცდაათი (20+10), ორმოცდაათი (40+10)
//! - Compound numbers use "და" (da = "and") connector
//! - Hundreds: unit prefix + ას (ორასი = 200)
//! - Short scale for large numbers (მილიონი, მილიარდი, ტრილიონი)
//!
//! ```
//! use n2words::ka_ge::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "ორმოცდაორი");
//! assert_eq!(to_ordinal(42)?, "მეორმოცდაორე");
//! assert_eq!(to_currency(42.5)?, "ორმოცდაორი ლარი ორმოცდაათი თეთრი");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::KA as CURRENCY_VOCAB;
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

// Numbers 0-9 (primitives)
const ONES: [&str; 10] = [
    "ნული",
    "ერთი",
    "ორი",
    "სამი",
    "ოთხი",
    "ხუთი",
    "ექვსი",
    "შვიდი",
    "რვა",
    "ცხრა",
];

// Numbers 10-19
const TEENS: [&str; 10] = [
    "ათი",
    "თერთმეტი",
    "თორმეტი",
    "ცამეტი",
    "თოთხმეტი",
    "თხუთმეტი",
    "თექვსმეტი",
    "ჩვიდმეტი",
    "თვრამეტი",
    "ცხრამეტი",
];

// Vigesimal bases: 20, 40, 60, 80 (with და connector forms)
const VIGESIMAL: [&str; 5] = ["", "ოცი", "ორმოცი", "სამოცი", "ოთხმოცი"];
const VIGESIMAL_DA: [&str; 5] = ["", "ოცდა", "ორმოცდა", "სამოცდა", "ოთხმოცდა"];

// Prefixes for hundreds (unit forms without final vowel)
const HUNDRED_PREFIXES: [&str; 10] = [
    "",
    "",
    "ორ",
    "სამ",
    "ოთხ",
    "ხუთ",
    "ექვს",
    "შვიდ",
    "რვა",
    "ცხრა",
];

const HUNDRED: &str = "ასი";
const HUNDRED_STEM: &str = "ას"; // Without final vowel
const THOUSAND: &str = "ათასი";
const THOUSAND_STEM: &str = "ათას"; // Without final vowel

// Scale words (short scale) - indexed by segment position
const SCALES: [&str; 8] = [
    "",
    "",
    "მილიონი",
    "მილიარდი",
    "ტრილიონი",
    "კვადრილიონი",
    "კვინტილიონი",
    "სექსტილიონი",
];

// 3-digit grouping; the table tops out at sextillion. Past it the builder
// clamps to the last scale word, silently collapsing the magnitude — so cap
// there.

/// The cardinal ceiling, derived from the scale table: `10^24`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32 - 1);
/// The ordinal ceiling: `10^24` (ordinals build on the cardinal speller).
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32 - 1);
/// The currency ceiling: `10^24`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32 - 1);

const ZERO: &str = "ნული";
const NEGATIVE: &str = "მინუს";
const DECIMAL_SEP: &str = "მთელი";

// Ordinal prefix and suffix (Georgian forms ordinals as მე- + stem + -ე)
const ORDINAL_PREFIX: &str = "მე";
const ORDINAL_SUFFIX: &str = "ე";

// Ordinal forms for 1-9 (these are special)
const ORDINAL_ONES: [&str; 10] = [
    "",
    "პირველი",
    "მეორე",
    "მესამე",
    "მეოთხე",
    "მეხუთე",
    "მეექვსე",
    "მეშვიდე",
    "მერვე",
    "მეცხრე",
];

// ============================================================================
// Segment building
// ============================================================================

/// The word without its final vowel (ი or ა), the form used before a scale
/// word; a word ending in any other letter is returned unchanged.
fn stem_of(word: &str) -> &str {
    word.strip_suffix('ი')
        .or_else(|| word.strip_suffix('ა'))
        .unwrap_or(word)
}

/// Words for 0-99 using the vigesimal system.
fn build_tens(n: u32) -> String {
    if n < 10 {
        return ONES[n as usize].to_string();
    }
    if n < 20 {
        return TEENS[(n - 10) as usize].to_string();
    }

    let vigesimal_group = (n / 20) as usize;
    let remainder = n % 20;

    if remainder == 0 {
        return VIGESIMAL[vigesimal_group].to_string();
    }

    // Use და connector: ოცდა + remainder
    let base = VIGESIMAL_DA[vigesimal_group];
    if remainder < 10 {
        return format!("{base}{}", ONES[remainder as usize]);
    }
    format!("{base}{}", TEENS[(remainder - 10) as usize])
}

/// Words for a 0-999 segment: the full form and the stem form (without the
/// final vowel), used before ათასი.
fn build_segment(n: u32) -> (String, String) {
    if n == 0 {
        return (String::new(), String::new());
    }
    if n < 100 {
        let word = build_tens(n);
        // Remove final vowel for stem
        let stem = stem_of(&word).to_string();
        return (word, stem);
    }

    let hundreds = (n / 100) as usize;
    let remainder = n % 100;

    // Build hundreds: ასი (100), ორასი (200), etc.
    let hundred_form = if remainder > 0 { HUNDRED_STEM } else { HUNDRED };
    let hundred_word = if hundreds == 1 {
        hundred_form.to_string()
    } else {
        format!("{}{hundred_form}", HUNDRED_PREFIXES[hundreds])
    };

    if remainder > 0 {
        let remainder_word = build_tens(remainder);
        // Stem removes final vowel from remainder
        let stem = format!("{hundred_word} {}", stem_of(&remainder_word));
        return (format!("{hundred_word} {remainder_word}"), stem);
    }

    // Hundreds only - stem removes final ი
    let stem = hundred_word[..hundred_word.len() - 'ი'.len_utf8()].to_string();
    (hundred_word, stem)
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    if let Some(small) = n.to_u64() {
        // Fast path: numbers < 1000
        if small < 1000 {
            return build_segment(small as u32).0;
        }

        // Fast path: numbers < 1,000,000 (thousands)
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            let thousand_word = if remainder > 0 {
                THOUSAND_STEM
            } else {
                THOUSAND
            };
            let mut result = if thousands == 1 {
                // "ათასი" not "ერთი ათასი"
                thousand_word.to_string()
            } else {
                // Use stem form before ათასი
                format!("{} {thousand_word}", build_segment(thousands).1)
            };

            if remainder > 0 {
                result.push(' ');
                result.push_str(&build_segment(remainder).0);
            }

            return result;
        }
    }

    // For numbers >= 1,000,000, use scale decomposition
    build_large_number_words(n)
}

/// Words for one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    let mut parts: Vec<String> = Vec::new();
    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }

        if scale_index == 0 {
            // Units (no scale)
            parts.push(build_segment(segment).0);
        } else if scale_index == 1 {
            // Thousands - check if there's a remainder
            let has_remainder = segments[0] != 0;
            let thousand_word = if has_remainder {
                THOUSAND_STEM
            } else {
                THOUSAND
            };

            if segment == 1 {
                parts.push(thousand_word.to_string());
            } else {
                parts.push(format!("{} {thousand_word}", build_segment(segment).1));
            }
        } else {
            // Million and above
            let scale_word = SCALES
                .get(scale_index)
                .copied()
                .unwrap_or(SCALES[SCALES.len() - 1]);
            if segment == 1 {
                parts.push(format!("{} {scale_word}", ONES[1]));
            } else {
                parts.push(format!("{} {scale_word}", build_segment(segment).0));
            }
        }
    }

    parts.join(" ")
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut result = String::new();

    // Handle leading zeros
    let zeros = decimal.bytes().take_while(|&b| b == b'0').count();
    for _ in 0..zeros {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(ZERO);
    }

    // Convert remainder as a single number
    let remainder = &decimal[zeros..];
    if !remainder.is_empty() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&digits_to_biguint(remainder)));
    }

    result
}

/// Spells a number in Georgian words.
///
/// ```
/// # use n2words::ka_ge::to_cardinal;
/// assert_eq!(to_cardinal(21)?, "ოცდაერთი");
/// assert_eq!(to_cardinal(100)?, "ასი");
/// assert_eq!(to_cardinal(1000)?, "ათასი");
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
/// Georgian ordinals are formed by:
/// - 1-9: special forms (პირველი, მეორე, მესამე, etc.)
/// - 10+: მე- prefix + cardinal stem + -ე suffix
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64().filter(|&small| small <= 9) {
        return ORDINAL_ONES[small as usize].to_string();
    }

    // For 10+, use მე- + cardinal stem + -ე
    let cardinal = integer_to_words(n);

    // Remove final vowel and add -ე suffix
    format!("{ORDINAL_PREFIX}{}{ORDINAL_SUFFIX}", stem_of(&cardinal))
}

/// Spells a positive integer as a Georgian ordinal.
///
/// ```
/// # use n2words::ka_ge::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "პირველი");
/// assert_eq!(to_ordinal(10)?, "მეათე");
/// assert_eq!(to_ordinal(21)?, "მეოცდაერთე");
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
        currency: Currency = Currency::GEL, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Georgian lari.
///
/// ```
/// # use n2words::ka_ge::to_currency;
/// assert_eq!(to_currency(1)?, "ერთი ლარი");
/// assert_eq!(to_currency(2.5)?, "ორი ლარი ორმოცდაათი თეთრი");
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

/// Spells an amount in any currency Georgian has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Georgian has no words for and
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

    let mut parts: Vec<String> = Vec::new();

    if money.negative {
        parts.push(NEGATIVE.to_string());
    }

    // Lari (invariable)
    if !money.major.is_zero() || money.minor == 0 {
        parts.push(format!(
            "{} {}",
            integer_to_words(&money.major),
            words.major_at(0)
        ));
    }

    // Tetri (invariable)
    if money.minor > 0 {
        parts.push(format!(
            "{} {}",
            integer_to_words(&BigUint::from(money.minor)),
            words.minor_at(0)
        ));
    }

    Ok(parts.join(" "))
}

/// Registry entry for `ka-GE`.
pub static LANGUAGE: Language = Language {
    code: "ka-GE",
    name: "Georgian (Georgia)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
