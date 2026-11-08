//! Turkish (Türkiye) (`tr-TR`).
//!
//! - Omits "bir" (one) before hundreds and thousands: "yüz", "bin"
//! - Optional `drop_spaces` for the compound form: "yirmibir"
//! - Short scale (milyar = 10^9) up to kentilyon (10^18)
//! - Ordinals add a vowel-harmony suffix to the compound cardinal: "yirmibirinci"
//!
//! ```
//! use n2words::tr_tr::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "yirmi bir");
//! assert_eq!(to_ordinal(21)?, "yirmibirinci");
//! assert_eq!(to_currency(1.5)?, "bir lira elli kuruş");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::TR as CURRENCY_VOCAB;
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
    "", "bir", "iki", "üç", "dört", "beş", "altı", "yedi", "sekiz", "dokuz",
];
const TEENS: [&str; 10] = [
    "on", "on bir", "on iki", "on üç", "on dört", "on beş", "on altı", "on yedi", "on sekiz",
    "on dokuz",
];
const TENS: [&str; 10] = [
    "", "", "yirmi", "otuz", "kırk", "elli", "altmış", "yetmiş", "seksen", "doksan",
];

const HUNDRED: &str = "yüz";
const THOUSAND: &str = "bin";

const ZERO: &str = "sıfır";
const NEGATIVE: &str = "eksi";
const DECIMAL_SEP: &str = "virgül";

/// Short scale from a million up; units and thousands are handled apart.
const SCALES: &[&str] = &["milyon", "milyar", "trilyon", "katrilyon", "kentilyon"];

/// The cardinal ceiling, derived from the scale table: `10^21`. Segments are
/// units, thousands, then one per [`SCALES`] entry.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32 + 1);
/// The ordinal ceiling: `10^21` (the ordinal is the cardinal plus a suffix).
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32 + 1);
/// The currency ceiling: `10^21`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32 + 1);

// Turkish ordinals take an -(i/ı/u/ü)nci suffix with vowel harmony; 1-10 are
// irregular (index 0 unused).
const ORDINAL_SPECIAL: [&str; 11] = [
    "",
    "birinci",
    "ikinci",
    "üçüncü",
    "dördüncü",
    "beşinci",
    "altıncı",
    "yedinci",
    "sekizinci",
    "dokuzuncu",
    "onuncu",
];

/// The word separator: nothing in the compound form, a space otherwise.
const fn separator(drop_spaces: bool) -> &'static str {
    if drop_spaces { "" } else { " " }
}

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment, joined by `sep`; "bir" is omitted before "yüz".
fn build_segment(n: u32, sep: &str) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<String> = Vec::new();
    // Hundreds: omit "bir" before yüz
    if hundreds > 0 {
        parts.push(if hundreds == 1 {
            HUNDRED.to_string()
        } else {
            format!("{}{sep}{HUNDRED}", ONES[hundreds])
        });
    }
    // Tens and ones
    let tens_ones = n % 100;
    if tens_ones > 0 {
        parts.push(if tens_ones < 10 {
            ONES[ones].to_string()
        } else if tens_ones < 20 {
            TEENS[ones].replace(' ', sep)
        } else if ones == 0 {
            TENS[tens].to_string()
        } else {
            format!("{}{sep}{}", TENS[tens], ONES[ones])
        });
    }
    parts.join(sep)
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint, drop_spaces: bool) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }
    let sep = separator(drop_spaces);

    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32, sep);
        }
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;
            // Omit "bir" before bin (thousand)
            let mut result = if thousands == 1 {
                THOUSAND.to_string()
            } else {
                format!("{}{sep}{THOUSAND}", build_segment(thousands, sep))
            };
            if remainder > 0 {
                result.push_str(sep);
                result.push_str(&build_segment(remainder, sep));
            }
            return result;
        }
    }

    build_large_number_words(n, sep)
}

/// Words for a number of one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint, sep: &str) -> String {
    let segments = split_groups(n, 1000);
    let mut parts: Vec<String> = Vec::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let word = build_segment(segment, sep);
        parts.push(match i {
            // Units segment
            0 => word,
            // Thousands: omit "bir" before bin
            1 if segment == 1 => THOUSAND.to_string(),
            1 => format!("{word}{sep}{THOUSAND}"),
            // Millions and above keep "bir" before the scale word
            _ => format!("{word}{sep}{}", SCALES[i - 2]),
        });
    }
    parts.join(sep)
}

