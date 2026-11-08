//! Persian (Iran) (`fa-IR`): Persian as used in Iran.
//!
//! - "و" (and) conjunction for compound numbers
//! - Omit "یک" (one) before thousand
//! - Pre-composed hundreds (دویست, سيصد, etc.)
//! - No fixed scale ceiling: the speller composes every magnitude from
//!   هزار, میلیون and میلیارد
//!
//! ```
//! use n2words::fa_ir::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "چهل و دو");
//! assert_eq!(to_ordinal(42)?, "چهل و دوم");
//! assert_eq!(to_currency(1000)?, "هزار ریال");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::FA as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

/// The cardinal ceiling: none, the speller composes every magnitude.
pub const CARDINAL_MAX: Max = Max::Unbounded;
/// The ordinal ceiling: none.
pub const ORDINAL_MAX: Max = Max::Unbounded;
/// The currency ceiling: none.
pub const CURRENCY_MAX: Max = Max::Unbounded;

// ============================================================================
// Vocabulary
// ============================================================================

const ONES: [&str; 10] = [
    "", "یک", "دو", "سه", "چهار", "پنج", "شش", "هفت", "هشت", "نه",
];
/// 10–19, indexed by `n - 10`.
const TEENS: [&str; 10] = [
    "ده",
    "یازده",
    "دوازده",
    "سیزده",
    "چهارده",
    "پانزده",
    "شانزده",
    "هفده",
    "هجده",
    "نوزده",
];
/// 20–90, indexed by the tens digit.
const TENS: [&str; 10] = [
    "",
    "",
    "بیست",
    "سی",
    "چهل",
    "پنجاه",
    "شصت",
    "هفتاد",
    "هشتاد",
    "نود",
];
/// 100–900, indexed by the hundreds digit.
const HUNDREDS: [&str; 10] = [
    "",
    "صد",
    "دویست",
    "سيصد",
    "چهار صد",
    "پانصد",
    "ششصد",
    "هفتصد",
    "هشتصد",
    "نهصد",
];

const THOUSAND: &str = "هزار";
const MILLION: &str = "میلیون";
const MILLIARD: &str = "میلیارد";

const ZERO: &str = "صفر";
const NEGATIVE: &str = "منفى";
const DECIMAL_SEP: &str = "ممیّز";

// ============================================================================
// Ordinal vocabulary
// ============================================================================

// Persian ordinals: add -ُم (-om) suffix to the cardinal, with special forms
// for 1st, 2nd and 3rd.
const ORDINAL_SUFFIX: &str = "م";
const ORDINAL_ONES: [&str; 10] = [
    "",
    "اول", // avval (first)
    "دوم", // dovvom (second)
    "سوم", // sevvom (third)
    "چهارم",
    "پنجم",
    "ششم",
    "هفتم",
    "هشتم",
    "نهم",
];

// ============================================================================
// Conversion
// ============================================================================

const BILLION: u64 = 1_000_000_000;

/// Words for a non-negative integer below one milliard (10^9).
fn small_to_words(n: u64) -> String {
    if n == 0 {
        return ZERO.to_string();
    }

    // 1-9
    if n <= 9 {
        return ONES[n as usize].to_string();
    }

    // 10-19
    if n <= 19 {
        return TEENS[(n - 10) as usize].to_string();
    }

    // 20-99
    if n < 100 {
        let ones = (n % 10) as usize;
        let tens = (n / 10) as usize;
        if ones == 0 {
            return TENS[tens].to_string();
        }
        return format!("{} و {}", TENS[tens], ONES[ones]);
    }

    // 100-999
    if n < 1000 {
        let hundreds = (n / 100) as usize;
        let remainder = n % 100;
        if remainder == 0 {
            return HUNDREDS[hundreds].to_string();
        }
        return format!("{} و {}", HUNDREDS[hundreds], small_to_words(remainder));
    }

    // 1000-999999
    if n < 1_000_000 {
        let multiplier = n / 1000;
        // Persian omits "one" before thousand: 1000 is just "هزار", not "یک هزار"
        let prefix = if multiplier == 1 {
            String::new()
        } else {
            format!("{} ", small_to_words(multiplier))
        };
        let remainder = n % 1000;
        let suffix = if remainder == 0 {
            String::new()
        } else {
            format!(" {}", small_to_words(remainder))
        };
        return format!("{prefix}{THOUSAND}{suffix}");
    }

    // 1000000-999999999 (millions)
    let multiplier = n / 1_000_000;
    let remainder = n % 1_000_000;
    let mut result = format!("{} {MILLION}", small_to_words(multiplier));
    if remainder != 0 {
        result.push_str(" و ");
        result.push_str(&small_to_words(remainder));
    }
    result
}

/// Converts a non-negative integer to Persian cardinal words. From one
/// milliard (10^9) up the number is "<multiplier> میلیارد و <remainder>",
/// the multiplier itself spelled recursively, so there is no ceiling.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }
    if let Some(small) = n.to_u64() {
        if small < BILLION {
            return small_to_words(small);
        }
    }

    // 1000000000+ (milliards - 10^9)
    let billion = BigUint::from(BILLION);
    let multiplier = n / &billion;
    let remainder = n % &billion;
    let mut result = format!("{} {MILLIARD}", integer_to_words(&multiplier));
    if !remainder.is_zero() {
        result.push_str(" و ");
        result.push_str(&integer_to_words(&remainder));
    }
    result
}

/// Converts the fractional digits of a number to Persian words: leading
/// zeros one by one, then the rest as a single number.
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

// ============================================================================
// Cardinal
// ============================================================================

/// Spells a number in Persian words.
///
/// ```
/// # use n2words::fa_ir::to_cardinal;
/// assert_eq!(to_cardinal(1001)?, "هزار یک");
/// assert_eq!(to_cardinal(-17.42)?, "منفى هفده ممیّز چهل و دو");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value.
pub fn to_cardinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    let number = parse_cardinal(&value.into())?;

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

/// Persian ordinals: اول (1st), دوم (2nd), سوم (3rd), then cardinal + م suffix.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special forms for 1-9
    if let Some(small) = n.to_u64() {
        if (1..=9).contains(&small) {
            return ORDINAL_ONES[small as usize].to_string();
        }
    }
    // For 10+, add -م suffix to cardinal
    format!("{}{ORDINAL_SUFFIX}", integer_to_words(n))
}

/// Spells a positive integer as a Persian ordinal.
///
/// ```
/// # use n2words::fa_ir::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "اول");
/// assert_eq!(to_ordinal(2)?, "دوم");
/// assert_eq!(to_ordinal(10)?, "دهم");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidOrdinal`](crate::Error::InvalidOrdinal) for zero, a
/// negative or a fraction.
pub fn to_ordinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    let n = parse_ordinal(&value.into())?;
    Ok(integer_to_ordinal(&n))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::IRR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Iranian rials.
///
/// The rial has no everyday minor unit (historically the dinar was 1/100
/// rial, but it is not used today), so a fractional amount is an error
/// rather than being silently dropped.
///
/// ```
/// # use n2words::fa_ir::to_currency;
/// assert_eq!(to_currency(42)?, "چهل و دو ریال");
/// assert_eq!(to_currency(-5)?, "منفى پنج ریال");
/// assert!(to_currency(1.5).is_err());
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value, [`Error::NoMinorUnit`](crate::Error::NoMinorUnit) for a fractional
/// amount.
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency Persian has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Persian has no words for.
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

/// Registry entry for `fa-IR`.
pub static LANGUAGE: Language = Language {
    code: "fa-IR",
    name: "Persian (Iran)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
