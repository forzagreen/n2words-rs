//! Japanese (Japan) (`ja-JP`).
//!
//! - Myriad (万-based) grouping: 4 digits per segment instead of 3
//! - 一 omission: "一" is omitted before 十, 百, 千 but NOT before 万 and
//!   higher scales
//! - Kanji numerals: 零一二三四五六七八九
//! - No spaces between characters
//!
//! ```
//! use n2words::ja_jp::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "四十二");
//! assert_eq!(to_ordinal(42)?, "第四十二");
//! assert_eq!(to_currency(10000)?, "一万円");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::JA as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, parse_cardinal, parse_currency, parse_ordinal, split_groups};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

// Ones words (1-9), index 0 unused
const ONES: [&str; 10] = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九"];

// Scale words for powers of 10,000 (万-based system)
// Index 0 = 万 (10^4), 1 = 億 (10^8), 2 = 兆 (10^12), etc.
const SCALES: &[&str] = &[
    "万",       // 10^4 (man)
    "億",       // 10^8 (oku)
    "兆",       // 10^12 (chō)
    "京",       // 10^16 (kei)
    "垓",       // 10^20 (gai)
    "秭",       // 10^24 (jo/shi)
    "穣",       // 10^28 (jō)
    "溝",       // 10^32 (kō)
    "澗",       // 10^36 (kan)
    "正",       // 10^40 (sei)
    "載",       // 10^44 (sai)
    "極",       // 10^48 (goku)
    "恒河沙",   // 10^52 (gōgasha)
    "阿僧祇",   // 10^56 (asōgi)
    "那由他",   // 10^60 (nayuta)
    "不可思議", // 10^64 (fukashigi)
    "無量大数", // 10^68 (muryōtaisū)
];

/// The cardinal ceiling, derived from the scale table: `10^72`. Each scale
/// word covers a power of 10,000, so the first unsupported value is
/// `10^((SCALES.len() + 1) * 4)`.
pub const CARDINAL_MAX: Max = Max::myriad(SCALES.len() as u32);
/// The ordinal ceiling: `10^72` (ordinals prefix the cardinal speller).
pub const ORDINAL_MAX: Max = Max::myriad(SCALES.len() as u32);
/// The currency ceiling: `10^72` (currency builds on the cardinal speller).
pub const CURRENCY_MAX: Max = Max::myriad(SCALES.len() as u32);

const ZERO: &str = "零";
const NEGATIVE: &str = "マイナス";
const DECIMAL_SEP: &str = "点";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

// Ordinal prefix
const ORDINAL_PREFIX: &str = "第";

// Internal scale words (within 4-digit segments)
const TEN: &str = "十";
const HUNDRED: &str = "百";
const THOUSAND: &str = "千";

// ============================================================================
// Segment Building
// ============================================================================

/// Appends `ONES[digit] + scale`, omitting 一 when the digit is 1.
fn push_place(result: &mut String, digit: usize, scale: &str) {
    if digit > 0 {
        if digit != 1 {
            result.push_str(ONES[digit]);
        }
        result.push_str(scale);
    }
}

/// Words for a 0-9999 segment with the 一 omission rules: 一 is omitted
/// before 十, 百 and 千.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = ((n / 100) % 10) as usize;
    let thousands = (n / 1000) as usize;

    let mut result = String::new();
    push_place(&mut result, thousands, THOUSAND);
    push_place(&mut result, hundreds, HUNDRED);
    push_place(&mut result, tens, TEN);
    if ones > 0 {
        result.push_str(ONES[ones]);
    }
    result
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
        // Fast path: numbers < 10000
        if small < 10_000 {
            return build_segment(small as u32);
        }

        // Fast path: numbers < 100,000,000 (万 range)
        if small < 100_000_000 {
            let man = (small / 10_000) as u32;
            let remainder = (small % 10_000) as u32;

            // For 万 and above, 一 is needed before the scale word when the
            // segment is 1.
            let mut result = if man == 1 {
                format!("{}{}", ONES[1], SCALES[0]) // 一万
            } else {
                format!("{}{}", build_segment(man), SCALES[0])
            };
            if remainder > 0 {
                result.push_str(&build_segment(remainder));
            }
            return result;
        }
    }

    // For numbers >= 100,000,000, use scale decomposition
    build_large_number_words(n)
}

/// Words for a number of 100,000,000 or more, by 4-digit (myriad) segments.
fn build_large_number_words(n: &BigUint) -> String {
    // Segments least-significant first (index 0 = units, 1 = 万, etc.)
    let segments = split_groups(n, 10_000);

    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if i > 0 {
            // For scales >= 万, 一 is needed before the scale word when the
            // segment is 1.
            if segment == 1 {
                result.push_str(ONES[1]);
            } else {
                result.push_str(&build_segment(segment));
            }
            result.push_str(SCALES[i - 1]);
        } else {
            // Units segment (no scale word)
            result.push_str(&build_segment(segment));
        }
    }

    if result.is_empty() {
        ZERO.to_string()
    } else {
        result
    }
}

/// Words for the digits after the decimal point, digit by digit.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut result = String::new();
    for digit in decimal.bytes() {
        let digit = usize::from(digit - b'0');
        if digit == 0 {
            result.push_str(ZERO);
        } else {
            result.push_str(ONES[digit]);
        }
    }
    result
}

/// Spells a number in Japanese kanji numerals.
///
/// ```
/// # use n2words::ja_jp::to_cardinal;
/// assert_eq!(to_cardinal(10000)?, "一万");
/// assert_eq!(to_cardinal(100000000)?, "一億");
/// assert_eq!(to_cardinal(-17.42)?, "マイナス十七点四二");
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
    // The fraction is spelled digit by digit, so only the integer part has a ceiling.
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

/// Ordinal words for a positive integer: the 第 prefix plus the cardinal.
fn integer_to_ordinal(n: &BigUint) -> String {
    format!("{ORDINAL_PREFIX}{}", integer_to_words(n))
}

/// Spells a positive integer as a Japanese ordinal (第 + cardinal).
///
/// ```
/// # use n2words::ja_jp::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "第一");
/// assert_eq!(to_ordinal(10)?, "第十");
/// assert_eq!(to_ordinal(100)?, "第百");
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
    // Ordinals prefix the cardinal speller, so they share its ceiling.
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
        currency: Currency = Currency::JPY, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Japanese yen.
///
/// Yen has no everyday minor unit: a fractional amount is an error rather
/// than spelling a historical, no-longer-circulating 銭 (sen), which was
/// demonetised in 1953.
///
/// ```
/// # use n2words::ja_jp::to_currency;
/// assert_eq!(to_currency(42)?, "四十二円");
/// assert_eq!(to_currency(1)?, "一円");
/// assert_eq!(to_currency(0)?, "零円");
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

/// Spells an amount in any currency Japanese has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Japanese has no words for.
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
    if money.major.is_zero() {
        result.push_str(ZERO);
    } else {
        result.push_str(&integer_to_words(&money.major));
    }
    result.push_str(words.major_at(0));
    Ok(result)
}

/// Registry entry for `ja-JP`.
pub static LANGUAGE: Language = Language {
    code: "ja-JP",
    name: "Japanese (Japan)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
