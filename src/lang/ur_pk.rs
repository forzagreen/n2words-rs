//! Urdu (Pakistan) (`ur-PK`).
//!
//! - Indian numbering system (ہزار, لاکھ, کروڑ)
//! - Urdu script (right-to-left)
//! - 3-2-2 grouping pattern (last 3 digits, then groups of 2)
//! - Complete word forms for 0-99
//!
//! ```
//! use n2words::ur_pk::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(215)?, "دو سو پندرہ");
//! assert_eq!(to_ordinal(10)?, "دسواں");
//! assert_eq!(to_currency(42.5)?, "بیالیس روپے پچاس پیسے");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::UR as CURRENCY_VOCAB;
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

const ZERO: &str = "صفر";
const NEGATIVE: &str = "منفی";
const DECIMAL_SEP: &str = "اعشاریہ";
const HUNDRED: &str = "سو";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

/// Ordinal suffix (added to the cardinal).
const ORDINAL_SUFFIX: &str = "واں";

/// Special ordinals for the first few numbers.
const ORDINAL_SPECIAL: [&str; 7] = ["", "پہلا", "دوسرا", "تیسرا", "چوتھا", "پانچواں", "چھٹا"];

#[rustfmt::skip]
const BELOW_HUNDRED: [&str; 100] = [
    "صفر", "ایک", "دو", "تین", "چار", "پانچ", "چھ", "سات", "آٹھ", "نو",
    "دس", "گیارہ", "بارہ", "تیرہ", "چودہ", "پندرہ", "سولہ", "سترہ", "اٹھارہ", "انیس",
    "بیس", "اکیس", "بائیس", "تیئیس", "چوبیس", "پچیس", "چھبیس", "ستائیس", "اٹھائیس", "انتیس",
    "تیس", "اکتیس", "بتیس", "تینتیس", "چونتیس", "پینتیس", "چھتیس", "سینتیس", "اڑتیس", "انتالیس",
    "چالیس", "اکتالیس", "بیالیس", "تینتالیس", "چوالیس", "پینتالیس", "چھالیس", "سینتالیس", "اڑتالیس", "انچاس",
    "پچاس", "اکاون", "باون", "ترپن", "چون", "پچپن", "چھپن", "ستاون", "اٹھاون", "انسٹھ",
    "ساٹھ", "اکسٹھ", "باسٹھ", "ترسٹھ", "چونسٹھ", "پینسٹھ", "چھیاسٹھ", "سڑسٹھ", "اڑسٹھ", "انہتر",
    "ستر", "اکہتر", "بہتر", "تہتر", "چوہتر", "پچھتر", "چھہتر", "ستتر", "اٹھہتر", "اناسی",
    "اسی", "اکیاسی", "بیاسی", "تریاسی", "چوراسی", "پچاسی", "چھیاسی", "ستاسی", "اٹھاسی", "نواسی",
    "نوے", "اکانوے", "بانوے", "ترانوے", "چورانوے", "پچانوے", "چھیانوے", "ستانوے", "اٹھانوے", "ننانوے",
];

/// Scale words: index 0 = units (empty), 1 = thousand, 2 = lakh, 3 = crore, etc.
const SCALE_WORDS: &[&str] = &[
    "", "ہزار", "لاکھ", "کروڑ", "ارب", "کھرب", "نیل", "پدم", "شنکھ",
];

// 3-2-2 Indian grouping: a 3-digit base segment, then 2 digits per scale word
// (SCALE_WORDS[0] = "" is the units slot). Past the table the scale word is
// dropped, which collapses the magnitude — so cap there.

/// The cardinal ceiling, derived from the scale table: `10^19`.
pub const CARDINAL_MAX: Max = Max::indian(SCALE_WORDS.len() as u32);
/// The ordinal ceiling: `10^19`.
pub const ORDINAL_MAX: Max = Max::indian(SCALE_WORDS.len() as u32);
/// The currency ceiling: `10^19`.
pub const CURRENCY_MAX: Max = Max::indian(SCALE_WORDS.len() as u32);

// ============================================================================
// Segment Building
// ============================================================================

/// Words for a 0-999 segment.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    if n < 100 {
        return BELOW_HUNDRED[n as usize].to_string();
    }

    let hundreds = (n / 100) as usize;
    let remainder = (n % 100) as usize;

    if remainder == 0 {
        return format!("{} {HUNDRED}", BELOW_HUNDRED[hundreds]);
    }
    format!(
        "{} {HUNDRED} {}",
        BELOW_HUNDRED[hundreds], BELOW_HUNDRED[remainder]
    )
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Words for a non-negative integer.
///
/// South Asian 3-2-2 grouping: first 3 digits, then groups of 2.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Fast path: numbers < 1000 (direct lookup).
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32);
        }
    }

    // Segments least-significant first: the 3-digit base, then 2 digits each.
    let segments = split_groups_indian(n, 1000, 100);

    // Build the result from most-significant to least.
    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if !result.is_empty() {
            result.push(' ');
        }

        if i == 0 {
            result.push_str(&build_segment(segment));
        } else {
            result.push_str(BELOW_HUNDRED[segment as usize]);
        }

        if i > 0 {
            if let Some(scale) = SCALE_WORDS.get(i).filter(|s| !s.is_empty()) {
                result.push(' ');
                result.push_str(scale);
            }
        }
    }

    result
}

/// Words for the digits after the decimal separator: leading zeros one by
/// one, then the rest as a single number.
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

/// Spells a number in Urdu words.
///
/// ```
/// # use n2words::ur_pk::to_cardinal;
/// assert_eq!(to_cardinal(1000000)?, "دس لاکھ");
/// assert_eq!(to_cardinal(-17)?, "منفی سترہ");
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
    // Both the integer part and the decimal's significant digits are spelled
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

/// Ordinal words for a positive integer.
///
/// Urdu ordinals: the first 6 are irregular, then add the -واں suffix.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special ordinals for 1-6.
    if let Some(small) = n.to_u64() {
        if (1..=6).contains(&small) {
            return ORDINAL_SPECIAL[small as usize].to_string();
        }
    }

    // For 7 and above, add the suffix to the cardinal.
    let mut result = integer_to_words(n);
    result.push_str(ORDINAL_SUFFIX);
    result
}

/// Spells a positive integer as an Urdu ordinal.
///
/// ```
/// # use n2words::ur_pk::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "پہلا");
/// assert_eq!(to_ordinal(3)?, "تیسرا");
/// assert_eq!(to_ordinal(1001)?, "ایک ہزار ایکواں");
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
    // Ordinals build on the cardinal speller, so they share its ceiling.
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
        currency: Currency = Currency::PKR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Pakistani rupees.
///
/// ```
/// # use n2words::ur_pk::to_currency;
/// assert_eq!(to_currency(1)?, "ایک روپیہ");
/// assert_eq!(to_currency(0.01)?, "ایک پیسہ");
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

/// Spells an amount in any currency Urdu has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Urdu has no words for and
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

    // Rupees part: shown if non-zero, or if there are no paise.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        // Singular for 1 rupee, plural otherwise.
        result.push_str(words.major_at(if money.major.is_one() { 0 } else { 1 }));
    }

    // Paise part.
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        // Singular for 1 paisa, plural otherwise.
        result.push_str(words.minor_at(if money.minor == 1 { 0 } else { 1 }));
    }

    Ok(result)
}

/// Registry entry for `ur-PK`.
pub static LANGUAGE: Language = Language {
    code: "ur-PK",
    name: "Urdu (Pakistan)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
