//! Vietnamese (Vietnam) (`vi-VN`).
//!
//! - Special pronunciation: "lăm" for 5 in the ones place after a tens word,
//!   "mốt" for a final 1 after twenty and up
//! - "Lẻ" (odd/extra) marker when the tens place is zero after hundreds or a
//!   scale word
//! - Short scale with Vietnamese words (nghìn, triệu, tỷ), compounding past
//!   tỷ: nghìn tỷ, triệu tỷ, tỷ tỷ
//!
//! ```
//! use n2words::vi_vn::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(101)?, "một trăm lẻ một");
//! assert_eq!(to_ordinal(1)?, "thứ nhất");
//! assert_eq!(to_currency(1001)?, "một nghìn lẻ một đồng");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::VI as CURRENCY_VOCAB;
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

// Base vocabulary for building lookup tables
const ONES: [&str; 10] = [
    "không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín",
];

// Scale words indexed by scale level (0 = units, 1 = thousands, etc.)
// Vietnamese composes large numbers by cycling nghìn/triệu/tỷ and appending
// another tỷ every three groups. The recursion past tỷ tỷ (10^18) is rarely
// used and not firmly fixed, so the table stops at that scale word rather
// than invent more; the ceiling then falls where the next one would be needed.
const SCALES: &[&str] = &["", "nghìn", "triệu", "tỷ", "nghìn tỷ", "triệu tỷ", "tỷ tỷ"];

/// The cardinal ceiling, derived from the scale table: `10^21`. 3-digit
/// grouping with `SCALES[0]` the (empty) units slot; the highest scale word is
/// tỷ tỷ at index 6 (10^18), so values from `10^(3 * SCALES.len())` up have
/// no scale word.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32 - 1);
/// The ordinal ceiling: `10^21` (ordinals are built from the cardinal speller).
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32 - 1);
/// The currency ceiling: `10^21`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32 - 1);

const HUNDRED: &str = "trăm";
const ZERO: &str = "không";
const NEGATIVE: &str = "âm";
const DECIMAL_SEP: &str = "phẩy";
const LE: &str = "lẻ"; // "odd/extra" marker for gaps

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

const ORDINAL_PREFIX: &str = "thứ";
// First is special: "thứ nhất" (not "thứ một")
const ORDINAL_ONE: &str = "nhất";

// Special forms
const MOT_FINAL: &str = "mốt"; // 1 in the ones place after a tens word (21, 31, ...)
const LAM: &str = "lăm"; // 5 in the ones place after a tens word (15, 25, ...)

// ============================================================================
// Segment Building
// ============================================================================

/// Words for 0-99 with the special forms (mốt, lăm).
fn build_below_hundred(n: u32) -> String {
    if n == 0 {
        return ONES[0].to_string();
    }
    if n < 10 {
        return ONES[n as usize].to_string();
    }

    // Teens: 10-19
    if n < 20 {
        let ones = (n - 10) as usize;
        return match ones {
            0 => "mười".to_string(),
            5 => "mười lăm".to_string(),
            _ => format!("mười {}", ONES[ones]),
        };
    }

    // 20-99
    let ones = (n % 10) as usize;
    let tens = (n / 10) as usize;
    let tens_word = format!("{} mươi", ONES[tens]);

    match ones {
        0 => tens_word,
        1 => format!("{tens_word} {MOT_FINAL}"),
        5 => format!("{tens_word} {LAM}"),
        _ => format!("{tens_word} {}", ONES[ones]),
    }
}

/// The ones word after "lẻ": "năm", not "lăm", for 5.
fn after_le(n: u32) -> &'static str {
    if n == 5 { "năm" } else { ONES[n as usize] }
}

/// Words for a 0-999 segment.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let hundreds = (n / 100) as usize;
    let remainder = n % 100;

    let mut result = String::new();
    if hundreds > 0 {
        result = format!("{} {HUNDRED}", ONES[hundreds]);
    }

    if remainder > 0 {
        if remainder < 10 {
            // A single digit after hundreds needs "lẻ"
            if result.is_empty() {
                result = ONES[remainder as usize].to_string();
            } else {
                result.push(' ');
                result.push_str(LE);
                result.push(' ');
                result.push_str(after_le(remainder));
            }
        } else {
            // 10-99 after hundreds
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&build_below_hundred(remainder));
        }
    }

    result
}

