//! German (Germany) (`de-DE`).
//!
//! - Inverted tens-ones order: "einundzwanzig" (one-and-twenty) for 21-99
//! - Compound words without spaces below million level
//! - Three forms of 1: "eins" (standalone), "ein" (before hundert/tausend), "eine" (before Million+)
//! - Scale pluralization: Million → Millionen, Milliarde → Milliarden
//! - Spaces only around million+ scale words
//!
//! ```
//! use n2words::de_de::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "zweiundvierzig");
//! assert_eq!(to_ordinal(21)?, "einundzwanzigste");
//! assert_eq!(to_currency(42.5)?, "zweiundvierzig Euro und fünfzig Cent");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::DE as CURRENCY_VOCAB;
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

// Ones words (1-9), index 0 unused
const ONES: [&str; 10] = [
    "", "eins", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun",
];

// "ein" form for use before hundert/und
const EIN: &str = "ein";

// Teens (10-19)
const TEENS: [&str; 10] = [
    "zehn",
    "elf",
    "zwölf",
    "dreizehn",
    "vierzehn",
    "fünfzehn",
    "sechzehn",
    "siebzehn",
    "achtzehn",
    "neunzehn",
];

// Tens (20-90)
const TENS: [&str; 10] = [
    "", "", "zwanzig", "dreißig", "vierzig", "fünfzig", "sechzig", "siebzig", "achtzig", "neunzig",
];

// Scale words (index 0 = thousand, 1 = million, etc.)
const SCALES: &[&str] = &[
    "tausend",
    "Million",
    "Milliarde",
    "Billion",
    "Billiarde",
    "Trillion",
    "Trilliarde",
    "Quadrillion",
    "Quadrilliarde",
];

// Pluralized scale words (million+)
const SCALES_PLURAL: &[&str] = &[
    "tausend",
    "Millionen",
    "Milliarden",
    "Billionen",
    "Billiarden",
    "Trillionen",
    "Trilliarden",
    "Quadrillionen",
    "Quadrilliarden",
];

// Supported magnitude ceiling. SCALES is indexed [scale_index - 1] starting at
// 'tausend' (10^3), so segments are [units, then one per SCALES entry] ->
// ceiling 10^((SCALES.len() + 1) * 3) = 10^30. Ordinal (cardinal + suffix) and
// currency build on the cardinal, and the decimal is spelled via
// integer_to_words, so all share the ceiling.

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32);
/// The ordinal ceiling: `10^30`.
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32);

const HUNDRED: &str = "hundert";
const ZERO: &str = "null";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "komma";

// ============================================================================
// Ordinal vocabulary
// ============================================================================

// Ordinal ones (1-9): erste, zweite, dritte, vierte...
// Note: 1st (erste), 3rd (dritte), 7th (siebte), 8th (achte) are irregular
const ORDINAL_ONES: [&str; 10] = [
    "", "erste", "zweite", "dritte", "vierte", "fünfte", "sechste", "siebte", "achte", "neunte",
];

// Ordinal teens: zehnte, elfte, zwölfte...
const ORDINAL_TEENS: [&str; 10] = [
    "zehnte",
    "elfte",
    "zwölfte",
    "dreizehnte",
    "vierzehnte",
    "fünfzehnte",
    "sechzehnte",
    "siebzehnte",
    "achtzehnte",
    "neunzehnte",
];

// Ordinal tens: zwanzigste, dreißigste...
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "zwanzigste",
    "dreißigste",
    "vierzigste",
    "fünfzigste",
    "sechzigste",
    "siebzigste",
    "achtzigste",
    "neunzigste",
];

// Ordinal scale suffix: -ste for 20+
const ORDINAL_SUFFIX: &str = "ste";

// Ordinal stems of the scale words (SCALES lowercased, final -e dropped),
// written inside one word with their multiplier: Million → zweimillionste,
// Milliarde → milliardste
const ORDINAL_SCALE_STEMS: &[&str] = &[
    "tausend",
    "million",
    "milliard",
    "billion",
    "billiard",
    "trillion",
    "trilliard",
    "quadrillion",
    "quadrilliard",
];

