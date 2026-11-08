//! Swahili (Kenya) (`sw-KE`): Swahili as used in Kenya.
//!
//! - "na" connector for compound numbers: "ishirini na moja" (21)
//! - Reversed hundreds: "mia moja" (one hundred)
//! - Scale words before their multiplier: "elfu moja" (1000), "milioni mbili"
//! - Ordinals are "wa" + cardinal; first and second are "wa kwanza", "wa pili"
//!
//! ```
//! use n2words::sw_ke::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "arobaini na mbili");
//! assert_eq!(to_ordinal(42)?, "wa arobaini na mbili");
//! assert_eq!(to_currency(42.5)?, "shilingi arobaini na mbili na senti hamsini");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::SW as CURRENCY_VOCAB;
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
    "sifuri", "moja", "mbili", "tatu", "nne", "tano", "sita", "saba", "nane", "tisa",
];
/// Tens words, indexed by the tens digit (index 0 unused).
const TENS: [&str; 10] = [
    "",
    "kumi",
    "ishirini",
    "thelathini",
    "arobaini",
    "hamsini",
    "sitini",
    "sabini",
    "themanini",
    "tisini",
];

/// Scale words; index 0 is the (empty) units slot.
const SCALE_WORDS: &[&str] = &[
    "",
    "elfu",
    "milioni",
    "bilioni",
    "trilioni",
    "kwadrilioni",
    "kwintilioni",
];

/// The cardinal ceiling, derived from the scale table: `10^21`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The ordinal ceiling: `10^21`.
pub const ORDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The currency ceiling: `10^21`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);

const ZERO: &str = "sifuri";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "nukta";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

// Swahili ordinals use "wa" + cardinal: wa kwanza (1st), wa pili (2nd).
// The first few have special forms; index 0 is unused.
const ORDINAL_ONES: [&str; 10] = [
    "",
    "wa kwanza",
    "wa pili",
    "wa tatu",
    "wa nne",
    "wa tano",
    "wa sita",
    "wa saba",
    "wa nane",
    "wa tisa",
];
const ORDINAL_PREFIX: &str = "wa";

// ============================================================================
// Cardinal
// ============================================================================

/// Words for 0-99.
fn words_under_100(n: u32) -> String {
    if n < 10 {
        return ONES[n as usize].to_string();
    }
    if n == 10 {
        return TENS[1].to_string();
    }
    if n < 20 {
        // 11-19: 'kumi na <digit>'
        return format!("{} na {}", TENS[1], ONES[(n - 10) as usize]);
    }
    let tens = (n / 10) as usize;
    let ones = (n % 10) as usize;
    if ones == 0 {
        return TENS[tens].to_string();
    }
    format!("{} na {}", TENS[tens], ONES[ones])
}

/// Words for 0-999.
fn words_under_1000(n: u32) -> String {
    if n < 100 {
        return words_under_100(n);
    }
    if n == 100 {
        return "mia moja".to_string();
    }
    let hundreds = (n / 100) as usize;
    let rest = n % 100;

    // Hundreds: 'mia <digit>'
    let mut result = format!("mia {}", ONES[hundreds]);
    if rest > 0 {
        if rest < 10 {
            result.push_str(" na ");
            result.push_str(ONES[rest as usize]);
        } else {
            result.push(' ');
            result.push_str(&words_under_100(rest));
        }
    }

    result
}

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Segments stored least-significant first: [ones, thousands, millions, ...]
    let segments = split_groups(n, 1000);
    let mut parts: Vec<String> = Vec::new();

    // Iterate from highest scale to lowest
    for (scale_index, &val) in segments.iter().enumerate().rev() {
        if val == 0 {
            continue;
        }

        if scale_index == 0 {
            // Units segment
            if val < 10 && !parts.is_empty() {
                parts.push(format!("na {}", ONES[val as usize]));
            } else if val == 100 && !parts.is_empty() {
                // In compound numbers (e.g., 1100 -> 'elfu moja mia'), use
                // 'mia' not 'mia moja'
                parts.push("mia".to_string());
            } else {
                parts.push(words_under_1000(val));
            }
        } else {
            // Scale segments: 'elfu moja', 'milioni mbili'
            let unit = if val == 1 {
                ONES[1].to_string()
            } else {
                words_under_1000(val)
            };
            parts.push(format!("{} {unit}", SCALE_WORDS[scale_index]));
        }
    }

    parts.join(" ")
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// the rest as a single number.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut result = String::new();

    let significant = decimal.trim_start_matches('0');
    for _ in 0..decimal.len() - significant.len() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(ZERO);
    }

    if !significant.is_empty() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&digits_to_biguint(significant)));
    }

    result
}

/// Spells a number in Swahili words.
///
/// ```
/// # use n2words::sw_ke::to_cardinal;
/// assert_eq!(to_cardinal(11)?, "kumi na moja");
/// assert_eq!(to_cardinal(3.14)?, "tatu nukta kumi na nne");
/// assert_eq!(to_cardinal(-5)?, "minus tano");
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
    // The decimal's significant digits are spelled as one number, so they
    // must clear the ceiling too.
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

/// Ordinal words for a positive integer: wa kwanza (1st), wa pili (2nd),
/// wa tatu (3rd), ...
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special forms for 1-9
    if let Some(small) = n.to_u64() {
        if (1..=9).contains(&small) {
            return ORDINAL_ONES[small as usize].to_string();
        }
    }

    // For 10+, use the "wa" prefix + cardinal
    format!("{ORDINAL_PREFIX} {}", integer_to_words(n))
}

/// Spells a positive integer as a Swahili ordinal.
///
/// ```
/// # use n2words::sw_ke::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "wa kwanza");
/// assert_eq!(to_ordinal(2)?, "wa pili");
/// assert_eq!(to_ordinal(10)?, "wa kumi");
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
        currency: Currency = Currency::KES, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Kenyan shillings (shilingi and senti,
/// 100 senti = 1 shilingi).
///
/// ```
/// # use n2words::sw_ke::to_currency;
/// assert_eq!(to_currency(42)?, "shilingi arobaini na mbili");
/// assert_eq!(to_currency(1.5)?, "shilingi moja na senti hamsini");
/// assert_eq!(to_currency(-5)?, "minus shilingi tano");
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

/// Spells an amount in any currency Swahili has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Swahili has no words for and
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

    // Shillings part: the noun comes first and is invariable.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(words.major_at(0));
        result.push(' ');
        result.push_str(&integer_to_words(&money.major));
    }

    // Senti part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(" na ");
        }
        result.push_str(words.minor_at(0));
        result.push(' ');
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
    }

    Ok(result)
}

/// Registry entry for `sw-KE`.
pub static LANGUAGE: Language = Language {
    code: "sw-KE",
    name: "Swahili (Kenya)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
