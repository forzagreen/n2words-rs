//! Simplified Chinese (China) (`zh-Hans-CN`).
//!
//! - Myriad-based (万, 亿) grouping: 4 digits per segment
//! - Formal (financial) vs common numerals, through the `formal` option
//! - Zero insertion for skipped positions
//! - No word separators (concatenated format)
//!
//! ```
//! use n2words::zh_hans_cn::{to_cardinal, to_cardinal_with, to_currency, to_ordinal, CardinalOptions};
//!
//! assert_eq!(to_cardinal(17.42)?, "壹拾柒点肆贰");
//! assert_eq!(to_cardinal_with(2000000, &CardinalOptions { formal: false })?, "二百万");
//! assert_eq!(to_ordinal(21)?, "第贰拾壹");
//! assert_eq!(to_currency(42.5)?, "肆拾贰圆伍角整");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::currency::vocab::ZH_HANS as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, parse_cardinal, parse_currency, parse_ordinal};
use crate::options::options;
use crate::{Error, Result};
use num_bigint::BigUint;
use num_traits::ToPrimitive;

// ============================================================================
// Vocabulary
// ============================================================================

/// One numeral set: the digit words and the place words within a 4-digit
/// segment.
struct Numerals {
    ones: [&'static str; 10],
    ten: &'static str,
    hundred: &'static str,
    thousand: &'static str,
}

// Common (everyday) numerals
const COMMON: Numerals = Numerals {
    ones: ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"],
    ten: "十",
    hundred: "百",
    thousand: "千",
};

// Formal (financial) numerals - harder to alter/forge
const FORMAL: Numerals = Numerals {
    ones: ["零", "壹", "贰", "叁", "肆", "伍", "陆", "柒", "捌", "玖"],
    ten: "拾",
    hundred: "佰",
    thousand: "仟",
};

/// The numeral set the `formal` option selects.
const fn numerals(formal: bool) -> &'static Numerals {
    if formal { &FORMAL } else { &COMMON }
}

// Scale words
const WAN_WORD: &str = "万"; // 10,000
const YI_WORD: &str = "亿"; // 100,000,000

// Supported magnitude ceiling (checked at the public entry points). Numbers
// >= 亿 (10^8) split into n / 10^8, which is then spelled by convert_below_yi,
// itself only valid below 亿. So the ceiling is 亿² = 10^16. Ordinal (第 +
// cardinal) and currency build on the cardinal, so they share it. Decimals are
// spelled digit by digit, so they have no ceiling.

/// The cardinal ceiling: `10^16` (亿², the bound of the 亿 recursion).
pub const CARDINAL_MAX: Max = Max::bounded(16);
/// The ordinal ceiling: `10^16`.
pub const ORDINAL_MAX: Max = Max::bounded(16);
/// The currency ceiling: `10^16`.
pub const CURRENCY_MAX: Max = Max::bounded(16);

const ZERO: &str = "零";
const NEGATIVE: &str = "负";
const DECIMAL_SEP: &str = "点";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

// Ordinal prefix
const ORDINAL_PREFIX: &str = "第";

// ============================================================================
// Currency Grammar (Chinese Yuan / Renminbi)
// ============================================================================

// "Whole" suffix when no jiao/fen
const ZHENG: &str = "整";

// ============================================================================
// Conversion
// ============================================================================

/// The integer as a `u64`. Every caller has already checked it against a
/// `10^16` ceiling, so this never fails in practice; it reports the ceiling
/// rather than panicking if it ever did.
fn below_ceiling(n: &BigUint, max: Max) -> Result<u64> {
    n.to_u64().ok_or(Error::OutOfRange { max })
}

