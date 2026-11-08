//! Hausa (Nigeria) (`ha-NG`): Hausa as used in Nigeria.
//!
//! - Authentic Boko orthography with ɗ (hooked d) and ' (glottal stop)
//! - Teens with the "sha" prefix: "sha ɗaya" (11)
//! - Compound numbers with the "da" connector: "ashirin da ɗaya" (21)
//! - Arabic loanwords for tens (ashirin, talatin, arba'in, ...)
//! - Reversed multiplier order: "biyu ɗari" (200), "biyu dubu" (2000)
//! - Implicit one before ɗari and dubu: "ɗari" (100), "dubu" (1000)
//! - Per-digit decimal reading
//! - Ordinals are "na" + cardinal; first is "na farko"
//!
//! ```
//! use n2words::ha_ng::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "arba'in da biyu");
//! assert_eq!(to_ordinal(42)?, "na arba'in da biyu");
//! assert_eq!(to_currency(42.5)?, "arba'in da biyu naira da hamsin kobo");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::HA as CURRENCY_VOCAB;
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
    "", "ɗaya", "biyu", "uku", "huɗu", "biyar", "shida", "bakwai", "takwas", "tara",
];
const TEENS: [&str; 10] = [
    "goma",
    "sha ɗaya",
    "sha biyu",
    "sha uku",
    "sha huɗu",
    "sha biyar",
    "sha shida",
    "sha bakwai",
    "sha takwas",
    "sha tara",
];
/// Arabic loanwords for tens.
const TENS: [&str; 10] = [
    "", "", "ashirin", "talatin", "arba'in", "hamsin", "sittin", "saba'in", "tamanin", "tis'in",
];

const HUNDRED: &str = "ɗari";
const THOUSAND: &str = "dubu";

const ZERO: &str = "sifiri";
const NEGATIVE: &str = "babu";
const DECIMAL_SEP: &str = "digo";

/// Short scale; index 0 is the (empty) units slot.
const SCALE_WORDS: &[&str] = &["", THOUSAND, "miliyan", "biliyan"];

/// The cardinal ceiling, derived from the scale table: `10^12`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The ordinal ceiling: `10^12`.
pub const ORDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The currency ceiling: `10^12`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

// Hausa ordinals: "na" + cardinal (na ɗaya = 1st, na biyu = 2nd).
// First has the special form "na fari" or "farko".
const ORDINAL_PREFIX: &str = "na";
const ORDINAL_FIRST: &str = "na farko";

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment with Hausa patterns.
///
/// Hausa uses reversed order for hundreds, "biyu ɗari" (200), and the "da"
/// connector for ones, "ashirin da ɗaya" (21).
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones = (n % 10) as usize;
    let tens_digit = ((n / 10) % 10) as usize;
    let hundreds_digit = (n / 100) as usize;

    let mut parts: Vec<String> = Vec::new();

    // Hundreds: implicit one, or "biyu ɗari" (reversed order).
    if hundreds_digit > 0 {
        if hundreds_digit == 1 {
            parts.push(HUNDRED.to_string());
        } else {
            // Reversed: multiplier + hundred word
            parts.push(format!("{} {HUNDRED}", ONES[hundreds_digit]));
        }
    }

    // Tens and ones
    let tens_ones = n % 100;

    if tens_ones == 0 {
        // Just hundreds
    } else if tens_ones < 10 {
        // Single digit, with the "da" connector if after hundreds
        if hundreds_digit > 0 {
            parts.push(format!("da {}", ONES[ones]));
        } else {
            parts.push(ONES[ones].to_string());
        }
    } else if tens_ones < 20 {
        // Teens (10-19): "sha X"
        parts.push(TEENS[ones].to_string());
    } else if ones == 0 {
        // Even tens (20, 30, 40, ...)
        parts.push(TENS[tens_digit].to_string());
    } else {
        // Tens + ones with the "da" connector
        parts.push(format!("{} da {}", TENS[tens_digit], ONES[ones]));
    }

    parts.join(" ")
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
        if small < 1000 {
            return build_segment(small as u32);
        }
    }

    build_large_number_words(n)
}