/// Words for the digits after the decimal separator: leading zeros one by
/// one, then the rest as a single number.
fn decimal_part_to_words(decimal: &str, drop_spaces: bool) -> String {
    let sep = separator(drop_spaces);
    let mut result = String::new();
    let zeros = decimal.bytes().take_while(|&b| b == b'0').count();
    for _ in 0..zeros {
        if !result.is_empty() {
            result.push_str(sep);
        }
        result.push_str(ZERO);
    }
    let remainder = &decimal[zeros..];
    if !remainder.is_empty() {
        if !result.is_empty() {
            result.push_str(sep);
        }
        result.push_str(&integer_to_words(
            &digits_to_biguint(remainder),
            drop_spaces,
        ));
    }
    result
}

options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Remove spaces for compound form
        drop_spaces: bool = false;
    }
}

/// Spells a number in Turkish words.
///
/// ```
/// # use n2words::tr_tr::to_cardinal;
/// assert_eq!(to_cardinal(1000)?, "bin");
/// assert_eq!(to_cardinal(-17.42)?, "eksi on yedi virgül kırk iki");
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

/// Spells a number in Turkish words, with options.
///
/// ```
/// # use n2words::tr_tr::{to_cardinal_with, CardinalOptions};
/// let compound = CardinalOptions { drop_spaces: true };
/// assert_eq!(to_cardinal_with(1243, &compound)?, "binikiyüzkırküç");
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

    let sep = separator(options.drop_spaces);
    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
        result.push_str(sep);
    }
    result.push_str(&integer_to_words(&number.integer, options.drop_spaces));
    if let Some(fraction) = &number.fraction {
        result.push_str(sep);
        result.push_str(DECIMAL_SEP);
        result.push_str(sep);
        result.push_str(&decimal_part_to_words(fraction, options.drop_spaces));
    }
    Ok(result)
}

// ============================================================================
// Ordinal
// ============================================================================

/// The ordinal suffix by Turkish vowel harmony: the last vowel of the word
/// decides between back (a, ı, o, u) and front (e, i, ö, ü) forms.
fn ordinal_suffix(word: &str) -> &'static str {
    for c in word.chars().rev() {
        match c {
            'o' | 'u' => return "uncu",
            'a' | 'ı' => return "ıncı",
            'ö' | 'ü' => return "üncü",
            'e' | 'i' => return "inci",
            _ => {}
        }
    }
    "inci"
}

/// Ordinal words for a positive integer: irregular forms for 1-10, otherwise
/// the compound cardinal plus a harmonised suffix.
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if (1..=10).contains(&small) {
            return ORDINAL_SPECIAL[small as usize].to_string();
        }
    }
    let mut cardinal = integer_to_words(n, true);
    cardinal.push_str(ordinal_suffix(&cardinal));
    cardinal
}

/// Spells a positive integer as a Turkish ordinal.
///
/// ```
/// # use n2words::tr_tr::to_ordinal;
/// assert_eq!(to_ordinal(3)?, "üçüncü");
/// assert_eq!(to_ordinal(100)?, "yüzüncü");
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
        currency: Currency = Currency::TRY, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Turkish lira (100 kuruş = 1 lira).
///
/// ```
/// # use n2words::tr_tr::to_currency;
/// assert_eq!(to_currency(42)?, "kırk iki lira");
/// assert_eq!(to_currency(-5)?, "eksi beş lira");
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

/// Spells an amount in any currency Turkish has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Turkish has no words for and
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
    // Lira part: the nouns are invariable, so the first form serves every count.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major, false));
        result.push(' ');
        result.push_str(words.major_at(0));
    }
    // Kuruş part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor), false));
        result.push(' ');
        result.push_str(words.minor_at(0));
    }
    Ok(result)
}

/// Registry entry for `tr-TR`.
pub static LANGUAGE: Language = Language {
    code: "tr-TR",
    name: "Turkish (Türkiye)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
