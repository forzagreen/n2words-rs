//! Swedish (Sweden) (`sv-SE`): Swedish as used in Sweden.
//!
//! - Hyphenation for tens-ones: "tjugo-ett" (21)
//! - "och" after hundreds before small numbers: "hundra och ett" (101)
//! - "ett" omitted before hundra and tusen: "hundra" (100), "tusen" (1000)
//! - "en" (not "ett") before million+ scales: "en miljon"
//! - Long scale naming with -ard forms (miljon, miljard, biljon, ...)
//!
//! ```
//! use n2words::sv_se::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "fyrtio-två");
//! assert_eq!(to_ordinal(42)?, "fyrtio-tvåde");
//! assert_eq!(to_currency(42.5)?, "fyrtio-två kronor och femtio öre");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::SV as CURRENCY_VOCAB;
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
    "", "ett", "två", "tre", "fyra", "fem", "sex", "sju", "åtta", "nio",
];

const TEENS: [&str; 10] = [
    "tio", "elva", "tolv", "tretton", "fjorton", "femton", "sexton", "sjutton", "arton", "nitton",
];
const TENS: [&str; 10] = [
    "", "", "tjugo", "trettio", "fyrtio", "femtio", "sextio", "sjuttio", "åttio", "nittio",
];

const HUNDRED: &str = "hundra";

const ZERO: &str = "noll";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "komma";

/// Scale words (long scale with -ard forms), indexed from "tusen" (10^3).
const SCALES: &[&str] = &[
    "tusen",
    "miljon",
    "miljard",
    "biljon",
    "biljard",
    "triljon",
    "triljard",
    "kvadriljon",
];

// Supported magnitude ceiling. SCALES is indexed [scale_index - 1] from
// "tusen" (10^3), so segments are [units, then one per SCALES entry] ->
// ceiling 10^((SCALES.len() + 1) * 3) = 10^27. Ordinal (cardinal + suffix)
// and currency build on the cardinal, and the decimal is spelled via
// integer_to_words, so all share it.

/// The cardinal ceiling, derived from the scale table: `10^27`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32);
/// The ordinal ceiling: `10^27`.
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32);
/// The currency ceiling: `10^27`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32);

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

// Swedish ordinals: 1st-2nd use -a (första, andra), then the -de suffix.
// 1-12 have special forms; index 0 is unused.
const ORDINAL_ONES: [&str; 13] = [
    "", "första", "andra", "tredje", "fjärde", "femte", "sjätte", "sjunde", "åttonde", "nionde",
    "tionde", "elfte", "tolfte",
];

// ============================================================================
// Segment building
// ============================================================================

/// A 0-999 segment's words, with the metadata the "och" logic needs.
struct Segment {
    word: String,
    has_hundred: bool,
    less_than_100: bool,
}

/// Builds the words for a 0-999 segment.
fn build_segment(n: u32) -> Segment {
    if n == 0 {
        return Segment {
            word: String::new(),
            has_hundred: false,
            less_than_100: false,
        };
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    // Hundreds - omit "ett" before hundra
    let hundred_word = match hundreds {
        0 => None,
        1 => Some(HUNDRED.to_string()),
        h => Some(format!("{} {HUNDRED}", ONES[h])),
    };

    // Tens and ones with hyphenation
    let tens_ones_word = if tens == 1 {
        TEENS[ones].to_string()
    } else if tens >= 2 {
        if ones > 0 {
            format!("{}-{}", TENS[tens], ONES[ones])
        } else {
            TENS[tens].to_string()
        }
    } else {
        ONES[ones].to_string()
    };

    // Combine with "och" after hundreds if there's a remainder
    match hundred_word {
        Some(h) if !tens_ones_word.is_empty() => Segment {
            word: format!("{h} och {tens_ones_word}"),
            has_hundred: true,
            less_than_100: false,
        },
        Some(h) => Segment {
            word: h,
            has_hundred: true,
            less_than_100: false,
        },
        None => Segment {
            word: tens_ones_word,
            has_hundred: false,
            less_than_100: true,
        },
    }
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
            return build_segment(small as u32).word;
        }

        // Fast path: numbers < 1,000,000 (thousands)
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            // Omit "ett" before tusen
            let mut result = if thousands == 1 {
                SCALES[0].to_string()
            } else {
                format!("{} {}", build_segment(thousands).word, SCALES[0])
            };

            if remainder > 0 {
                let remainder_result = build_segment(remainder);
                // Insert "och" if remainder < 100 (doesn't have hundred)
                if remainder_result.less_than_100 {
                    result.push_str(" och ");
                } else {
                    result.push(' ');
                }
                result.push_str(&remainder_result.word);
            }

            return result;
        }
    }

    // For numbers >= 1,000,000, use scale decomposition
    build_large_number_words(n)
}

