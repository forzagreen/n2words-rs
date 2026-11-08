//! British English (`en-GB`).
//!
//! - "and" after hundreds: "one hundred and twenty-three"
//! - "and" before the final segment: "one million and one"
//! - Hyphenated tens-ones: "twenty-one", "forty-two"
//! - Short scale (billion = 10^9) up to vigintillion (10^63)
//!
//! ```
//! use n2words::en_gb::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(101)?, "one hundred and one");
//! assert_eq!(to_ordinal(101)?, "one hundred and first");
//! assert_eq!(to_currency(42.5)?, "forty-two pounds and fifty pence");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::EN as CURRENCY_VOCAB;
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
    "", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
];
const TEENS: [&str; 10] = [
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
];
const TENS: [&str; 10] = [
    "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];
const SCALES: &[&str] = &[
    "thousand",
    "million",
    "billion",
    "trillion",
    "quadrillion",
    "quintillion",
    "sextillion",
    "septillion",
    "octillion",
    "nonillion",
    "decillion",
    "undecillion",
    "duodecillion",
    "tredecillion",
    "quattuordecillion",
    "quindecillion",
    "sexdecillion",
    "septendecillion",
    "octodecillion",
    "novemdecillion",
    "vigintillion",
];

/// The cardinal ceiling, derived from the scale table: `10^66`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32);
/// The ordinal ceiling: `10^66`.
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32);
/// The currency ceiling: `10^66`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32);

const HUNDRED: &str = "hundred";
const ZERO: &str = "zero";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "point";

const ORDINAL_ONES: [&str; 10] = [
    "", "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
];
const ORDINAL_TEENS: [&str; 10] = [
    "tenth",
    "eleventh",
    "twelfth",
    "thirteenth",
    "fourteenth",
    "fifteenth",
    "sixteenth",
    "seventeenth",
    "eighteenth",
    "nineteenth",
];
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "twentieth",
    "thirtieth",
    "fortieth",
    "fiftieth",
    "sixtieth",
    "seventieth",
    "eightieth",
    "ninetieth",
];

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment, and whether it has a hundreds place.
fn build_segment(n: u32) -> (String, bool) {
    if n == 0 {
        return (String::new(), false);
    }
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let tens_ones = if tens == 1 {
        TEENS[ones].to_string()
    } else if tens >= 2 {
        if ones > 0 {
            format!("{}-{}", TENS[tens], ONES[ones])
        } else {
            TENS[tens].to_string()
        }
    } else if ones > 0 {
        ONES[ones].to_string()
    } else {
        String::new()
    };

    if hundreds > 0 {
        if tens_ones.is_empty() {
            (format!("{} {HUNDRED}", ONES[hundreds]), true)
        } else {
            (
                format!("{} {HUNDRED} and {tens_ones}", ONES[hundreds]),
                true,
            )
        }
    } else {
        (tens_ones, false)
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
        if small < 1000 {
            return build_segment(small as u32).0;
        }
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;
            let mut result = format!("{} {}", build_segment(thousands).0, SCALES[0]);
            if remainder > 0 {
                let (word, has_hundred) = build_segment(remainder);
                result.push_str(if has_hundred { " " } else { " and " });
                result.push_str(&word);
            }
            return result;
        }
    }
    build_large_number_words(n)
}

/// Words for one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let first_non_zero = segments.iter().position(|&s| s != 0);

    let mut result = String::new();
    let mut prev_was_scale = false;
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let (word, has_hundred) = build_segment(segment);
        let is_last = first_non_zero == Some(i);
        // "and" only before the final segment when it follows a scale word
        // and has no hundreds of its own.
        if !result.is_empty() && is_last && prev_was_scale && !has_hundred {
            result.push_str(" and");
        }
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&word);
        if i > 0 {
            result.push(' ');
            result.push_str(SCALES[i - 1]);
            prev_was_scale = true;
        } else {
            prev_was_scale = false;
        }
    }
    result
}

/// Words for the digits after the decimal point.
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

/// Spells a number in British English words.
///
/// ```
/// # use n2words::en_gb::to_cardinal;
/// assert_eq!(to_cardinal(1001)?, "one thousand and one");
/// assert_eq!(to_cardinal(-3.14)?, "minus three point fourteen");
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

/// Ordinal words for a 0-999 segment: "first", "twenty-third", "one hundred and forty-fifth".
fn build_ordinal_segment(n: u32) -> String {
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let tens_ones = if tens == 1 {
        ORDINAL_TEENS[ones].to_string()
    } else if tens >= 2 {
        if ones > 0 {
            format!("{}-{}", TENS[tens], ORDINAL_ONES[ones])
        } else {
            ORDINAL_TENS[tens].to_string()
        }
    } else if ones > 0 {
        ORDINAL_ONES[ones].to_string()
    } else {
        String::new()
    };

    if hundreds > 0 {
        if tens_ones.is_empty() {
            format!("{} hundredth", ONES[hundreds])
        } else {
            format!("{} {HUNDRED} and {tens_ones}", ONES[hundreds])
        }
    } else {
        tens_ones
    }
}

/// Ordinal words for a positive integer.
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_ordinal_segment(small as u32);
        }
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;
            let thousands_word = build_segment(thousands).0;
            if remainder == 0 {
                return format!("{thousands_word} {}th", SCALES[0]);
            }
            // "and" before a remainder below 100, as in the cardinal.
            let connector = if remainder < 100 { " and " } else { " " };
            return format!(
                "{thousands_word} {}{connector}{}",
                SCALES[0],
                build_ordinal_segment(remainder)
            );
        }
    }
    build_large_ordinal(n)
}

/// Ordinal words for one million or more.
fn build_large_ordinal(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let lowest_non_zero = segments.iter().position(|&s| s != 0).unwrap_or(0);

    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let is_lowest = i == lowest_non_zero;
        if !result.is_empty() {
            result.push_str(if is_lowest && segment < 100 {
                " and "
            } else {
                " "
            });
        }
        if is_lowest {
            if i == 0 {
                result.push_str(&build_ordinal_segment(segment));
            } else {
                result.push_str(&format!("{} {}th", build_segment(segment).0, SCALES[i - 1]));
            }
        } else {
            result.push_str(&build_segment(segment).0);
            if i > 0 {
                result.push(' ');
                result.push_str(SCALES[i - 1]);
            }
        }
    }
    result
}

/// Spells a positive integer as a British English ordinal.
///
/// ```
/// # use n2words::en_gb::to_ordinal;
/// assert_eq!(to_ordinal(1001)?, "one thousand and first");
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
        /// Use "and" between pounds and pence (e.g., "one pound and fifty pence")
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::GBP, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in pounds sterling.
///
/// ```
/// # use n2words::en_gb::to_currency;
/// assert_eq!(to_currency(0.01)?, "one penny");
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

/// Spells an amount in any currency English has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) and
/// [`Error::NoMinorUnit`](crate::Error::NoMinorUnit).
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
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(if money.major.is_one() { 0 } else { 1 }));
    }
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(if options.and { " and " } else { " " });
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(if money.minor == 1 { 0 } else { 1 }));
    }
    Ok(result)
}

/// Registry entry for `en-GB`.
pub static LANGUAGE: Language = Language {
    code: "en-GB",
    name: "British English",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