// ============================================================================
// Segment building
// ============================================================================

/// Pushes the hundreds of a segment: "ein" before hundert, not "eins".
fn push_hundreds(result: &mut String, hundreds: usize) {
    if hundreds > 0 {
        result.push_str(if hundreds == 1 { EIN } else { ONES[hundreds] });
        result.push_str(HUNDRED);
    }
}

/// Pushes the 1-99 part of a segment in cardinal form. German inverts ones
/// and tens: "einundzwanzig" = one-and-twenty, with "ein" (not "eins") before
/// "und". A lone 1 is spelled `lone_one`.
fn push_tens_ones(result: &mut String, tens: usize, ones: usize, lone_one: &str) {
    if tens == 1 {
        // Teens
        result.push_str(TEENS[ones]);
    } else if tens >= 2 && ones > 0 {
        // Inverted: "einundzwanzig" (one-and-twenty)
        result.push_str(if ones == 1 { EIN } else { ONES[ones] });
        result.push_str("und");
        result.push_str(TENS[tens]);
    } else if tens >= 2 {
        // Just tens
        result.push_str(TENS[tens]);
    } else if ones == 1 {
        result.push_str(lone_one);
    } else if ones > 0 {
        // Just ones (no tens, possibly after hundreds)
        result.push_str(ONES[ones]);
    }
}

/// Builds the words for a 0-999 segment (standalone form, uses "eins").
fn build_segment(n: u32) -> String {
    let mut result = String::new();
    let n = n as usize;
    push_hundreds(&mut result, n / 100);
    push_tens_ones(&mut result, (n / 10) % 10, n % 10, ONES[1]);
    result
}

/// Builds the words for a 0-999 segment used before "tausend": "ein" instead
/// of "eins" for 1 ("eintausend", "zweihunderteintausend", not
/// "zweihunderteinstausend").
fn build_segment_for_thousand(n: u32) -> String {
    let mut result = String::new();
    let n = n as usize;
    push_hundreds(&mut result, n / 100);
    push_tens_ones(&mut result, (n / 10) % 10, n % 10, EIN);
    result
}

// ============================================================================
// Cardinal
// ============================================================================

/// A word group; million+ scale words are spaced apart from their neighbours.
struct Part {
    words: String,
    is_scale: bool,
}

/// Joins parts with German spacing rules: spaces only around million+ scale
/// words.
fn join_german_parts(parts: &[Part]) -> String {
    if parts.is_empty() {
        return ZERO.to_string();
    }
    let mut result = String::new();
    for (i, part) in parts.iter().enumerate() {
        // Space before a million+ scale word, and after one
        if i > 0 && (part.is_scale || parts[i - 1].is_scale) {
            result.push(' ');
        }
        result.push_str(&part.words);
    }
    result
}

/// Pushes a million+ segment and its scale word: "eine Million",
/// "zwei Millionen".
fn push_large_scale(parts: &mut Vec<Part>, segment: u32, scale_index: usize) {
    let (words, scale_word) = if segment == 1 {
        ("eine".to_string(), SCALES[scale_index - 1])
    } else {
        (build_segment(segment), SCALES_PLURAL[scale_index - 1])
    };
    parts.push(Part {
        words,
        is_scale: false,
    });
    parts.push(Part {
        words: scale_word.to_string(),
        is_scale: true,
    });
}

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    if let Some(small) = n.to_u64() {
        // Fast path: numbers < 1000
        if small < 1000 {
            return build_segment(small as u32);
        }
        // Fast path: numbers < 1,000,000
        if small < 1_000_000 {
            // Compound: "eintausendzweihundert" (no spaces)
            let mut result = build_segment_for_thousand((small / 1000) as u32);
            result.push_str(SCALES[0]);
            result.push_str(&build_segment((small % 1000) as u32));
            return result;
        }
    }

    // For numbers >= 1,000,000, use scale decomposition
    build_large_number_words(n)
}