/// "lẻ"-prefixed words for a small remainder (1-99) after a scale word.
fn build_le_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    if n < 10 {
        return format!("{LE} {}", after_le(n));
    }
    format!("{LE} {}", build_below_hundred(n))
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
        // Fast path: numbers < 100
        if small < 100 {
            return build_below_hundred(small as u32);
        }

        // Fast path: numbers < 1000
        if small < 1000 {
            return build_segment(small as u32);
        }

        // Fast path: numbers < 1,000,000 (thousands)
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            let mut result = format!("{} {}", build_segment(thousands), SCALES[1]);
            if remainder == 0 {
                return result;
            }
            result.push(' ');
            // A remainder below 100 needs the "lẻ" marker
            if remainder < 100 {
                result.push_str(&build_le_segment(remainder));
            } else {
                result.push_str(&build_segment(remainder));
            }
            return result;
        }
    }

    // For numbers >= 1,000,000, use scale decomposition
    build_large_number_words(n)
}

/// Words for a number of 1,000,000 or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    // Segments least-significant first (index 0 = units, 1 = nghìn, ...)
    let segments = split_groups(n, 1000);

    let mut parts: Vec<String> = Vec::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let words = build_segment(segment);
        if i > 0 {
            parts.push(format!("{words} {}", SCALES[i]));
        } else {
            parts.push(words);
        }
    }

    if parts.is_empty() {
        return ZERO.to_string();
    }
    if parts.len() == 1 {
        return parts.remove(0);
    }

    // A small final segment (1-99, no hundreds) takes "lẻ" after the scale word
    let last_segment = segments[0];
    if (1..=99).contains(&last_segment) {
        parts.pop();
        parts.push(build_le_segment(last_segment));
    }
    parts.join(" ")
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    let zeros = decimal.bytes().take_while(|&b| b == b'0').count();
    for _ in 0..zeros {
        parts.push(ZERO.to_string());
    }
    let remainder = &decimal[zeros..];
    if !remainder.is_empty() {
        parts.push(integer_to_words(&digits_to_biguint(remainder)));
    }
    parts.join(" ")
}

/// Spells a number in Vietnamese words.
///
/// ```
/// # use n2words::vi_vn::to_cardinal;
/// assert_eq!(to_cardinal(1000001)?, "một triệu lẻ một");
/// assert_eq!(to_cardinal(-17.42)?, "âm mười bảy phẩy bốn mươi hai");
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

/// Ordinal words for a positive integer: "thứ" + cardinal, except "thứ nhất"
/// for 1 (not "thứ một").
fn integer_to_ordinal(n: &BigUint) -> String {
    if n.is_one() {
        return format!("{ORDINAL_PREFIX} {ORDINAL_ONE}");
    }
    format!("{ORDINAL_PREFIX} {}", integer_to_words(n))
}

/// Spells a positive integer as a Vietnamese ordinal.
///
/// ```
/// # use n2words::vi_vn::to_ordinal;
/// assert_eq!(to_ordinal(2)?, "thứ hai");
/// assert_eq!(to_ordinal(21)?, "thứ hai mươi mốt");
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
    // Ordinals are built from the cardinal speller, so they share its ceiling.
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
        currency: Currency = Currency::VND, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Vietnamese đồng.
///
/// The đồng has no everyday subunit (xu are historical): a fractional amount
/// is an error rather than being silently rounded away.
///
/// ```
/// # use n2words::vi_vn::to_currency;
/// assert_eq!(to_currency(21)?, "hai mươi mốt đồng");
/// assert_eq!(to_currency(-1000)?, "âm một nghìn đồng");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value, [`Error::OutOfRange`](crate::Error::OutOfRange) at or past
/// [`CURRENCY_MAX`], [`Error::NoMinorUnit`](crate::Error::NoMinorUnit) for a
/// fractional amount.
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency Vietnamese has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Vietnamese has no words for.
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
    result.push_str(&integer_to_words(&money.major));
    result.push(' ');
    result.push_str(words.major_at(0));
    Ok(result)
}

/// Registry entry for `vi-VN`.
pub static LANGUAGE: Language = Language {
    code: "vi-VN",
    name: "Vietnamese (Vietnam)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
