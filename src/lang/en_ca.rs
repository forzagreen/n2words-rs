//! Canadian English (`en-CA`).
//!
//! - Follows British English style for number words
//! - "and" after hundreds: "one hundred and twenty-three" (default)
//! - "and" before the final segment: "one million and one" (default)
//! - Hyphenated tens-ones: "twenty-one", "forty-two"
//! - Short scale (billion = 10^9) up to vigintillion (10^63)
//! - Optional hundred-pairing: 1500 → "fifteen hundred" (colloquial)
//! - Optional "and" omission: 101 → "one hundred one" (American style)
//! - Currency: Canadian dollar (CAD) - dollar/dollars, cent/cents
//!
//! ```
//! use n2words::en_ca::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(101)?, "one hundred and one");
//! assert_eq!(to_ordinal(101)?, "one hundred first");
//! assert_eq!(to_currency(42.5)?, "forty-two dollars and fifty cents");
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
// Shared helpers
// ============================================================================

/// Words for a 0-999 segment, and whether it has a hundreds place.
fn build_segment(n: u32, use_and: bool) -> (String, bool) {
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
            let connector = if use_and { " and " } else { " " };
            (
                format!("{} {HUNDRED}{connector}{tens_ones}", ONES[hundreds]),
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
fn integer_to_words(n: &BigUint, hundred_pairing: bool, use_and: bool) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32, use_and).0;
        }

        // Hundred-pairing: 1100-9999 → "eleven hundred" ... "ninety-nine hundred ninety-nine".
        if hundred_pairing && (1100..=9999).contains(&small) {
            let high = (small / 100) as u32;
            let low = (small % 100) as u32;
            let mut result = format!("{} {HUNDRED}", build_segment(high, use_and).0);
            if low > 0 {
                // "and" before the remainder when requested.
                if use_and {
                    result.push_str(" and");
                }
                result.push(' ');
                result.push_str(&build_segment(low, use_and).0);
            }
            return result;
        }

        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;
            let mut result = format!("{} {}", build_segment(thousands, use_and).0, SCALES[0]);
            if remainder > 0 {
                let (word, has_hundred) = build_segment(remainder, use_and);
                // "and" before the remainder when requested and it has no hundred.
                if use_and && !has_hundred {
                    result.push_str(" and");
                }
                result.push(' ');
                result.push_str(&word);
            }
            return result;
        }
    }

    build_large_number_words(n, use_and)
}

/// Words for a number of one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint, use_and: bool) -> String {
    let segments = split_groups(n, 1000);
    // The lowest non-zero segment is the one an "and" may precede.
    let first_non_zero = if use_and {
        segments.iter().position(|&s| s != 0)
    } else {
        None
    };

    let mut result = String::new();
    let mut prev_was_scale = false;
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let (word, has_hundred) = build_segment(segment, use_and);
        let is_last = first_non_zero == Some(i);
        // "and" only before the final segment when it follows a scale word
        // and has no hundreds of its own.
        if use_and && !result.is_empty() && is_last && prev_was_scale && !has_hundred {
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

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
fn decimal_part_to_words(decimal: &str, use_and: bool) -> String {
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
        result.push_str(&integer_to_words(
            &digits_to_biguint(remainder),
            false,
            use_and,
        ));
    }
    result
}

options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Use hundred-pairing for 1100-9999 (e.g., "fifteen hundred" instead of "one thousand five hundred")
        hundred_pairing: bool = false;
        /// Use "and" after hundreds and before final small numbers (default: true, Canadian/British style)
        and: bool = true;
    }
}

/// Spells a number in Canadian English words.
///
/// ```
/// # use n2words::en_ca::to_cardinal;
/// assert_eq!(to_cardinal(1001)?, "one thousand and one");
/// assert_eq!(to_cardinal(-1000000)?, "minus one million");
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

