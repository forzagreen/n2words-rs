//! Danish (Denmark) (`da-DK`).
//!
//! - Vigesimal (base-20) tens naming: halvtreds (50), treds (60), etc.
//! - Units-before-tens: "enogtyve" (21) = one-and-twenty
//! - Compound thousands: "ettusind", "firetusinde"
//! - "og" conjunction after hundreds and thousands
//! - Long scale for millions+
//!
//! ```
//! use n2words::da_dk::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "toogfyrre");
//! assert_eq!(to_ordinal(42)?, "toogfyrrede");
//! assert_eq!(to_currency(42.5)?, "toogfyrre kroner og halvtreds øre");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::DA as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{
    Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal, split_groups,
};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

const ONES: [&str; 10] = [
    "", "et", "to", "tre", "fire", "fem", "seks", "syv", "otte", "ni",
];
// "en" form used in vigesimal pattern (X og Y) and before millions
const ONES_VIGESIMAL: [&str; 10] = [
    "", "en", "to", "tre", "fire", "fem", "seks", "syv", "otte", "ni",
];

const TEENS: [&str; 10] = [
    "ti", "elleve", "tolv", "tretten", "fjorten", "femten", "seksten", "sytten", "atten", "nitten",
];

// Danish vigesimal tens (base-20 derived names)
const TENS: [&str; 10] = [
    "",
    "",
    "tyve",
    "tredive",
    "fyrre",
    "halvtreds",
    "treds",
    "halvfjerds",
    "firs",
    "halvfems",
];

const HUNDRED: &str = "hundrede";
const THOUSAND: &str = "tusind";

const ZERO: &str = "nul";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "komma";

// Long scale: millioner, millarder, billioner, etc.
const SCALES: &[&str] = &[
    "millioner",
    "millarder",
    "billioner",
    "billarder",
    "trillioner",
    "trillarder",
    "quadrillioner",
    "quadrillarder",
];

// Supported magnitude ceiling (checked at the public entry points). Segments
// are [units, thousands, then one per SCALES entry], so cardinals span at most
// SCALES.len() + 2 groups of 3 digits — they must stay below 10^30. Ordinals
// and currency build on the cardinal, so they share the same ceiling.

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32 + 1);
/// The ordinal ceiling: `10^30`.
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32 + 1);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32 + 1);

// ============================================================================
// Ordinal vocabulary
// ============================================================================

// Danish ordinals: 1st-2nd special, others use -te/-nde suffix
// "anden/andet" for 2nd (common/neuter), we use common form
const ORDINAL_SPECIAL: [&str; 13] = [
    "", "første", "anden", "tredje", "fjerde", "femte", "sjette", "syvende", "ottende", "niende",
    "tiende", "ellevte", "tolvte",
];

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<String> = Vec::new();

    // Hundreds: "ethundrede", "tohundrede" (compound, no space)
    if hundreds > 0 {
        parts.push(format!("{}{HUNDRED}", ONES[hundreds]));
    }

    // Tens and ones
    let tens_ones = n % 100;

    if tens_ones == 0 {
        // Just hundreds
    } else if tens_ones < 10 {
        // Single digit
        parts.push(ONES[ones].to_string());
    } else if tens_ones < 20 {
        // Teens
        parts.push(TEENS[ones].to_string());
    } else if ones == 0 {
        // Even tens
        parts.push(TENS[tens].to_string());
    } else {
        // Units-before-tens: "enogtyve", "treogfyrre"
        parts.push(format!("{}og{}", ONES_VIGESIMAL[ones], TENS[tens]));
    }

    // Combine with " og " between hundreds and remainder
    parts.join(" og ")
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
        // Fast path: numbers < 1000 (direct lookup)
        if small < 1000 {
            return build_segment(small as u32);
        }

        // Fast path: numbers < 1,000,000 (thousands)
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            // Compound thousands: "ettusind", "firetusind"
            let mut result = build_segment(thousands);
            result.push_str(THOUSAND);

            if remainder > 0 {
                // Add 'e' suffix and " og " for remainder: "firetusinde og ..."
                result.push_str("e og ");
                result.push_str(&build_segment(remainder));
            }

            return result;
        }
    }

    // For numbers >= 1,000,000, use scale decomposition
    build_large_number_words(n)
}

/// Kind of a scale part, for the joining rules.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PartKind {
    Units,
    Thousand,
    Million,
}

