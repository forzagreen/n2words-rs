//! Dutch (Netherlands) (`nl-NL`).
//!
//! Dutch as used in the Netherlands.
//!
//! Dutch-specific rules:
//!
//! - Inverted tens-ones: eenentwintig (one-and-twenty)
//! - "ën" connector when ones ends in 'e' (twee, drie)
//! - Compound words without spaces
//! - Hundred pairing for 1100-9999 (elfhonderd style)
//! - "één" vs "een" (`accent_one` option)
//! - Optional "en" separator (`include_optional_and` option)
//! - Long scale with -ard forms
//!
//! ```
//! use n2words::nl_nl::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "eenentwintig");
//! assert_eq!(to_cardinal(1104)?, "elfhonderd vier");
//! assert_eq!(to_ordinal(21)?, "eenentwintigste");
//! assert_eq!(to_currency(42.5)?, "tweeënveertig euro en vijftig cent");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::NL as CURRENCY_VOCAB;
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

const ONES: [&str; 10] = [
    "", "een", "twee", "drie", "vier", "vijf", "zes", "zeven", "acht", "negen",
];
const TEENS: [&str; 10] = [
    "tien",
    "elf",
    "twaalf",
    "dertien",
    "veertien",
    "vijftien",
    "zestien",
    "zeventien",
    "achttien",
    "negentien",
];
const TENS: [&str; 10] = [
    "", "", "twintig", "dertig", "veertig", "vijftig", "zestig", "zeventig", "tachtig", "negentig",
];

const HUNDRED: &str = "honderd";

// Scale words (long scale with -ard forms)
const SCALES: &[&str] = &[
    "duizend",
    "miljoen",
    "miljard",
    "biljoen",
    "biljard",
    "triljoen",
    "triljard",
    "quadriljoen",
    "quadriljard",
];

// Scale ordinal words. Its length derives the ordinal ceiling.
const SCALE_ORDINAL: &[&str] = &[
    "",
    "duizendste",
    "miljoenste",
    "miljardste",
    "biljoenste",
    "biljardste",
    "triljoenste",
];

/// The cardinal ceiling, derived from the scale table: `10^30`.
///
/// `SCALES` is indexed `[scale_index - 1]` from "duizend" (10^3), so
/// cardinals reach `10^((SCALES.len() + 1) * 3)`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32);
/// The ordinal ceiling: `10^21`.
///
/// The ordinal of a number whose lowest non-zero group is a scale group uses
/// the shorter `SCALE_ORDINAL` table, so ordinals are bounded at
/// `10^(SCALE_ORDINAL.len() * 3)`.
pub const ORDINAL_MAX: Max = Max::western(SCALE_ORDINAL.len() as u32 - 1);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32);