/// A word of a large number, with the metadata the "och" logic needs.
struct Part {
    word: String,
    has_hundred: bool,
    is_scale: bool,
}

/// Words for a number of 1,000,000 or more.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    let mut parts: Vec<Part> = Vec::new();
    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }

        if scale_index == 0 {
            // Units segment
            let segment_result = build_segment(segment);
            parts.push(Part {
                word: segment_result.word,
                has_hundred: segment_result.has_hundred,
                is_scale: false,
            });
        } else {
            // Segment with scale word
            let scale_word = SCALES[scale_index - 1];

            let segment_word = if segment == 1 {
                // Omit "ett" before tusen, use "en" before million+
                if scale_index == 1 {
                    String::new() // Just "tusen"
                } else {
                    "en".to_string() // "en miljon"
                }
            } else {
                build_segment(segment).word
            };

            if !segment_word.is_empty() {
                parts.push(Part {
                    word: segment_word,
                    has_hundred: false,
                    is_scale: false,
                });
            }
            parts.push(Part {
                word: scale_word.to_string(),
                has_hundred: false,
                is_scale: true,
            });
        }
    }

    // Join with Swedish "och" rules
    join_swedish_parts(&parts)
}

/// Joins parts with Swedish "och" rules: "och" goes before the final segment
/// if it follows a scale word and doesn't have "hundra".
fn join_swedish_parts(parts: &[Part]) -> String {
    match parts {
        [] => return ZERO.to_string(),
        [only] => return only.word.clone(),
        _ => {}
    }

    let mut result: Vec<&str> = Vec::with_capacity(parts.len() + 1);
    let last = parts.len() - 1;
    for (i, part) in parts.iter().enumerate() {
        if i == last {
            let prev_part = &parts[i - 1];
            // Insert "och" if previous was scale and this doesn't have hundred
            if prev_part.is_scale && !part.has_hundred {
                result.push("och");
            }
        }
        result.push(&part.word);
    }

    result.join(" ")
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// the rest as a single number.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut result = String::new();

    // Handle leading zeros
    let significant = decimal.trim_start_matches('0');
    for _ in 0..decimal.len() - significant.len() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(ZERO);
    }

    // Convert remainder as a single number
    if !significant.is_empty() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&digits_to_biguint(significant)));
    }

    result
}

/// Spells a number in Swedish words.
///
/// ```
/// # use n2words::sv_se::to_cardinal;
/// assert_eq!(to_cardinal(42)?, "fyrtio-två");
/// assert_eq!(to_cardinal(101)?, "hundra och ett");
/// assert_eq!(to_cardinal(1_000_000)?, "en miljon");
/// assert_eq!(to_cardinal("3.05")?, "tre komma noll fem");
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

/// Ordinal words for a positive integer: första (1st), andra (2nd),
/// tredje (3rd), ...
///
/// Special forms for 1-12; past that, the cardinal plus the -de suffix.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special forms for 1-12
    if let Some(small) = n.to_u64() {
        if (1..=12).contains(&small) {
            return ORDINAL_ONES[small as usize].to_string();
        }
    }

    // For numbers > 12, add the -de suffix to the cardinal. Compounds ending
    // in "ett" keep it (no första pattern): Swedish ordinals mostly add -de.
    let mut result = integer_to_words(n);
    result.push_str("de");
    result
}

/// Spells a positive integer as a Swedish ordinal.
///
/// ```
/// # use n2words::sv_se::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "första");
/// assert_eq!(to_ordinal(2)?, "andra");
/// assert_eq!(to_ordinal(21)?, "tjugo-ettde");
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
        currency: Currency = Currency::SEK, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Swedish kronor (krona/kronor and öre,
/// 100 öre = 1 krona).
///
/// ```
/// # use n2words::sv_se::to_currency;
/// assert_eq!(to_currency(1)?, "en krona");
/// assert_eq!(to_currency(42)?, "fyrtio-två kronor");
/// assert_eq!(to_currency(1.5)?, "en krona och femtio öre");
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

/// Spells an amount in any currency Swedish has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Swedish has no words for and
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

    // Kronor part
    if !money.major.is_zero() || money.minor == 0 {
        if money.major.is_one() {
            // Use "en" for 1 krona (not "ett")
            result.push_str("en ");
            result.push_str(words.major_at(0));
        } else {
            result.push_str(&integer_to_words(&money.major));
            result.push(' ');
            result.push_str(words.major_at(1));
        }
    }

    // Öre part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(" och ");
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(0));
    }

    Ok(result)
}

/// Registry entry for `sv-SE`.
pub static LANGUAGE: Language = Language {
    code: "sv-SE",
    name: "Swedish (Sweden)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