/// Words for a number >= 1,000,000.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let mut parts: Vec<Part> = Vec::new();

    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        match scale_index {
            // Units segment (no scale word)
            0 => parts.push(Part {
                words: build_segment(segment),
                is_scale: false,
            }),
            // Thousands: compound without space
            1 => parts.push(Part {
                words: build_segment_for_thousand(segment) + SCALES[0],
                is_scale: false,
            }),
            // Million+: space around scale word
            _ => push_large_scale(&mut parts, segment, scale_index),
        }
    }

    join_german_parts(&parts)
}

/// Words for the fraction digits: each leading zero is "null", the rest is
/// read as one number.
fn decimal_part_to_words(fraction: &str) -> String {
    let mut result = String::new();

    // Handle leading zeros
    let digits = fraction.trim_start_matches('0');
    for _ in 0..fraction.len() - digits.len() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(ZERO);
    }

    // Convert remainder as a single number
    if !digits.is_empty() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&digits_to_biguint(digits)));
    }

    result
}

/// Spells a number in German words.
///
/// ```
/// # use n2words::de_de::to_cardinal;
/// assert_eq!(to_cardinal(21)?, "einundzwanzig");
/// assert_eq!(to_cardinal(1000)?, "eintausend");
/// assert_eq!(to_cardinal(1000000)?, "eine Million");
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
    // Both the integer part and the fraction's significant digits are spelled
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

/// Builds the final (ordinal) segment for 1-999: the hundreds stay cardinal,
/// the last component takes -te for 1-19 and -ste for 20+.
fn build_ordinal_segment(n: u32) -> String {
    let mut result = String::new();
    let n = n as usize;
    let ones = n % 10;
    let tens = (n / 10) % 10;

    // Hundreds part (cardinal form)
    push_hundreds(&mut result, n / 100);

    if tens == 1 {
        // Teens: 10-19
        result.push_str(ORDINAL_TEENS[ones]);
    } else if tens >= 2 && ones > 0 {
        // Compound: einundzwanzig → einundzwanzigste
        result.push_str(if ones == 1 { EIN } else { ONES[ones] });
        result.push_str("und");
        result.push_str(ORDINAL_TENS[tens]);
    } else if tens >= 2 {
        // Just tens: zwanzig → zwanzigste
        result.push_str(ORDINAL_TENS[tens]);
    } else if ones > 0 {
        // Just ones: eins → erste
        result.push_str(ORDINAL_ONES[ones]);
    } else if n > 0 {
        // Only hundreds (100, 200, etc.): add the ordinal suffix
        result.push_str(ORDINAL_SUFFIX);
    }

    result
}

/// Ordinal words for a positive integer.
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        // Fast path: numbers < 1000
        if small < 1000 {
            return build_ordinal_segment(small as u32);
        }
        // Fast path: numbers < 1,000,000
        if small < 1_000_000 {
            let remainder = (small % 1000) as u32;
            let mut result = build_segment_for_thousand((small / 1000) as u32);
            result.push_str(SCALES[0]);
            if remainder == 0 {
                // Exact thousands: "eintausendste", "zweitausendste"
                result.push_str(ORDINAL_SUFFIX);
            } else {
                // Has remainder: cardinal thousands + ordinal remainder
                result.push_str(&build_ordinal_segment(remainder));
            }
            return result;
        }
    }

    // For larger numbers, use scale decomposition
    build_large_ordinal(n)
}

