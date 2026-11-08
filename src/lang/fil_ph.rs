//! Filipino (Philippines) (`fil-PH`).
//!
//! - Linker "ng" after vowels: "isang daang" (100)
//! - Linker " na " after consonants: "siyam na daang" (900)
//! - Special tens with linker: "limampung anim" (56)
//! - Per-digit decimal reading
//!
//! ```
//! use n2words::fil_ph::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(256)?, "dalawang daang limampung anim");
//! assert_eq!(to_ordinal(42)?, "ika-apatnapu dalawa");
//! assert_eq!(to_currency(42.5)?, "apatnapu dalawang piso at limampung sentimo");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::FIL as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, parse_cardinal, parse_currency, parse_ordinal, split_groups};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

const ONES: [&str; 10] = [
    "", "isa", "dalawa", "tatlo", "apat", "lima", "anim", "pito", "walo", "siyam",
];
const TEENS: [&str; 10] = [
    "sampu",
    "labinisa",
    "labindalawa",
    "labintatlo",
    "labinapat",
    "labinlima",
    "labinanum",
    "labimpito",
    "labingwalo",
    "labinsiyam",
];
const TENS: [&str; 10] = [
    "",
    "",
    "dalawampu",
    "tatlumpu",
    "apatnapu",
    "limampu",
    "animnapu",
    "pitumpu",
    "walumpu",
    "siyamnapu",
];

// Scale words include linker (end with "ng")
const HUNDRED: &str = "daang";
const THOUSAND: &str = "libong";

const ZERO: &str = "zero";
const NEGATIVE: &str = "negatibo";
const DECIMAL_SEP: &str = "punto";

// Short scale with linker; index 0 is the (empty) units slot.
const SCALE_WORDS: &[&str] = &["", THOUSAND, "milyong", "bilyong", "trilyong"];

/// The cardinal ceiling, derived from the scale table: `10^15`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The ordinal ceiling: `10^15`.
pub const ORDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The currency ceiling: `10^15`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

// Filipino ordinals use "ika-" prefix: ika-tatlo (3rd), ika-apat (4th), etc.
// Special forms for 1st and 2nd.
const ORDINAL_PREFIX: &str = "ika";
const ORDINAL_FIRST: &str = "una";
// second (contracted form)
const ORDINAL_SECOND: &str = "ikalawa";

// ============================================================================
// Helper Functions
// ============================================================================

const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];

/// Appends the linker: "ng" after a vowel, " na" after a consonant.
fn add_linker(word: &str) -> String {
    if word.ends_with(VOWELS) {
        format!("{word}ng")
    } else {
        format!("{word} na")
    }
}

/// Adds the linker to the last word of a phrase, unless it already ends in
/// "ng" (a scale word such as "daang" carries its own linker).
fn link_last_word(words: &str) -> String {
    match words.rfind(' ') {
        None => {
            // Single word
            if words.ends_with("ng") {
                return words.to_string(); // Already has linker
            }
            add_linker(words)
        }
        Some(last_space_idx) => {
            // Multi-word: add linker to last word
            let (prefix, last_word) = words.split_at(last_space_idx + 1);
            if last_word.ends_with("ng") {
                return words.to_string(); // Already has linker
            }
            format!("{prefix}{}", add_linker(last_word))
        }
    }
}

// ============================================================================
// Segment Building
// ============================================================================

/// Words for a 0-999 segment.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones = (n % 10) as usize;
    let tens_digit = ((n / 10) % 10) as usize;
    let hundreds_digit = (n / 100) as usize;

    let mut parts: Vec<String> = Vec::new();

    // Hundreds: "isang daan", "dalawang daan", "siyam na daan"
    if hundreds_digit > 0 {
        let hundred_prefix = add_linker(ONES[hundreds_digit]);
        parts.push(format!("{hundred_prefix} {HUNDRED}"));
    }

    // Tens and ones
    let tens_ones = (n % 100) as usize;

    if tens_ones == 0 {
        // Just hundreds
    } else if tens_ones < 10 {
        // Single digit
        parts.push(ONES[ones].to_string());
    } else if tens_ones < 20 {
        // Teens (10-19)
        parts.push(TEENS[ones].to_string());
    } else if ones == 0 {
        // Even tens (20, 30, 40, etc.)
        parts.push(TENS[tens_digit].to_string());
    } else if tens_digit == 5 {
        // limampu (50) gets special linker: "limampung anim" (56)
        parts.push(format!("{}ng {}", TENS[tens_digit], ONES[ones]));
    } else {
        // Tens + ones
        parts.push(format!("{} {}", TENS[tens_digit], ONES[ones]));
    }

    parts.join(" ")
}

