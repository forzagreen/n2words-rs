//! Azerbaijani (Azerbaijan) (`az-AZ`).
//!
//! - Turkic language patterns
//! - Implicit "bir" (one) omission before hundreds and thousands: "yüz", "min"
//! - Short scale (milyar = 10^9) up to kentilyon (10^18)
//! - Ordinals add a vowel-harmony suffix to the space-less cardinal: "iyirmibirinci"
//!
//! ```
//! use n2words::az_az::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "qırx iki");
//! assert_eq!(to_ordinal(21)?, "iyirmibirinci");
//! assert_eq!(to_currency(42.5)?, "qırx iki manat əlli qəpik");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::AZ as CURRENCY_VOCAB;
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
    "", "bir", "iki", "üç", "dörd", "beş", "altı", "yeddi", "səkkiz", "doqquz",
];
const TEENS: [&str; 10] = [
    "on",
    "on bir",
    "on iki",
    "on üç",
    "on dörd",
    "on beş",
    "on altı",
    "on yeddi",
    "on səkkiz",
    "on doqquz",
];
const TENS: [&str; 10] = [
    "", "", "iyirmi", "otuz", "qırx", "əlli", "altmış", "yetmiş", "səksən", "doxsan",
];

const HUNDRED: &str = "yüz";
const THOUSAND: &str = "min";

const ZERO: &str = "sıfır";
const NEGATIVE: &str = "mənfi";
const DECIMAL_SEP: &str = "nöqtə";

/// Short scale, indexed by 3-digit segment (0 = units, 1 = thousands, ...).
const SCALE_WORDS: &[&str] = &[
    "",
    THOUSAND,
    "milyon",
    "milyar",
    "trilyon",
    "katrilyon",
    "kentilyon",
];

/// The cardinal ceiling, derived from the scale table: `10^21`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The ordinal ceiling: `10^21` (the ordinal is the cardinal plus a suffix).
pub const ORDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);
/// The currency ceiling: `10^21`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_WORDS.len() as u32 - 1);

// Azerbaijani ordinals take an -(i/ı/u/ü)nci suffix with vowel harmony; 1-10
// are irregular (index 0 unused).
const ORDINAL_SPECIAL: [&str; 11] = [
    "",
    "birinci",
    "ikinci",
    "üçüncü",
    "dördüncü",
    "beşinci",
    "altıncı",
    "yeddinci",
    "səkkizinci",
    "doqquzuncu",
    "onuncu",
];

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment; "bir" is omitted before "yüz".
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<&str> = Vec::new();
    if hundreds > 0 {
        if hundreds > 1 {
            parts.push(ONES[hundreds]);
        }
        parts.push(HUNDRED);
    }
    if tens == 1 {
        parts.push(TEENS[ones]);
    } else {
        if tens > 1 {
            parts.push(TENS[tens]);
        }
        if ones > 0 {
            parts.push(ONES[ones]);
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

/// Words for a number of one thousand or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let mut parts: Vec<String> = Vec::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let scale_word = SCALE_WORDS.get(i).copied().unwrap_or("");
        parts.push(if i == 0 {
            build_segment(segment)
        } else if i == 1 && segment == 1 {
            // Omit "bir" before thousand
            scale_word.to_string()
        } else {
            format!("{} {scale_word}", build_segment(segment))
        });
    }
    parts.join(" ")
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

/// Spells a number in Azerbaijani words.
///
/// ```
/// # use n2words::az_az::to_cardinal;
/// assert_eq!(to_cardinal(1243)?, "min iki yüz qırx üç");
/// assert_eq!(to_cardinal(-17.42)?, "mənfi on yeddi nöqtə qırx iki");
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

/// The ordinal suffix by Azerbaijani vowel harmony: the last vowel of the
/// word decides between back (a, ı, o, u) and front (ə, e, i, ö, ü) forms.
fn ordinal_suffix(word: &str) -> &'static str {
    for c in word.chars().rev() {
        match c {
            'o' | 'u' => return "uncu",
            'a' | 'ı' => return "ıncı",
            'ö' | 'ü' => return "üncü",
            'ə' | 'e' | 'i' => return "inci",
            _ => {}
        }
    }
    "inci"
}

/// Ordinal words for a positive integer: irregular forms for 1-10, otherwise
/// the cardinal without spaces plus a harmonised suffix.
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if (1..=10).contains(&small) {
            return ORDINAL_SPECIAL[small as usize].to_string();
        }
    }
    let mut cardinal = integer_to_words(n).replace(' ', "");
    cardinal.push_str(ordinal_suffix(&cardinal));
    cardinal
}

/// Spells a positive integer as an Azerbaijani ordinal.
///
/// ```
/// # use n2words::az_az::to_ordinal;
/// assert_eq!(to_ordinal(3)?, "üçüncü");
/// assert_eq!(to_ordinal(40)?, "qırxıncı");
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
    // The ordinal builds on the cardinal, so it shares the ceiling.
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
        currency: Currency = Currency::AZN, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Azerbaijani manat (100 qəpik = 1 manat).
///
/// ```
/// # use n2words::az_az::to_currency;
/// assert_eq!(to_currency(42)?, "qırx iki manat");
/// assert_eq!(to_currency(-1)?, "mənfi bir manat");
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

/// Spells an amount in any currency Azerbaijani has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Azerbaijani has no words for and
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
    // Manat part: the nouns are invariable, so the first form serves every count.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(0));
    }
    // Qəpik part
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

/// Registry entry for `az-AZ`.
pub static LANGUAGE: Language = Language {
    code: "az-AZ",
    name: "Azerbaijani (Azerbaijan)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