/// Whether `word` is a single digit (1-9).
fn is_single_digit(word: &str) -> bool {
    ONES[1..].contains(&word)
}

/// Words for a number of 1000 or more.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    // Build raw parts (segment words and scale words), most significant first.
    let mut raw_parts: Vec<String> = Vec::new();
    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        raw_parts.push(build_segment(segment));
        if scale_index > 0 {
            let scale_word = SCALE_WORDS.get(scale_index).copied().unwrap_or("");
            raw_parts.push(scale_word.to_string());
        }
    }

    // Filter out the implicit "ɗaya" before ɗari or dubu.
    let mut filtered: Vec<&str> = Vec::new();
    for (i, part) in raw_parts.iter().enumerate() {
        let next_part = raw_parts.get(i + 1).map(String::as_str);
        // Skip "ɗaya" before ɗari or dubu (implicit one)
        if part == "ɗaya" && matches!(next_part, Some(HUNDRED | THOUSAND)) {
            continue;
        }
        filtered.push(part);
    }

    // Join with the correct separators.
    let mut result = String::new();
    for (i, &part) in filtered.iter().enumerate() {
        let prev_part = if i > 0 { Some(filtered[i - 1]) } else { None };

        // Use the "da" connector when the current part is a single digit
        // following a scale word.
        match prev_part {
            Some(prev)
                if is_single_digit(part)
                    && (prev == THOUSAND || prev == HUNDRED || SCALE_WORDS.contains(&prev)) =>
            {
                result.push_str(" da ");
            }
            Some(_) => result.push(' '),
            None => {}
        }

        result.push_str(part);
    }

    result
}

/// Words for the digits after the decimal point, read one by one.
fn decimal_part_to_words(decimal: &str) -> String {
    let digits: Vec<&str> = decimal
        .bytes()
        .map(|b| {
            let d = (b - b'0') as usize;
            if d == 0 { ZERO } else { ONES[d] }
        })
        .collect();
    digits.join(" ")
}

/// Spells a number in Hausa words.
///
/// ```
/// # use n2words::ha_ng::to_cardinal;
/// assert_eq!(to_cardinal(200)?, "biyu ɗari");
/// assert_eq!(to_cardinal(1001)?, "dubu da ɗaya");
/// assert_eq!(to_cardinal("3.14")?, "uku digo ɗaya huɗu");
/// assert_eq!(to_cardinal(-42)?, "babu arba'in da biyu");
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
    // Decimals are read digit by digit, so only the integer part has a ceiling.
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

/// Ordinal words for a positive integer: na farko (1st), na biyu (2nd),
/// na uku (3rd), ...
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special form for first
    if n.is_one() {
        return ORDINAL_FIRST.to_string();
    }

    // For 2+, use the "na" prefix + cardinal
    format!("{ORDINAL_PREFIX} {}", integer_to_words(n))
}

/// Spells a positive integer as a Hausa ordinal.
///
/// ```
/// # use n2words::ha_ng::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "na farko");
/// assert_eq!(to_ordinal(2)?, "na biyu");
/// assert_eq!(to_ordinal(10)?, "na goma");
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
        currency: Currency = Currency::NGN, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Nigerian naira (naira and kobo, 100 kobo = 1 naira).
///
/// ```
/// # use n2words::ha_ng::to_currency;
/// assert_eq!(to_currency(42)?, "arba'in da biyu naira");
/// assert_eq!(to_currency(1.5)?, "ɗaya naira da hamsin kobo");
/// assert_eq!(to_currency(-5)?, "babu biyar naira");
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

/// Spells an amount in any currency Hausa has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Hausa has no words for and
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

    // Naira part: shown if non-zero, or if there are no kobo. The nouns are
    // invariable, so the first form serves every count.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(0));
    }

    // Kobo part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(" da ");
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(0));
    }

    Ok(result)
}

/// Registry entry for `ha-NG`.
pub static LANGUAGE: Language = Language {
    code: "ha-NG",
    name: "Hausa (Nigeria)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