/// Words for a 0-999 segment with the linker on the last word (for use
/// before a scale word).
fn build_segment_with_linker(n: u32) -> String {
    let segment_word = build_segment(n);
    if segment_word.is_empty() {
        return String::new();
    }
    link_last_word(&segment_word)
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32);
        }
    }

    build_large_number_words(n)
}

/// Words for one thousand or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segment_values = split_groups(n, 1000);

    let mut result = String::new();

    for (i, &segment) in segment_values.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }

        let scale_word = SCALE_WORDS.get(i).copied().unwrap_or("");

        if !result.is_empty() {
            result.push(' ');
        }

        if i == 0 {
            result.push_str(&build_segment(segment));
        } else {
            // Add linker to segment before scale word
            result.push_str(&build_segment_with_linker(segment));
            result.push(' ');
            result.push_str(scale_word);
        }
    }

    result
}

/// Per-digit reading of the digits after the decimal point.
fn decimal_part_to_words(decimal: &str) -> String {
    decimal
        .bytes()
        .map(|b| {
            let d = (b - b'0') as usize;
            if d == 0 { ZERO } else { ONES[d] }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Spells a number in Filipino words.
///
/// ```
/// # use n2words::fil_ph::to_cardinal;
/// assert_eq!(to_cardinal(12345)?, "labindalawang libong tatlong daang apatnapu lima");
/// assert_eq!(to_cardinal("3.14")?, "tatlo punto isa apat");
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
    // Decimals are read digit by digit, so only the integer part is bounded.
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

/// Ordinal words for a positive integer: una (1st), ikalawa (2nd), then
/// "ika-" + cardinal.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special forms for 1st and 2nd
    if n.is_one() {
        return ORDINAL_FIRST.to_string();
    }
    if n.to_u64() == Some(2) {
        return ORDINAL_SECOND.to_string();
    }

    // For 3+, use ika- prefix + cardinal
    format!("{ORDINAL_PREFIX}-{}", integer_to_words(n))
}

/// Spells a positive integer as a Filipino ordinal.
///
/// ```
/// # use n2words::fil_ph::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "una");
/// assert_eq!(to_ordinal(2)?, "ikalawa");
/// assert_eq!(to_ordinal(3)?, "ika-tatlo");
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
        currency: Currency = Currency::PHP, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Philippine pesos (piso and sentimo).
///
/// ```
/// # use n2words::fil_ph::to_currency;
/// assert_eq!(to_currency(42)?, "apatnapu dalawang piso");
/// assert_eq!(to_currency(1.5)?, "isang piso at limampung sentimo");
/// assert_eq!(to_currency(-5)?, "negatibo limang piso");
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

/// Spells an amount in any currency Filipino has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Filipino has no words for and
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
    let peso = words.major_at(0);

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }

    // Pesos part
    if !money.major.is_zero() || money.minor == 0 {
        let peso_words = integer_to_words(&money.major);
        if money.major.is_zero() {
            result.push_str(&peso_words);
        } else {
            // Add linker to the last word before "piso"
            result.push_str(&link_last_word(&peso_words));
        }
        result.push(' ');
        result.push_str(peso);
    }

    // Sentimos part
    if money.minor > 0 {
        // assert_minor_unit already guaranteed minor is 0 whenever the
        // currency has no minor unit, so reaching this branch implies a real
        // list of forms.
        let sentimo = words.minor_at(0);
        if !money.major.is_zero() {
            result.push_str(" at ");
        }
        let sentimo_words = integer_to_words(&BigUint::from(money.minor));
        // Add linker before "sentimo"
        result.push_str(&link_last_word(&sentimo_words));
        result.push(' ');
        result.push_str(sentimo);
    }

    Ok(result)
}

/// Registry entry for `fil-PH`.
pub static LANGUAGE: Language = Language {
    code: "fil-PH",
    name: "Filipino (Philippines)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
