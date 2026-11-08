//! Norwegian Bokmål (Norway) (`nb-NO`).
//!
//! - Hyphenated tens+ones: "tjue-en" (21)
//! - "og" conjunction after hundreds
//! - Comma separator after thousands before hundreds
//! - Short scale: million, milliard, billion, etc.
//!
//! ```
//! use n2words::nb_no::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "førti-to");
//! assert_eq!(to_ordinal(42)?, "førti-tode");
//! assert_eq!(to_currency(42.5)?, "førti-to kroner og femti øre");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::NB as CURRENCY_VOCAB;
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
    "", "en", "to", "tre", "fire", "fem", "seks", "syv", "åtte", "ni",
];

const TEENS: [&str; 10] = [
    "ti", "elleve", "tolv", "tretten", "fjorten", "femten", "seksten", "sytten", "atten", "nitten",
];
const TENS: [&str; 10] = [
    "", "", "tjue", "tretti", "førti", "femti", "seksti", "sytti", "åtti", "nitti",
];

const HUNDRED: &str = "hundre";
const THOUSAND: &str = "tusen";

const ZERO: &str = "null";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "komma";

// Short scale: million, milliard, billion, etc.
const SCALES: &[&str] = &[
    "million",
    "milliard",
    "billion",
    "billiard",
    "kvintillion",
    "sekstillion",
    "septillion",
    "oktillion",
];

// Supported magnitude ceiling (checked at the public entry points). SCALES is
// indexed [scale_index - 2] (units and thousands separate), so segments are
// [units, thousands, then one per SCALES entry] -> ceiling 10^((SCALES.len() +
// 2) * 3) = 10^30. Ordinal (cardinal + suffix) and currency build on the
// cardinal, and the decimal is spelled via integer_to_words, so all share it.

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32 + 1);
/// The ordinal ceiling: `10^30`.
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32 + 1);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32 + 1);

// ============================================================================
// Ordinal vocabulary
// ============================================================================