const ZERO: &str = "nul";
const NEGATIVE: &str = "min";
const DECIMAL_SEP: &str = "komma";

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment (compound, no spaces).
///
/// `with_and` includes "en" for values < 13 after hundreds.
fn build_segment(n: u32, with_and: bool) -> String {
    if n == 0 {
        return String::new();
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;
    let tens_ones = (n % 100) as usize;

    let mut result = String::new();

    // Hundreds
    if hundreds > 0 {
        if hundreds > 1 {
            result.push_str(ONES[hundreds]);
        }
        result.push_str(HUNDRED);
    }

    // Tens and ones
    if tens_ones == 0 {
        // Just hundreds
    } else if tens_ones < 10 {
        // Single digit - add "en" if with_and and after hundreds
        if hundreds > 0 && with_and {
            result.push_str("en");
        }
        result.push_str(ONES[tens_ones]);
    } else if tens_ones < 20 {
        // Teens - add "en" if with_and and after hundreds and < 13
        if hundreds > 0 && with_and && tens_ones < 13 {
            result.push_str("en");
        }
        result.push_str(TEENS[ones]);
    } else if ones == 0 {
        result.push_str(TENS[tens]);
    } else {
        // 20-99: Dutch inverts with connector, "ën" if ones ends in 'e'
        // (twee, drie)
        let ones_word = ONES[ones];
        result.push_str(ones_word);
        result.push_str(if ones_word.ends_with('e') {
            "ën"
        } else {
            "en"
        });
        result.push_str(TENS[tens]);
    }

    result
}

// ============================================================================
// Cardinal
// ============================================================================

/// The resolved conversion options shared by cardinal, decimal and currency.
#[derive(Clone, Copy)]
struct Opts {
    accent_one: bool,
    include_optional_and: bool,
    no_hundred_pairing: bool,
}

/// Replaces every standalone word "een" with "één" (JS `/\been\b/g`).
///
/// Word boundaries follow JS's non-Unicode `\b`: only ASCII letters, digits
/// and `_` are word characters.
fn apply_accent(word: String, accent_one: bool) -> String {
    if !accent_one || !word.contains("een") {
        return word;
    }
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let bytes = word.as_bytes();
    let mut out = String::with_capacity(word.len() + 4);
    let mut last = 0;
    let mut i = 0;
    while i + 3 <= bytes.len() {
        if &bytes[i..i + 3] == b"een"
            && (i == 0 || !is_word(bytes[i - 1]))
            && (i + 3 == bytes.len() || !is_word(bytes[i + 3]))
        {
            out.push_str(&word[last..i]);
            out.push_str("één");
            i += 3;
            last = i;
        } else {
            i += 1;
        }
    }
    out.push_str(&word[last..]);
    out
}

/// Appends `word` after a scale, with " en " before small values when the
/// optional "en" is on.
fn push_after_scale(result: &mut String, word: &str, value: u32, include_optional_and: bool) {
    if include_optional_and && value < 13 {
        result.push_str(" en ");
    } else {
        result.push(' ');
    }
    result.push_str(word);
}

/// Converts a non-negative integer to Dutch words.
fn integer_to_words(n: &BigUint, options: Opts) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    let with_and = options.include_optional_and;

    if let Some(small) = n.to_u64() {
        // Fast path: numbers < 1000
        if small < 1000 {
            return apply_accent(build_segment(small as u32, with_and), options.accent_one);
        }

        // Hundred pairing for 1100-9999
        if !options.no_hundred_pairing && (1100..10000).contains(&small) {
            let high = (small / 100) as u32;
            let low = (small % 100) as u32;

            // Only use pairing when high is not a multiple of 10
            if high % 10 != 0 {
                let mut result = build_segment(high, with_and);
                result.push_str(HUNDRED);
                if low > 0 {
                    push_after_scale(&mut result, &build_segment(low, with_and), low, with_and);
                }
                return apply_accent(result, options.accent_one);
            }
        }

        // Fast path: numbers < 1,000,000 (thousands)
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            // "duizend" not "eenduizend"; otherwise compound: "vijfduizend"
            let mut result = if thousands == 1 {
                String::new()
            } else {
                build_segment(thousands, with_and)
            };
            result.push_str(SCALES[0]);

            if remainder > 0 {
                push_after_scale(
                    &mut result,
                    &build_segment(remainder, with_and),
                    remainder,
                    with_and,
                );
            }

            return apply_accent(result, options.accent_one);
        }
    }

    // For numbers >= 1,000,000, use scale decomposition
    apply_accent(build_large_number_words(n, with_and), options.accent_one)
}

/// Words for numbers >= 1,000,000.
fn build_large_number_words(n: &BigUint, include_optional_and: bool) -> String {
    let segments = split_groups(n, 1000);

    let mut result = String::new();
    let mut prev_was_scale = false;

    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }

        if i == 0 {
            // Units segment
            let word = build_segment(segment, include_optional_and);
            if result.is_empty() {
                result = word;
            } else {
                push_after_scale(
                    &mut result,
                    &word,
                    segment,
                    prev_was_scale && include_optional_and,
                );
            }
            prev_was_scale = false;
        } else if i == 1 {
            // Thousands - compound
            if !result.is_empty() {
                result.push(' ');
            }
            if segment != 1 {
                result.push_str(&build_segment(segment, include_optional_and));
            }
            result.push_str(SCALES[0]);
            prev_was_scale = true;
        } else {
            // Million and above - space around scale
            if !result.is_empty() {
                result.push(' ');
            }
            if segment == 1 {
                result.push_str("een");
            } else {
                result.push_str(&build_segment(segment, include_optional_and));
            }
            result.push(' ');
            result.push_str(SCALES[i - 1]);
            prev_was_scale = true;
        }
    }

    result
}