/// Spells a number in Canadian English words, with options.
///
/// ```
/// # use n2words::en_ca::{to_cardinal_with, CardinalOptions};
/// let american = CardinalOptions { and: false, ..Default::default() };
/// assert_eq!(to_cardinal_with(101, &american)?, "one hundred one");
/// let colloquial = CardinalOptions { hundred_pairing: true, ..Default::default() };
/// assert_eq!(to_cardinal_with(1500, &colloquial)?, "fifteen hundred");
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
    // Both the integer part and the fraction's significant digits are spelled
    // via the scale builder, so both must clear the ceiling.
    CARDINAL_MAX.check(&number.integer, number.fraction.as_deref())?;

    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }
    result.push_str(&integer_to_words(
        &number.integer,
        options.hundred_pairing,
        options.and,
    ));
    if let Some(fraction) = &number.fraction {
        result.push(' ');
        result.push_str(DECIMAL_SEP);
        result.push(' ');
        result.push_str(&decimal_part_to_words(fraction, options.and));
    }
    Ok(result)
}

// ============================================================================
// Ordinal
// ============================================================================

/// Ordinal words for a 0-999 segment: "first", "twenty-third", "one hundred forty-fifth".
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
            format!("{} {HUNDRED} {tens_ones}", ONES[hundreds])
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
            let thousands_word = build_segment(thousands, false).0;
            if remainder == 0 {
                return format!("{thousands_word} {}th", SCALES[0]);
            }
            return format!(
                "{thousands_word} {} {}",
                SCALES[0],
                build_ordinal_segment(remainder)
            );
        }
    }
    build_large_ordinal(n)
}

/// Ordinal words for one million or more: cardinal segments, then the lowest
/// non-zero segment in ordinal form.
fn build_large_ordinal(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
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
            if i == 0 {
                result.push_str(&build_ordinal_segment(segment));
            } else {
                result.push_str(&format!(
                    "{} {}th",
                    build_segment(segment, false).0,
                    SCALES[i - 1]
                ));
            }
        } else {
            result.push_str(&build_segment(segment, false).0);
            if i > 0 {
                result.push(' ');
                result.push_str(SCALES[i - 1]);
            }
        }
    }
    result
}

/// Spells a positive integer as a Canadian English ordinal.
///
/// ```
/// # use n2words::en_ca::to_ordinal;
/// assert_eq!(to_ordinal(21)?, "twenty-first");
/// assert_eq!(to_ordinal(1000000)?, "one millionth");
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
        /// Use "and" between dollars and cents (e.g., "one dollar and fifty cents")
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::CAD, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Canadian dollars.
///
/// ```
/// # use n2words::en_ca::to_currency;
/// assert_eq!(to_currency(1)?, "one dollar");
/// assert_eq!(to_currency(1000.99)?, "one thousand dollars and ninety-nine cents");
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
/// ```
/// # use n2words::en_ca::{to_currency_with, CurrencyOptions};
/// let opts = CurrencyOptions { and: false, ..Default::default() };
/// assert_eq!(to_currency_with(42.5, &opts)?, "forty-two dollars fifty cents");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency English has no words for and
/// [`Error::NoMinorUnit`](crate::Error::NoMinorUnit) for a fractional amount
/// in a currency without one.
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    // The currency decides how many decimal digits the parser keeps: a
    // 1000-subunit currency (TND, KWD, ...) read at 2 would turn "1.500" into
    // 50 minor units instead of 500.
    let words = CURRENCY_VOCAB.words(options.currency)?;
    let money = parse_currency(&value.into(), options.currency.minor_unit_digits())?;
    CURRENCY_MAX.check(&money.major, None)?;
    assert_minor_unit(money.minor, options.currency)?;

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }
    // Invariable nouns (taka, ringgit, naira, rand) have a single form, which
    // `major_at` serves for every count.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major, false, false));
        result.push(' ');
        result.push_str(words.major_at(if money.major.is_one() { 0 } else { 1 }));
    }
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(if options.and { " and " } else { " " });
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor), false, false));
        result.push(' ');
        result.push_str(words.minor_at(if money.minor == 1 { 0 } else { 1 }));
    }
    Ok(result)
}

/// Registry entry for `en-CA`.
pub static LANGUAGE: Language = Language {
    code: "en-CA",
    name: "Canadian English",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
