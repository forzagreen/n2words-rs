//! Thai (Thailand) (`th-TH`).
//!
//! - No word separators (continuous Thai script)
//! - Million-based grouping (ล้าน), repeated for each further power of a
//!   million, so there is no fixed ceiling
//! - 1 is "เอ็ด" in compounds (11, 21, 101, ...)
//! - 20 is "ยี่สิบ" (not "สองสิบ")
//! - Per-digit decimal reading
//!
//! ```
//! use n2words::th_th::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(12345678)?, "สิบสองล้านสามแสนสี่หมื่นห้าพันหกร้อยเจ็ดสิบแปด");
//! assert_eq!(to_ordinal(21)?, "ที่ยี่สิบเอ็ด");
//! assert_eq!(to_currency(42.5)?, "สี่สิบสองบาทห้าสิบสตางค์");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::TH as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, parse_cardinal, parse_currency, parse_ordinal, split_groups};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::Zero;

// Thai spells large numbers by repeating ล้าน (10^6), so every form is
// magnitude-preserving with no fixed ceiling.

/// The cardinal ceiling: none, ล้าน repeats for every further power of a
/// million.
pub const CARDINAL_MAX: Max = Max::Unbounded;
/// The ordinal ceiling: none.
pub const ORDINAL_MAX: Max = Max::Unbounded;
/// The currency ceiling: none.
pub const CURRENCY_MAX: Max = Max::Unbounded;

// ============================================================================
// Vocabulary
// ============================================================================

// Digit words for 1-9 (index `digit - 1`)
const ONES: [&str; 9] = ["หนึ่ง", "สอง", "สาม", "สี่", "ห้า", "หก", "เจ็ด", "แปด", "เก้า"];

const ZERO: &str = "ศูนย์";
const NEGATIVE: &str = "ลบ";
const DECIMAL_SEP: &str = "จุด";

const MILLION: &str = "ล้าน";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

const ORDINAL_PREFIX: &str = "ที่";

// ============================================================================
// Currency Vocabulary (Thai Baht)
// ============================================================================

const BAHT_ONLY: &str = "ถ้วน"; // "exactly" suffix when no satang; grammar, not a currency name

// ============================================================================
// Conversion
// ============================================================================

/// Words for a value below one million.
fn convert_below_million(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let mut value = n;
    let hundred_thousands = (value / 100_000) as usize;
    value %= 100_000;
    let ten_thousands = (value / 10_000) as usize;
    value %= 10_000;
    let thousands = (value / 1000) as usize;
    value %= 1000;
    let hundreds = (value / 100) as usize;
    value %= 100;
    let tens = (value / 10) as usize;
    let ones = (value % 10) as usize;

    let mut parts = String::new();

    if hundred_thousands > 0 {
        parts.push_str(ONES[hundred_thousands - 1]);
        parts.push_str("แสน");
    }

    if ten_thousands > 0 {
        if ten_thousands == 1 {
            parts.push_str("หนึ่งหมื่น");
        } else {
            parts.push_str(ONES[ten_thousands - 1]);
            parts.push_str("หมื่น");
        }
    }

    if thousands > 0 {
        parts.push_str(ONES[thousands - 1]);
        parts.push_str("พัน");
    }

    if hundreds > 0 {
        parts.push_str(ONES[hundreds - 1]);
        parts.push_str("ร้อย");
    }

    if tens > 0 {
        match tens {
            1 => parts.push_str("สิบ"),
            2 => parts.push_str("ยี่สิบ"),
            _ => {
                parts.push_str(ONES[tens - 1]);
                parts.push_str("สิบ");
            }
        }
    }

    if ones > 0 {
        let has_higher =
            hundred_thousands > 0 || ten_thousands > 0 || thousands > 0 || hundreds > 0 || tens > 0;
        if ones == 1 && (tens > 0 || has_higher) {
            parts.push_str("เอ็ด");
        } else {
            parts.push_str(ONES[ones - 1]);
        }
    }

    parts
}

/// Words for a non-negative integer, by million-based groups.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Groups least-significant first; group `i` is followed by ล้าน `i` times.
    let groups = split_groups(n, 1_000_000);
    let mut parts = String::new();
    for (i, &group) in groups.iter().enumerate().rev() {
        if group == 0 {
            continue;
        }
        parts.push_str(&convert_below_million(group));
        if i > 0 {
            parts.push_str(&MILLION.repeat(i));
        }
    }
    parts
}

/// Words for the digits after the decimal point, read digit by digit.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut digits = String::new();
    for digit in decimal.bytes() {
        let d = usize::from(digit - b'0');
        if d == 0 {
            digits.push_str(ZERO);
        } else {
            digits.push_str(ONES[d - 1]);
        }
    }
    digits
}

/// Spells a number in Thai words.
///
/// ```
/// # use n2words::th_th::to_cardinal;
/// assert_eq!(to_cardinal(1000000000)?, "หนึ่งพันล้าน");
/// assert_eq!(to_cardinal(3.14)?, "สามจุดหนึ่งสี่");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value. There is no ceiling.
pub fn to_cardinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    let number = parse_cardinal(&value.into())?;
    // Unbounded: the check is a no-op, kept so every form reads the same.
    CARDINAL_MAX.check(&number.integer, None)?;

    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
    }
    result.push_str(&integer_to_words(&number.integer));
    if let Some(fraction) = &number.fraction {
        result.push_str(DECIMAL_SEP);
        result.push_str(&decimal_part_to_words(fraction));
    }
    Ok(result)
}

// ============================================================================
// Ordinal
// ============================================================================

/// Ordinal words for a positive integer: the ที่ prefix plus the cardinal.
fn integer_to_ordinal(n: &BigUint) -> String {
    format!("{ORDINAL_PREFIX}{}", integer_to_words(n))
}

/// Spells a positive integer as a Thai ordinal (ที่ + cardinal).
///
/// ```
/// # use n2words::th_th::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "ที่หนึ่ง");
/// assert_eq!(to_ordinal(101)?, "ที่หนึ่งร้อยเอ็ด");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidOrdinal`](crate::Error::InvalidOrdinal) for zero, a
/// negative or a fraction.
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
        currency: Currency = Currency::THB, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Thai baht.
///
/// The baht has the satang as subunit (100 satang = 1 baht); a whole amount
/// takes the "ถ้วน" (exactly) suffix.
///
/// ```
/// # use n2words::th_th::to_currency;
/// assert_eq!(to_currency(42)?, "สี่สิบสองบาทถ้วน");
/// assert_eq!(to_currency(1.01)?, "หนึ่งบาทหนึ่งสตางค์");
/// assert_eq!(to_currency(-5)?, "ลบห้าบาทถ้วน");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value.
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency Thai has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Thai has no words for and
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
    }

    // Baht part - always shown
    result.push_str(&integer_to_words(&money.major));
    result.push_str(words.major_at(0));

    // Satang part, or the "exactly" suffix
    if money.minor > 0 {
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push_str(words.minor_at(0));
    } else {
        result.push_str(BAHT_ONLY);
    }

    Ok(result)
}

/// Registry entry for `th-TH`.
pub static LANGUAGE: Language = Language {
    code: "th-TH",
    name: "Thai (Thailand)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
