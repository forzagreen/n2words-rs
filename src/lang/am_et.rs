//! Amharic (Ethiopia) (`am-ET`): Amharic as used in Ethiopia (Ge'ez script).
//!
//! - Ge'ez/Ethiopic script numerals
//! - Teens formed with the "አስራ" prefix: "አስራ አንድ" (11)
//! - Keeps "one" before hundred: "አንድ መቶ" (100)
//! - Short scale naming (ሺ, ሚሊዮን, ቢሊዮን)
//! - Per-digit decimal reading
//! - Ordinals add the "-ኛ" suffix to the cardinal; first is "አንደኛ"
//!
//! ```
//! use n2words::am_et::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "አርባ ሁለት");
//! assert_eq!(to_ordinal(42)?, "አርባ ሁለትኛ");
//! assert_eq!(to_currency(42.5)?, "አርባ ሁለት ብር ሃምሳ ሳንቲም");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::AM as CURRENCY_VOCAB;
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
    "",
    "አንድ",
    "ሁለት",
    "ሶስት",
    "አራት",
    "አምስት",
    "ስድስት",
    "ሰባት",
    "ስምንት",
    "ዘጠኝ",
];
const TEENS: [&str; 10] = [
    "አስር",
    "አስራ አንድ",
    "አስራ ሁለት",
    "አስራ ሶስት",
    "አስራ አራት",
    "አስራ አምስት",
    "አስራ ስድስት",
    "አስራ ሰባት",
    "አስራ ስምንት",
    "አስራ ዘጠኝ",
];
const TENS: [&str; 10] = [
    "",
    "",
    "ሃያ",
    "ሰላሳ",
    "አርባ",
    "ሃምሳ",
    "ስልሳ",
    "ሰባ",
    "ሰማንያ",
    "ዘጠና",
];

const HUNDRED: &str = "መቶ";
const THOUSAND: &str = "ሺ";

const ZERO: &str = "ዜሮ";
const NEGATIVE: &str = "አሉታዊ";
const DECIMAL_SEP: &str = "ነጥብ";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

/// Ordinal suffix.
const ORDINAL_SUFFIX: &str = "ኛ";

/// Special ordinal for first.
const FIRST: &str = "አንደኛ";

/// Short scale; index 0 is the (empty) units slot.
const SCALE_WORDS: &[&str] = &["", THOUSAND, "ሚሊዮን", "ቢሊዮን"];

/// The cardinal ceiling, derived from the scale table: `10^12`.
///
/// `SCALE_WORDS` reaches ቢሊዮን (10^9), so values must stay below
/// `10^(SCALE_WORDS.len() * 3)`. Decimals are read digit by digit, so they
/// have no ceiling.
pub const CARDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The ordinal ceiling: `10^12` (ordinals build on the cardinal).
pub const ORDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The currency ceiling: `10^12` (currency builds on the cardinal).
pub const CURRENCY_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);

// ============================================================================
// Segment building
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

    // Amharic keeps "one" before hundred: "አንድ መቶ" (100).
    if hundreds_digit > 0 {
        parts.push(format!("{} {HUNDRED}", ONES[hundreds_digit]));
    }

    if tens_digit == 1 {
        parts.push(TEENS[ones].to_string());
    } else {
        if tens_digit > 1 {
            parts.push(TENS[tens_digit].to_string());
        }
        if ones > 0 {
            parts.push(ONES[ones].to_string());
        }
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

/// Words for a number of 1000 or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    let mut parts: Vec<String> = Vec::new();
    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if scale_index == 0 {
            parts.push(build_segment(segment));
        } else {
            let scale_word = SCALE_WORDS.get(scale_index).copied().unwrap_or("");
            parts.push(format!("{} {scale_word}", build_segment(segment)));
        }
    }

    parts.join(" ")
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

/// Spells a number in Amharic words.
///
/// ```
/// # use n2words::am_et::to_cardinal;
/// assert_eq!(to_cardinal(1001)?, "አንድ ሺ አንድ");
/// assert_eq!(to_cardinal("3.14")?, "ሶስት ነጥብ አንድ አራት");
/// assert_eq!(to_cardinal(-42)?, "አሉታዊ አርባ ሁለት");
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

/// Ordinal words for a positive integer.
///
/// In Amharic, ordinals are formed by adding the -ኛ suffix to the cardinal.
/// Special case: 1 = አንደኛ (not አንድኛ).
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special case: 1 → አንደኛ
    if n.is_one() {
        return FIRST.to_string();
    }

    // Get cardinal form and add ordinal suffix
    let mut cardinal = integer_to_words(n);
    cardinal.push_str(ORDINAL_SUFFIX);
    cardinal
}

/// Spells a positive integer as an Amharic ordinal.
///
/// ```
/// # use n2words::am_et::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "አንደኛ");
/// assert_eq!(to_ordinal(2)?, "ሁለትኛ");
/// assert_eq!(to_ordinal(100)?, "አንድ መቶኛ");
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
        currency: Currency = Currency::ETB, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Ethiopian birr.
///
/// ```
/// # use n2words::am_et::to_currency;
/// assert_eq!(to_currency(1)?, "አንድ ብር");
/// assert_eq!(to_currency(0.99)?, "ዘጠና ዘጠኝ ሳንቲም");
/// assert_eq!(to_currency(0.01)?, "አንድ ሳንቲም");
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

/// Spells an amount in any currency Amharic has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Amharic has no words for and
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

    // Birr part: shown if non-zero, or if there are no santim. The nouns are
    // invariable, so the first form serves every count.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(0));
    }

    // Santim part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(0));
    }

    Ok(result)
}

/// Registry entry for `am-ET`.
pub static LANGUAGE: Language = Language {
    code: "am-ET",
    name: "Amharic (Ethiopia)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