/// Converts decimal digits to Dutch words: each leading zero as "nul", then
/// the rest as one number (without hundred pairing).
fn decimal_part_to_words(fraction: &str, options: Opts) -> String {
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
        result.push_str(&integer_to_words(
            &digits_to_biguint(digits),
            Opts {
                no_hundred_pairing: true,
                ..options
            },
        ));
    }

    result
}

options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Use "één" instead of "een"
        accent_one: bool = true;
        /// Include "en" before small numbers
        include_optional_and: bool = false;
        /// Disable hundred pairing (1104→duizend honderdvier)
        no_hundred_pairing: bool = false;
    }
}

/// Spells a number in Dutch words.
///
/// ```
/// # use n2words::nl_nl::to_cardinal;
/// assert_eq!(to_cardinal(1)?, "één");
/// assert_eq!(to_cardinal(1104)?, "elfhonderd vier");
/// assert_eq!(to_cardinal(17.42)?, "zeventien komma tweeënveertig");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value, [`Error::OutOfRange`](crate::Error::OutOfRange) at or past
/// [`CARDINAL_MAX`].
pub fn to_cardinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_cardinal_with(value, &CardinalOptions::default())
}

/// Spells a number in Dutch words, with options.
///
/// ```
/// # use n2words::nl_nl::{to_cardinal_with, CardinalOptions};
/// let plain = CardinalOptions { accent_one: false, ..CardinalOptions::default() };
/// assert_eq!(to_cardinal_with(1, &plain)?, "een");
/// let unpaired = CardinalOptions { no_hundred_pairing: true, ..CardinalOptions::default() };
/// assert_eq!(to_cardinal_with(1104, &unpaired)?, "duizend honderdvier");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_cardinal`].
pub fn to_cardinal_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CardinalOptions,
) -> Result<String> {
    let number = parse_cardinal(&value.into())?;
    // Both the integer part and the decimal's significant digits are spelled
    // via the scale builder, so both must clear the ceiling.
    CARDINAL_MAX.check(&number.integer, number.fraction.as_deref())?;

    let opts = Opts {
        accent_one: options.accent_one,
        include_optional_and: options.include_optional_and,
        no_hundred_pairing: options.no_hundred_pairing,
    };

    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }

    result.push_str(&integer_to_words(&number.integer, opts));

    if let Some(fraction) = &number.fraction {
        result.push(' ');
        result.push_str(DECIMAL_SEP);
        result.push(' ');
        result.push_str(&decimal_part_to_words(fraction, opts));
    }

    Ok(result)
}

// ============================================================================
// Ordinal
// ============================================================================

// Ordinal forms for 1-9
const ORDINAL_ONES: [&str; 10] = [
    "", "eerste", "tweede", "derde", "vierde", "vijfde", "zesde", "zevende", "achtste", "negende",
];

/// Converts a small cardinal (0-99) to ordinal.
///
/// Rules: 1-19 add -de (except eerste, derde, achtste), 20+ add -ste.
fn small_cardinal_to_ordinal(cardinal_word: String, n: u32) -> String {
    // Special cases for 1-9
    if (1..=9).contains(&n) {
        return ORDINAL_ONES[n as usize].to_string();
    }

    let mut result = cardinal_word;
    if n < 20 {
        // 10-19: add -de, or -e if it ends in d
        result.push_str(if result.ends_with('d') { "e" } else { "de" });
    } else {
        // 20+: add -ste
        result.push_str("ste");
    }
    result
}

/// Ordinal words for 0-999.
fn build_ordinal_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let hundreds = (n / 100) as usize;
    let tens_ones = n % 100;

    // Simple cases: 1-99
    if hundreds == 0 {
        return small_cardinal_to_ordinal(build_segment(n, false), n);
    }

    // Hundreds: need to build prefix + ordinal suffix
    let mut result = String::new();
    if hundreds > 1 {
        result.push_str(ONES[hundreds]);
    }
    result.push_str(HUNDRED);

    if tens_ones == 0 {
        // Exact hundred: honderdste, tweehonderdste
        result.push_str("ste");
        return result;
    }

    // Hundreds + remainder: honderd eerste, tweehonderd drieëntwintigste
    // Dutch ordinals with hundreds use a space before the ordinal part
    result.push(' ');
    result.push_str(&small_cardinal_to_ordinal(
        build_segment(tens_ones, false),
        tens_ones,
    ));
    result
}