/// Ordinal words for a number >= 1,000,000.
fn build_large_ordinal(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    // The lowest non-zero segment carries the ordinal ending
    let lowest_non_zero = segments.iter().position(|&s| s != 0).unwrap_or(0);

    let mut parts: Vec<Part> = Vec::new();

    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let is_lowest = scale_index == lowest_non_zero;
        if scale_index == 0 {
            // Units segment
            parts.push(Part {
                words: build_ordinal_segment(segment),
                is_scale: false,
            });
        } else if scale_index == 1 {
            // Thousands
            let mut words = build_segment_for_thousand(segment) + SCALES[0];
            if is_lowest {
                words.push_str(ORDINAL_SUFFIX);
            }
            parts.push(Part {
                words,
                is_scale: false,
            });
        } else if is_lowest {
            // Million+ ordinal: one lowercase word with its multiplier, which
            // drops "ein" only when the scale word stands alone: "millionste",
            // "zweimillionste", "eine Milliarde einmillionste"
            let mut words = if segment == 1 && parts.is_empty() {
                String::new()
            } else {
                build_segment_for_thousand(segment)
            };
            words.push_str(ORDINAL_SCALE_STEMS[scale_index - 1]);
            words.push_str(ORDINAL_SUFFIX);
            parts.push(Part {
                words,
                is_scale: false,
            });
        } else {
            // Million+
            push_large_scale(&mut parts, segment, scale_index);
        }
    }

    join_german_parts(&parts)
}

/// Spells a positive integer as a German ordinal.
///
/// German ordinals add -te for 1-19 and -ste for 20+.
/// Irregular forms: erste (1st), dritte (3rd), siebte (7th), achte (8th).
///
/// ```
/// # use n2words::de_de::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "erste");
/// assert_eq!(to_ordinal(3)?, "dritte");
/// assert_eq!(to_ordinal(100)?, "einhundertste");
/// assert_eq!(to_ordinal(1000000)?, "millionste");
/// assert_eq!(to_ordinal(2000000)?, "zweimillionste");
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
        /// Use "und" between euros and cents
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::EUR, table = CURRENCY_VOCAB;
    }
}

/// Spells a count placed before a noun: a final "eins" becomes "ein"
/// ("ein Euro", "einhundertein Euro", "eine Million ein Euro").
fn count_before_noun(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }
    if let Some(small) = n.to_u64() {
        // Below a million the form used before "tausend" is the one used
        // before a noun: "ein", "einhundertein", "zweihunderteintausendein"
        if small < 1000 {
            return build_segment_for_thousand(small as u32);
        }
        if small < 1_000_000 {
            let mut result = build_segment_for_thousand((small / 1000) as u32);
            result.push_str(SCALES[0]);
            result.push_str(&build_segment_for_thousand((small % 1000) as u32));
            return result;
        }
    }
    let mut words = integer_to_words(n);
    // Only a number ending in 01 ends in "eins" (21 is "einundzwanzig")
    if (n % 100u32).is_one() {
        words.pop();
    }
    words
}

/// Spells an amount in euros.
///
/// ```
/// # use n2words::de_de::to_currency;
/// assert_eq!(to_currency(1)?, "ein Euro");
/// assert_eq!(to_currency(101)?, "einhundertein Euro");
/// assert_eq!(to_currency(0.99)?, "neunundneunzig Cent");
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

/// Spells an amount in any currency German has words for.
///
/// ```
/// # use n2words::de_de::{to_currency_with, CurrencyOptions};
/// let opts = CurrencyOptions { and: false, ..Default::default() };
/// assert_eq!(to_currency_with(42.5, &opts)?, "zweiundvierzig Euro fünfzig Cent");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency German has no words for and
/// [`Error::NoMinorUnit`](crate::Error::NoMinorUnit) for a fractional amount
/// in a currency without one.
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    let words = CURRENCY_VOCAB.words(options.currency)?;
    let money = parse_currency(&value.into(), options.currency.minor_unit_digits())?;
    CURRENCY_MAX.check(&money.major, None)?; // minor units are <= 999, safe
    assert_minor_unit(money.minor, options.currency)?;

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }

    // Euros part
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&count_before_noun(&money.major));
        result.push(' ');
        result.push_str(words.major_at(0));
    }

    // Cents part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(if options.and { " und " } else { " " });
        }
        result.push_str(&count_before_noun(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(0));
    }

    Ok(result)
}

/// Registry entry for `de-DE`.
pub static LANGUAGE: Language = Language {
    code: "de-DE",
    name: "German (Germany)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