/// Words for a number below 万 (10,000), with zero insertion for skipped
/// positions.
fn convert_below_wan(value: u64, v: &Numerals) -> String {
    if value == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut needs_zero = false;

    // Thousands (千)
    let thousands_val = value / 1000;
    let thousands_remainder = value % 1000;
    if thousands_val > 0 {
        result.push_str(v.ones[thousands_val as usize]);
        result.push_str(v.thousand);
        needs_zero = thousands_remainder > 0 && thousands_remainder < 100;
    }

    // Hundreds (百)
    let hundreds_val = thousands_remainder / 100;
    let hundreds_remainder = thousands_remainder % 100;
    if hundreds_val > 0 {
        if needs_zero {
            result.push_str(ZERO);
        }
        result.push_str(v.ones[hundreds_val as usize]);
        result.push_str(v.hundred);
        needs_zero = hundreds_remainder > 0 && hundreds_remainder < 10;
    } else if thousands_val > 0 && hundreds_remainder > 0 {
        needs_zero = true;
    }

    // Tens (十)
    let tens_val = hundreds_remainder / 10;
    let ones_val = hundreds_remainder % 10;
    if tens_val > 0 {
        if needs_zero {
            result.push_str(ZERO);
        }
        result.push_str(v.ones[tens_val as usize]);
        result.push_str(v.ten);
        needs_zero = false;
    } else if (hundreds_val > 0 || thousands_val > 0) && ones_val > 0 {
        needs_zero = true;
    }

    // Ones
    if ones_val > 0 {
        if needs_zero {
            result.push_str(ZERO);
        }
        result.push_str(v.ones[ones_val as usize]);
    }

    result
}

/// Words for a number below 亿 (100 million).
fn convert_below_yi(value: u64, v: &Numerals) -> String {
    if value == 0 {
        return String::new();
    }

    if value >= 10_000 {
        let wan_value = value / 10_000;
        let wan_remainder = value % 10_000;

        let mut result = convert_below_wan(wan_value, v);
        result.push_str(WAN_WORD);

        if wan_remainder > 0 {
            let needs_zero = wan_value % 10 == 0 || wan_remainder < 1000;
            if needs_zero {
                result.push_str(ZERO);
            }
            result.push_str(&convert_below_wan(wan_remainder, v));
        }

        return result;
    }

    convert_below_wan(value, v)
}

/// Words for a non-negative integer below `10^16`.
fn integer_to_words(n: u64, formal: bool) -> String {
    if n == 0 {
        return ZERO.to_string();
    }

    let v = numerals(formal);

    // Handle numbers >= 亿 (100 million)
    if n >= 100_000_000 {
        let yi_value = n / 100_000_000;
        let yi_remainder = n % 100_000_000;

        let mut result = convert_below_yi(yi_value, v);
        result.push_str(YI_WORD);

        if yi_remainder > 0 {
            if yi_remainder < 10_000_000 {
                result.push_str(ZERO);
            }
            result.push_str(&convert_below_yi(yi_remainder, v));
        }

        return result;
    }

    convert_below_yi(n, v)
}

/// Words for the digits after the decimal point, digit by digit.
fn decimal_digits_to_words(decimal: &str, v: &Numerals) -> String {
    let mut result = String::new();
    for digit in decimal.bytes() {
        result.push_str(v.ones[usize::from(digit - b'0')]);
    }
    result
}

// ============================================================================
// Cardinal
// ============================================================================

options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Use formal/financial numerals
        formal: bool = true;
    }
}

/// Spells a number in Simplified Chinese, in formal (financial) numerals.
///
/// ```
/// # use n2words::zh_hans_cn::to_cardinal;
/// assert_eq!(to_cardinal(2000000)?, "贰佰万");
/// assert_eq!(to_cardinal(-17.42)?, "负壹拾柒点肆贰");
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

/// Spells a number in Simplified Chinese, with options.
///
/// ```
/// # use n2words::zh_hans_cn::{to_cardinal_with, CardinalOptions};
/// let common = CardinalOptions { formal: false };
/// assert_eq!(to_cardinal_with(17.42, &common)?, "一十七点四二");
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
    // The fraction is spelled digit by digit, so only the integer part has a ceiling.
    CARDINAL_MAX.check(&number.integer, None)?;
    let integer = below_ceiling(&number.integer, CARDINAL_MAX)?;

    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
    }
    result.push_str(&integer_to_words(integer, options.formal));
    if let Some(fraction) = &number.fraction {
        result.push_str(DECIMAL_SEP);
        result.push_str(&decimal_digits_to_words(fraction, numerals(options.formal)));
    }
    Ok(result)
}

// ============================================================================
// Ordinal
// ============================================================================