/// Words for one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    // Convert segments to words with scale tracking.
    // Callers guard the magnitude (CARDINAL_MAX) before reaching here.
    // scale_index: 0 = units, 1 = thousands, 2 = millions, etc.
    let mut parts: Vec<(String, PartKind)> = Vec::new();
    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let segment_word = build_segment(segment);

        if scale_index == 0 {
            // Units segment
            parts.push((segment_word, PartKind::Units));
        } else if scale_index == 1 {
            // Thousands - compound form
            parts.push((format!("{segment_word}{THOUSAND}"), PartKind::Thousand));
        } else {
            // Millions+ - space-separated, use "en" for 1
            let scale_word = SCALES[scale_index - 2];
            // "et" → "en" before millions+
            let num_word = if segment == 1 { "en" } else { &segment_word };
            parts.push((format!("{num_word} {scale_word}"), PartKind::Million));
        }
    }

    // Join parts with Danish rules
    join_danish_parts(&parts)
}

/// Joins parts with Danish spacing rules.
/// - After thousands with remainder: "tusinde og"
/// - Millions are space-separated
fn join_danish_parts(parts: &[(String, PartKind)]) -> String {
    if parts.is_empty() {
        return ZERO.to_string();
    }

    let mut tokens: Vec<String> = Vec::new();
    let mut i = 0;
    while i < parts.len() {
        let (word, kind) = &parts[i];
        match parts.get(i + 1) {
            Some((next_word, PartKind::Units)) if *kind == PartKind::Thousand => {
                // Thousands directly followed by the units segment: compound "…tusinde og …"
                tokens.push(format!("{word}e og {next_word}"));
                i += 1; // consumed the units part
            }
            _ => tokens.push(word.clone()),
        }
        i += 1;
    }

    // Every remaining boundary (between scale groups) is a single space.
    tokens.join(" ")
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut words: Vec<String> = Vec::new();

    // Handle leading zeros
    let significant = decimal.trim_start_matches('0');
    for _ in 0..decimal.len() - significant.len() {
        words.push(ZERO.to_string());
    }

    // Convert remainder as a single number
    if !significant.is_empty() {
        words.push(integer_to_words(&digits_to_biguint(significant)));
    }

    words.join(" ")
}

/// Spells a number in Danish words.
///
/// ```
/// # use n2words::da_dk::to_cardinal;
/// assert_eq!(to_cardinal(21)?, "enogtyve");
/// assert_eq!(to_cardinal(1000)?, "ettusind");
/// assert_eq!(to_cardinal(1_000_000)?, "en millioner");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value, [`Error::OutOfRange`](crate::Error::OutOfRange) at or past
/// [`CARDINAL_MAX`] (for the integer part or the decimal digits).
pub fn to_cardinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    let number = parse_cardinal(&value.into())?;
    // Both the integer part and the decimal's significant digits are spelled via
    // the scale builder, so both must clear the ceiling.
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
/// Danish ordinals: første (1st), anden (2nd), tredje (3rd), etc.
/// 1-12 have special forms, others use cardinal + -de suffix.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special forms for 1-12
    if let Some(small) = n.to_u64().filter(|small| (1..=12).contains(small)) {
        return ORDINAL_SPECIAL[small as usize].to_string();
    }

    // For numbers > 12, add -de suffix to cardinal
    let mut cardinal = integer_to_words(n);
    cardinal.push_str("de");
    cardinal
}

/// Spells a positive integer as a Danish ordinal.
///
/// ```
/// # use n2words::da_dk::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "første");
/// assert_eq!(to_ordinal(2)?, "anden");
/// assert_eq!(to_ordinal(21)?, "enogtyvede");
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
        currency: Currency = Currency::DKK, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Danish kroner.
///
/// Uses krone/kroner and øre (100 øre = 1 krone).
///
/// ```
/// # use n2words::da_dk::to_currency;
/// assert_eq!(to_currency(1)?, "en krone");
/// assert_eq!(to_currency(42)?, "toogfyrre kroner");
/// assert_eq!(to_currency(1.5)?, "en krone og halvtreds øre");
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

/// Spells an amount in any currency Danish has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Danish has no words for and
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

    // Kroner part - use "en" for 1 krone
    if !money.major.is_zero() || money.minor == 0 {
        if money.major.is_one() {
            result.push_str("en ");
            result.push_str(words.major_at(0));
        } else {
            result.push_str(&integer_to_words(&money.major));
            result.push(' ');
            result.push_str(words.major_at(1));
        }
    }

    // Øre part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(" og ");
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(0));
    }

    Ok(result)
}

/// Registry entry for `da-DK`.
pub static LANGUAGE: Language = Language {
    code: "da-DK",
    name: "Danish (Denmark)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