// Norwegian ordinals: 1st-12th special forms, then cardinal + -de/-te
const ORDINAL_SPECIAL: [&str; 13] = [
    "", "første", "andre", "tredje", "fjerde", "femte", "sjette", "sjuende", "åttende", "niende",
    "tiende", "ellevte", "tolvte",
];

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment, and whether it contains a hundred.
fn build_segment(n: u32) -> (String, bool) {
    if n == 0 {
        return (String::new(), false);
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<String> = Vec::new();
    let mut has_hundred = false;

    // Hundreds: "en hundre", "to hundre"
    if hundreds > 0 {
        has_hundred = true;
        parts.push(format!("{} {HUNDRED}", ONES[hundreds]));
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
        // Hyphenated: "tjue-en"
        parts.push(format!("{}-{}", TENS[tens], ONES[ones]));
    }

    // Combine with " og " between hundreds and remainder
    (parts.join(" og "), has_hundred)
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

            let mut result = build_segment(thousands).0;
            result.push(' ');
            result.push_str(THOUSAND);

            if remainder > 0 {
                let (remainder_word, has_hundred) = build_segment(remainder);
                // Comma before hundreds, " og " before small numbers
                result.push_str(if has_hundred { ", " } else { " og " });
                result.push_str(&remainder_word);
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

/// One scale part: its words, whether it contains a hundred, and its kind.
struct Part {
    word: String,
    has_hundred: bool,
    kind: PartKind,
}

/// Words for one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    let mut parts: Vec<Part> = Vec::new();
    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let (segment_word, has_hundred) = build_segment(segment);

        parts.push(if scale_index == 0 {
            // Units segment
            Part {
                word: segment_word,
                has_hundred,
                kind: PartKind::Units,
            }
        } else if scale_index == 1 {
            // Thousands
            Part {
                word: format!("{segment_word} {THOUSAND}"),
                has_hundred: false,
                kind: PartKind::Thousand,
            }
        } else {
            // Millions+
            Part {
                word: format!("{segment_word} {}", SCALES[scale_index - 2]),
                has_hundred: false,
                kind: PartKind::Million,
            }
        });
    }

    // Join parts with Norwegian rules
    join_norwegian_parts(&parts)
}

/// Joins parts with Norwegian spacing and comma rules.
fn join_norwegian_parts(parts: &[Part]) -> String {
    let mut result = String::new();

    for (i, part) in parts.iter().enumerate() {
        result.push_str(&part.word);

        if let Some(next) = parts.get(i + 1) {
            let separator = match part.kind {
                // After thousands: comma if next has hundred, else " og "
                PartKind::Thousand => {
                    if next.has_hundred {
                        ", "
                    } else {
                        " og "
                    }
                }
                // After millions: " og " for units without hundred, space otherwise
                PartKind::Million => {
                    if next.kind == PartKind::Units && !next.has_hundred {
                        " og "
                    } else {
                        " "
                    }
                }
                PartKind::Units => " ",
            };
            result.push_str(separator);
        }
    }

    if result.is_empty() {
        return ZERO.to_string();
    }
    result
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

/// Spells a number in Norwegian Bokmål words.
///
/// ```
/// # use n2words::nb_no::to_cardinal;
/// assert_eq!(to_cardinal(21)?, "tjue-en");
/// assert_eq!(to_cardinal(101)?, "en hundre og en");
/// assert_eq!(to_cardinal(1_000_000)?, "en million");
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
/// Norwegian ordinals: første (1st), andre (2nd), tredje (3rd), etc.
/// 1-12 have special forms, others use cardinal + de/te suffix.
/// Teens (13-19): drop -en and add -ende (tretten → trettende)
/// Numbers ending in 'en' (one): replace with 'ende' (tjue-en → tjue-ende)
/// Numbers ending in 9 (ni): add 'ede' (nitti-ni → nitti-niede)
fn integer_to_ordinal(n: &BigUint) -> String {
    let small = n.to_u64();

    // Special forms for 1-12
    if let Some(small) = small.filter(|small| (1..=12).contains(small)) {
        return ORDINAL_SPECIAL[small as usize].to_string();
    }

    let mut cardinal = integer_to_words(n);

    // Teens 13-19: drop -en and add -ende
    // tretten → trettende, nitten → nittende
    // For other numbers, check if cardinal ends with 'en' (one)
    // tjue-en → tjue-ende, en hundre og en → en hundre og ende
    if small.is_some_and(|small| (13..=19).contains(&small)) || cardinal.ends_with("en") {
        cardinal.truncate(cardinal.len() - 2);
        cardinal.push_str("ende");
        return cardinal;
    }

    // Numbers ending in 'ni' (9): add 'ede'
    // nitti-ni → nitti-niede
    if cardinal.ends_with("ni") {
        cardinal.push_str("ede");
        return cardinal;
    }

    // Others use -de suffix
    cardinal.push_str("de");
    cardinal
}

/// Spells a positive integer as a Norwegian Bokmål ordinal.
///
/// ```
/// # use n2words::nb_no::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "første");
/// assert_eq!(to_ordinal(2)?, "andre");
/// assert_eq!(to_ordinal(21)?, "tjue-ende");
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
        currency: Currency = Currency::NOK, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Norwegian kroner.
///
/// Uses krone/kroner and øre (100 øre = 1 krone).
///
/// ```
/// # use n2words::nb_no::to_currency;
/// assert_eq!(to_currency(1)?, "en krone");
/// assert_eq!(to_currency(42)?, "førti-to kroner");
/// assert_eq!(to_currency(1.5)?, "en krone og femti øre");
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

/// Spells an amount in any currency Norwegian Bokmål has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Norwegian Bokmål has no words for and
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

    // Kroner part
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

/// Registry entry for `nb-NO`.
pub static LANGUAGE: Language = Language {
    code: "nb-NO",
    name: "Norwegian Bokmål (Norway)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