/// Ordinal words for numbers >= 1000: cardinal words with the lowest non-zero
/// segment in ordinal form.
fn build_large_ordinal(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    // Find the lowest non-zero segment
    let lowest_non_zero = segments.iter().position(|&s| s != 0).unwrap_or(0);

    let mut result = String::new();

    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }

        if !result.is_empty() {
            result.push(' ');
        }

        if i == lowest_non_zero {
            // Last non-zero segment gets ordinal form
            if i == 0 {
                // Units segment: use ordinal segment builder
                result.push_str(&build_ordinal_segment(segment));
            } else if segment == 1 {
                // Exact scale: duizendste, een miljoenste, etc.
                if i > 1 {
                    result.push_str("een ");
                }
                result.push_str(SCALE_ORDINAL[i]);
            } else {
                // Segment + scale ordinal
                result.push_str(&build_segment(segment, false));
                if i > 1 {
                    result.push(' ');
                }
                result.push_str(SCALE_ORDINAL[i]);
            }
        } else if i == 1 {
            // Higher segments use cardinal form
            if segment != 1 {
                result.push_str(&build_segment(segment, false));
            }
            result.push_str(SCALES[0]);
        } else {
            if segment == 1 {
                result.push_str("een");
            } else {
                result.push_str(&build_segment(segment, false));
            }
            result.push(' ');
            result.push_str(SCALES[i - 1]);
        }
    }

    result
}

/// Spells a positive integer as a Dutch ordinal.
///
/// ```
/// # use n2words::nl_nl::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "eerste");
/// assert_eq!(to_ordinal(21)?, "eenentwintigste");
/// assert_eq!(to_ordinal(100)?, "honderdste");
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

    if let Some(small) = n.to_u64() {
        // Fast path: 1-9
        if small < 10 {
            return Ok(ORDINAL_ONES[small as usize].to_string());
        }

        // Fast path: 10-99
        if small < 100 {
            let small = small as u32;
            return Ok(small_cardinal_to_ordinal(
                build_segment(small, false),
                small,
            ));
        }

        // Fast path: 100-999
        if small < 1000 {
            return Ok(build_ordinal_segment(small as u32));
        }
    }

    // Large numbers: build with cardinal then convert last segment
    Ok(build_large_ordinal(&n))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// Include "en" between euros and cents
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::EUR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in euros.
///
/// ```
/// # use n2words::nl_nl::to_currency;
/// assert_eq!(to_currency(42.5)?, "tweeënveertig euro en vijftig cent");
/// assert_eq!(to_currency(1)?, "één euro");
/// assert_eq!(to_currency(0.01)?, "één cent");
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

/// Spells an amount in any currency Dutch has words for, with options.
///
/// ```
/// # use n2words::nl_nl::{to_currency_with, CurrencyOptions};
/// let opts = CurrencyOptions { and: false, ..CurrencyOptions::default() };
/// assert_eq!(to_currency_with(42.5, &opts)?, "tweeënveertig euro vijftig cent");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Dutch has no words for and
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

    let has_major = !money.major.is_zero();
    let has_minor = money.minor > 0;

    if !has_major && !has_minor {
        return Ok(format!("{ZERO} {}", words.major_at(0)));
    }

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }

    // Use accent_one for currency (één euro, één cent)
    let opts = Opts {
        accent_one: true,
        include_optional_and: false,
        no_hundred_pairing: true,
    };

    if has_major {
        result.push_str(&integer_to_words(&money.major, opts));
        result.push(' ');
        result.push_str(words.major_at(0));
    }

    if has_minor {
        if has_major {
            result.push_str(if options.and { " en " } else { " " });
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor), opts));
        result.push(' ');
        result.push_str(words.minor_at(0));
    }

    Ok(result)
}

/// Registry entry for `nl-NL`.
pub static LANGUAGE: Language = Language {
    code: "nl-NL",
    name: "Dutch (Netherlands)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