/// Ordinal words for a positive integer: the 第 prefix plus the cardinal.
fn integer_to_ordinal(n: u64, formal: bool) -> String {
    format!("{ORDINAL_PREFIX}{}", integer_to_words(n, formal))
}

options! {
    /// Options for [`to_ordinal_with`].
    pub struct OrdinalOptions {
        /// Use formal/financial numerals
        formal: bool = true;
    }
}

/// Spells a positive integer as a Simplified Chinese ordinal (第 + cardinal).
///
/// ```
/// # use n2words::zh_hans_cn::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "第壹");
/// assert_eq!(to_ordinal(10)?, "第壹拾");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidOrdinal`](crate::Error::InvalidOrdinal) for zero, a
/// negative or a fraction; [`Error::OutOfRange`](crate::Error::OutOfRange) at
/// or past [`ORDINAL_MAX`].
pub fn to_ordinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_ordinal_with(value, &OrdinalOptions::default())
}

/// Spells a positive integer as a Simplified Chinese ordinal, with options.
///
/// ```
/// # use n2words::zh_hans_cn::{to_ordinal_with, OrdinalOptions};
/// let common = OrdinalOptions { formal: false };
/// assert_eq!(to_ordinal_with(1, &common)?, "第一");
/// assert_eq!(to_ordinal_with(101, &common)?, "第一百零一");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_ordinal`].
pub fn to_ordinal_with<'a>(
    value: impl Into<Number<'a>>,
    options: &OrdinalOptions,
) -> Result<String> {
    let n = parse_ordinal(&value.into())?;
    ORDINAL_MAX.check(&n, None)?;
    let n = below_ceiling(&n, ORDINAL_MAX)?;
    Ok(integer_to_ordinal(n, options.formal))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// Use formal/financial numerals
        formal: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::CNY, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Chinese yuan (renminbi), in formal numerals.
///
/// ```
/// # use n2words::zh_hans_cn::to_currency;
/// assert_eq!(to_currency(1)?, "壹圆整");
/// assert_eq!(to_currency(0.05)?, "伍分");
/// assert_eq!(to_currency(1.01)?, "壹圆零壹分");
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

/// Spells an amount in any currency Simplified Chinese has words for, with
/// options.
///
/// ```
/// # use n2words::zh_hans_cn::{to_currency_with, CurrencyOptions};
/// let common = CurrencyOptions { formal: false, ..Default::default() };
/// assert_eq!(to_currency_with(42.5, &common)?, "四十二元五角整");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Simplified Chinese has no words for and
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
    let yuan = below_ceiling(&money.major, CURRENCY_MAX)?;

    let v = numerals(options.formal);
    // `major[0]`/`[1]` are the formal/common denomination words (圆/元).
    let yuan_word = words.major_at(if options.formal { 0 } else { 1 });

    // Split cents into jiao (tens) and fen (ones)
    let jiao = (money.minor / 10) as usize;
    let fen = (money.minor % 10) as usize;

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
    }

    // Yuan part
    if yuan > 0 {
        result.push_str(&integer_to_words(yuan, options.formal));
        result.push_str(yuan_word);
    }

    // Jiao part (1/10). `jiao > 0` implies a minor amount, and
    // `assert_minor_unit` guaranteed it is zero whenever the currency has no
    // minor unit, so `minor_at` cannot panic here.
    if jiao > 0 {
        result.push_str(v.ones[jiao]);
        result.push_str(words.minor_at(0));
    } else if yuan > 0 && fen > 0 {
        // Zero placeholder between yuan and fen
        result.push_str(ZERO);
    }

    // Fen part (1/100)
    if fen > 0 {
        result.push_str(v.ones[fen]);
        result.push_str(words.minor_at(1));
    } else if yuan > 0 || jiao > 0 {
        // 整 (zheng) marks a whole amount
        result.push_str(ZHENG);
    }

    // Zero case
    if yuan == 0 && jiao == 0 && fen == 0 {
        result.push_str(ZERO);
        result.push_str(yuan_word);
        result.push_str(ZHENG);
    }

    Ok(result)
}

/// Registry entry for `zh-Hans-CN`.
pub static LANGUAGE: Language = Language {
    code: "zh-Hans-CN",
    name: "Chinese (Simplified, China)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(with_options to_ordinal_with, OrdinalOptions, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
