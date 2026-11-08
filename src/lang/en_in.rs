//! English (India) (`en-IN`).
//!
//! - Indian numbering system (thousand, lakh, crore, arab, kharab, ...)
//! - 3-2-2 grouping pattern (last 3 digits, then groups of 2)
//! - British-style "and" after hundreds: "one hundred and twenty-three"
//! - "and" before a final segment below one hundred: "one lakh and one"
//! - Hyphenated tens-ones: "twenty-one", "forty-two"
//! - Currency: Indian rupee (INR) - rupee/rupees, paisa/paise
//!
//! ```
//! use n2words::en_in::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(
//!     to_cardinal(1234567)?,
//!     "twelve lakh thirty-four thousand five hundred and sixty-seven"
//! );
//! assert_eq!(to_ordinal(100001)?, "one lakh and first");
//! assert_eq!(to_currency(42.5)?, "forty-two rupees and fifty paise");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::EN as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{
    Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal, split_groups_indian,
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

/// Indian numbering scales: 10^3, 10^5, 10^7, 10^9, 10^11, 10^13, 10^15, 10^17.
const SCALES: &[&str] = &[
    "thousand", "lakh", "crore", "arab", "kharab", "neel", "padma", "shankh",
];

/// The cardinal ceiling, from the 3-2-2 grouping over the scale table: `10^19`.
pub const CARDINAL_MAX: Max = Max::indian(SCALES.len() as u32 + 1);
/// The ordinal ceiling: `10^19`.
pub const ORDINAL_MAX: Max = Max::indian(SCALES.len() as u32 + 1);
/// The currency ceiling: `10^19`.
pub const CURRENCY_MAX: Max = Max::indian(SCALES.len() as u32 + 1);

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

/// Words for a 0-999 segment (British-style, "and" after hundreds), and
/// whether it has a hundreds place.
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

/// Words for a 0-99 segment (no hundreds): the two-digit scale groups.
fn build_small_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = (n / 10) as usize;

    if tens == 1 {
        TEENS[ones].to_string()
    } else if tens >= 2 {
        if ones > 0 {
            format!("{}-{}", TENS[tens], ONES[ones])
        } else {
            TENS[tens].to_string()
        }
    } else {
        ONES[ones].to_string()
    }
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer, in South Asian 3-2-2 grouping: a 3-digit
/// units segment, then 2 digits per scale word (thousand, lakh, crore, ...).
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32).0;
        }
    }

    // Segments least-significant first: [units (0-999), thousand, lakh, ...].
    let segments = split_groups_indian(n, 1000, 100);

    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if i == 0 {
            // The units segment can be 0-999; "and" before it when earlier
            // segments exist and it has no hundred of its own.
            let (word, has_hundred) = build_segment(segment);
            if !result.is_empty() && !has_hundred {
                result.push_str(" and");
            }
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&word);
        } else {
            // The other segments are 0-99, each followed by its scale word.
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&build_small_segment(segment));
            if let Some(scale) = SCALES.get(i - 1) {
                result.push(' ');
                result.push_str(scale);
            }
        }
    }
    result
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
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

/// Spells a number in Indian English words.
///
/// ```
/// # use n2words::en_in::to_cardinal;
/// assert_eq!(to_cardinal(100000)?, "one lakh");
/// assert_eq!(to_cardinal(10000001)?, "one crore and one");
/// assert_eq!(to_cardinal(-17.42)?, "minus seventeen point forty-two");
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

/// Ordinal words for a 0-999 segment (the final segment only):
/// "first", "twenty-third", "one hundred and forty-fifth".
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

/// Ordinal words for a positive integer: cardinal segments, then the lowest
/// non-zero segment in ordinal form ("one lakhth", "one lakh and first").
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_ordinal_segment(small as u32);
        }
    }

    // Segments least-significant first, in the 3-2-2 pattern.
    let segments = split_groups_indian(n, 1000, 100);
    let lowest_non_zero = segments.iter().position(|&s| s != 0).unwrap_or(0);

    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let is_lowest = i == lowest_non_zero;
        // "and" before a final units segment below 100, as in the cardinal:
        // "one lakh and first".
        if !result.is_empty() {
            result.push_str(if is_lowest && i == 0 && segment < 100 {
                " and "
            } else {
                " "
            });
        }
        if is_lowest {
            if i == 0 {
                result.push_str(&build_ordinal_segment(segment));
            } else {
                // Scale position with nothing below: "one lakhth".
                result.push_str(&build_small_segment(segment));
                result.push(' ');
                result.push_str(SCALES.get(i - 1).copied().unwrap_or(""));
                result.push_str("th");
            }
        } else if i == 0 {
            result.push_str(&build_segment(segment).0);
        } else {
            result.push_str(&build_small_segment(segment));
            result.push(' ');
            result.push_str(SCALES.get(i - 1).copied().unwrap_or(""));
        }
    }
    result
}

/// Spells a positive integer as an Indian English ordinal.
///
/// ```
/// # use n2words::en_in::to_ordinal;
/// assert_eq!(to_ordinal(101)?, "one hundred and first");
/// assert_eq!(to_ordinal(100000)?, "one lakhth");
/// assert_eq!(to_ordinal(10100000)?, "one crore one lakhth");
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
        /// Use "and" between rupees and paise
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::INR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Indian rupees.
///
/// ```
/// # use n2words::en_in::to_currency;
/// assert_eq!(to_currency(1)?, "one rupee");
/// assert_eq!(to_currency(100000)?, "one lakh rupees");
/// assert_eq!(to_currency(0.5)?, "fifty paise");
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
/// # use n2words::en_in::{to_currency_with, CurrencyOptions};
/// let opts = CurrencyOptions { and: false, ..Default::default() };
/// assert_eq!(to_currency_with(42.5, &opts)?, "forty-two rupees fifty paise");
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

/// Registry entry for `en-IN`.
pub static LANGUAGE: Language = Language {
    code: "en-IN",
    name: "English (India)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
