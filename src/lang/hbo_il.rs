//! Biblical Hebrew (Israel) (`hbo-IL`).
//!
//! - Gender agreement: masculine by default, feminine via the `gender` option
//! - Dual forms for 2, 200, 2000: "שניים", "מאתיים", "אלפיים"
//! - Special construct state for 1-9 thousand: "שלשה אלפים" / "שלשת אלפים"
//! - "ו" (ve) conjunction rules vary by position
//! - Per-digit decimal reading: 1.7 → "אחד נקודה שבעה"
//! - Currency counts the shekel in the masculine and the gerah in the feminine
//!
//! ```
//! use n2words::hbo_il::{to_cardinal, to_cardinal_with, to_currency, to_ordinal, CardinalOptions};
//! use n2words::Gender;
//!
//! assert_eq!(to_cardinal(42)?, "ארבעים ושניים");
//! let feminine = CardinalOptions { gender: Gender::Feminine, ..Default::default() };
//! assert_eq!(to_cardinal_with(2, &feminine)?, "שתים");
//! assert_eq!(to_ordinal(21)?, "עשרים וראשון");
//! assert_eq!(to_currency(42.5)?, "ארבעים ושניים שקלים חמישים גרות");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::currency::vocab::HBO as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, parse_cardinal, parse_currency, parse_ordinal, split_groups};
use crate::options::{Gender, options};
use crate::{Error, Result};
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

// Masculine forms (default in Biblical Hebrew); index 0 unused.
const ONES_MASC: [&str; 10] = [
    "",
    "אחד",
    "שניים",
    "שלשה",
    "ארבעה",
    "חמשה",
    "ששה",
    "שבעה",
    "שמונה",
    "תשעה",
];
const TEENS_MASC: [&str; 10] = [
    "עשרה",
    "אחד עשר",
    "שנים עשר",
    "שלשה עשר",
    "ארבעה עשר",
    "חמשה עשר",
    "ששה עשר",
    "שבעה עשר",
    "שמונה עשר",
    "תשעה עשר",
];
const THOUSANDS_MASC: [&str; 10] = [
    "",
    "אלף",
    "אלפיים",
    "שלשה אלפים",
    "ארבעה אלפים",
    "חמשה אלפים",
    "ששה אלפים",
    "שבעה אלפים",
    "שמונה אלפים",
    "תשעה אלפים",
];

// Feminine forms; index 0 unused.
const ONES_FEM: [&str; 10] = [
    "",
    "אחת",
    "שתים",
    "שלש",
    "ארבע",
    "חמש",
    "שש",
    "שבע",
    "שמונה",
    "תשע",
];
const TEENS_FEM: [&str; 10] = [
    "עשר",
    "אחת עשרה",
    "שתים עשרה",
    "שלש עשרה",
    "ארבע עשרה",
    "חמש עשרה",
    "שש עשרה",
    "שבע עשרה",
    "שמונה עשרה",
    "תשע עשרה",
];
const THOUSANDS_FEM: [&str; 10] = [
    "",
    "אלף",
    "אלפיים",
    "שלשת אלפים",
    "ארבעת אלפים",
    "חמשת אלפים",
    "ששת אלפים",
    "שבעת אלפים",
    "שמונת אלפים",
    "תשעת אלפים",
];

// Shared vocabulary
const TENS: [&str; 10] = [
    "",
    "",
    "עשרים",
    "שלשים",
    "ארבעים",
    "חמישים",
    "ששים",
    "שבעים",
    "שמונים",
    "תשעים",
];
const HUNDREDS: [&str; 10] = [
    "",
    "מאה",
    "מאתיים",
    "שלשה מאות",
    "ארבעה מאות",
    "חמשה מאות",
    "ששה מאות",
    "שבעה מאות",
    "שמונה מאות",
    "תשעה מאות",
];
const HUNDREDS_FEM: [&str; 10] = [
    "",
    "מאה",
    "מאתיים",
    "שלש מאות",
    "ארבע מאות",
    "חמש מאות",
    "שש מאות",
    "שבע מאות",
    "שמונה מאות",
    "תשע מאות",
];

// Scale words (index 1 = thousands, 2 = millions, ...).
const SCALE: &[&str] = &[
    "",
    "אלף",
    "מיליון",
    "מיליארד",
    "טריליון",
    "קוודרליון",
    "קווינטיליון",
];
const SCALE_PLURAL: &[&str] = &[
    "",
    "אלפים",
    "מיליונים",
    "מיליארדים",
    "טריליונים",
    "קוודרליונים",
    "קווינטיליונים",
];

/// The cardinal ceiling, derived from the scale table: `10^21`.
pub const CARDINAL_MAX: Max = Max::western(SCALE.len() as u32 - 1);
/// The ordinal ceiling: `10^9`. Past a million the millions multiplier is a
/// single 0-999 segment, so ordinals stop below a billion.
pub const ORDINAL_MAX: Max = Max::bounded(9);
/// The currency ceiling: `10^21`.
pub const CURRENCY_MAX: Max = Max::western(SCALE.len() as u32 - 1);

const ZERO: &str = "אפס";
const NEGATIVE: &str = "מינוס";
const DECIMAL_SEP: &str = "נקודה";
/// The default conjunction, "ve".
const AND_WORD: &str = "ו";

// Ordinal forms (masculine).
const ORDINAL_ONES: [&str; 10] = [
    "",
    "ראשון",
    "שני",
    "שלישי",
    "רביעי",
    "חמישי",
    "שישי",
    "שביעי",
    "שמיני",
    "תשיעי",
];
const ORDINAL_TEENS: [&str; 10] = [
    "עשירי",
    "אחד עשר",
    "שנים עשר",
    "שלשה עשר",
    "ארבעה עשר",
    "חמשה עשר",
    "ששה עשר",
    "שבעה עשר",
    "שמונה עשר",
    "תשעה עשר",
];
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "עשרים",
    "שלשים",
    "ארבעים",
    "חמישים",
    "ששים",
    "שבעים",
    "שמונים",
    "תשעים",
];
const ORDINAL_HUNDRED: &str = "מאה";

/// The gender-dependent part of the vocabulary.
struct Vocab {
    ones: [&'static str; 10],
    teens: [&'static str; 10],
    thousands: [&'static str; 10],
    hundreds: [&'static str; 10],
}

static MASCULINE: Vocab = Vocab {
    ones: ONES_MASC,
    teens: TEENS_MASC,
    thousands: THOUSANDS_MASC,
    hundreds: HUNDREDS,
};

static FEMININE: Vocab = Vocab {
    ones: ONES_FEM,
    teens: TEENS_FEM,
    thousands: THOUSANDS_FEM,
    hundreds: HUNDREDS_FEM,
};

/// The vocabulary agreeing with `gender`.
fn vocab(gender: Gender) -> &'static Vocab {
    match gender {
        Gender::Masculine => &MASCULINE,
        Gender::Feminine => &FEMININE,
    }
}

// ============================================================================
// Segment building
// ============================================================================

/// Appends `word` to `result`, preceded by `joiner` unless `result` is empty.
fn append(result: &mut String, joiner: &str, word: &str) {
    if !result.is_empty() {
        result.push_str(joiner);
    }
    result.push_str(word);
}

/// Words for a 0-999 segment that precedes a scale word (thousands, millions,
/// ...): "ו" is added before the tens and the ones when they follow hundreds.
fn build_scale_segment(n: u32, and_word: &str, v: &Vocab) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut result = String::new();
    if hundreds > 0 {
        result.push_str(v.hundreds[hundreds]);
    }
    let joiner = format!(" {and_word}");
    if tens == 1 {
        // Teens (10-19)
        append(&mut result, &joiner, v.teens[ones]);
    } else {
        // Tens (20-90)
        if tens >= 2 {
            append(&mut result, &joiner, TENS[tens]);
        }
        // Ones
        if ones > 0 {
            append(&mut result, &joiner, v.ones[ones]);
        }
    }
    result
}

/// Words for the units segment (no scale word): "ו" is only added before the
/// final ones digit.
fn build_units_segment(n: u32, and_word: &str, v: &Vocab) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut result = String::new();
    if hundreds > 0 {
        result.push_str(v.hundreds[hundreds]);
    }
    if tens == 1 {
        // Teens (10-19), no conjunction
        append(&mut result, " ", v.teens[ones]);
    } else {
        // Tens (20-90), no conjunction
        if tens >= 2 {
            append(&mut result, " ", TENS[tens]);
        }
        // Ones: the conjunction goes only here
        if ones > 0 {
            append(&mut result, &format!(" {and_word}"), v.ones[ones]);
        }
    }
    result
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer, agreeing with `gender`.
fn integer_to_words(n: &BigUint, gender: Gender, and_word: &str) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }
    let v = vocab(gender);
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_units_segment(small as u32, and_word, v);
        }
    }

    let segments = split_groups(n, 1000);
    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if i == 0 {
            // Units segment (no scale word): "ו" before single-digit units
            // when they follow a scale word.
            let word = build_units_segment(segment, and_word, v);
            if result.is_empty() {
                result = word;
            } else {
                result.push(' ');
                if segment <= 9 {
                    result.push_str(and_word);
                }
                result.push_str(&word);
            }
        } else if i == 1 {
            // Thousands: special construct forms for 1-9.
            if !result.is_empty() {
                result.push(' ');
            }
            if segment <= 9 {
                result.push_str(v.thousands[segment as usize]);
            } else {
                result.push_str(&build_scale_segment(segment, and_word, v));
                result.push(' ');
                result.push_str(SCALE[1]);
            }
        } else {
            // Millions and above: a bare scale word for 1, plural otherwise.
            if !result.is_empty() {
                result.push(' ');
            }
            if segment == 1 {
                result.push_str(SCALE[i]);
            } else {
                result.push_str(&build_scale_segment(segment, and_word, v));
                result.push(' ');
                result.push_str(SCALE_PLURAL[i]);
            }
        }
    }
    result
}

/// Words for the digits after the decimal point, read digit by digit.
fn decimal_part_to_words(decimal: &str, gender: Gender) -> String {
    let ones = &vocab(gender).ones;
    let mut result = String::new();
    for digit in decimal.chars() {
        let d = digit.to_digit(10).unwrap_or(0) as usize;
        append(&mut result, " ", if d == 0 { ZERO } else { ones[d] });
    }
    result
}

options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Grammatical gender
        gender: Gender = Gender::Masculine;
        /// Custom conjunction word
        and_word: String = AND_WORD.to_string();
    }
}

/// Spells a number in Biblical Hebrew words (masculine forms).
///
/// ```
/// # use n2words::hbo_il::to_cardinal;
/// assert_eq!(to_cardinal(4196)?, "ארבעה אלפים מאה תשעים וששה");
/// assert_eq!(to_cardinal(-1.7)?, "מינוס אחד נקודה שבעה");
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

/// Spells a number in Biblical Hebrew words, with options.
///
/// ```
/// # use n2words::{hbo_il::{to_cardinal_with, CardinalOptions}, Gender};
/// let feminine = CardinalOptions { gender: Gender::Feminine, ..Default::default() };
/// assert_eq!(to_cardinal_with(5000, &feminine)?, "חמשת אלפים");
/// let plus = CardinalOptions { and_word: "+".to_string(), ..Default::default() };
/// assert_eq!(to_cardinal_with(123, &plus)?, "מאה עשרים +שלשה");
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
    // The fraction is read digit by digit, so only the integer part has a ceiling.
    CARDINAL_MAX.check(&number.integer, None)?;

    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }
    result.push_str(&integer_to_words(
        &number.integer,
        options.gender,
        &options.and_word,
    ));
    if let Some(fraction) = &number.fraction {
        result.push(' ');
        result.push_str(DECIMAL_SEP);
        result.push(' ');
        result.push_str(&decimal_part_to_words(fraction, options.gender));
    }
    Ok(result)
}

// ============================================================================
// Ordinal
// ============================================================================

/// Ordinal words for 0-99: "ראשון", "עשירי", "עשרים וראשון".
fn build_ordinal_tens_ones(n: u32) -> String {
    let n = n as usize;
    if n == 0 {
        return String::new();
    }
    if n < 10 {
        return ORDINAL_ONES[n].to_string();
    }
    if n < 20 {
        return ORDINAL_TEENS[n - 10].to_string();
    }
    let ones = n % 10;
    let tens = n / 10;
    // Round tens take "וראשון"; in a compound only the ones are ordinal.
    let last = if ones == 0 {
        ORDINAL_ONES[1]
    } else {
        ORDINAL_ONES[ones]
    };
    format!("{} ו{last}", ORDINAL_TENS[tens])
}

/// Ordinal words for a positive integer below `10^9`: cardinal (masculine)
/// higher segments, then the remainder in ordinal form.
fn integer_to_ordinal(n: u64) -> String {
    if n == 0 {
        return String::new();
    }
    if n == 1 {
        return ORDINAL_ONES[1].to_string();
    }
    if n < 100 {
        return build_ordinal_tens_ones(n as u32);
    }
    if n < 1000 {
        let hundreds = (n / 100) as usize;
        let remainder = (n % 100) as u32;
        if remainder == 0 {
            return ORDINAL_HUNDRED.to_string();
        }
        return format!(
            "{} {}",
            HUNDREDS[hundreds],
            build_ordinal_tens_ones(remainder)
        );
    }
    if n < 1_000_000 {
        let thousands = (n / 1000) as u32;
        let remainder = (n % 1000) as u32;
        // Cardinal thousands + ordinal remainder
        let mut result = if thousands <= 9 {
            THOUSANDS_MASC[thousands as usize].to_string()
        } else {
            format!(
                "{} {}",
                build_scale_segment(thousands, AND_WORD, &MASCULINE),
                SCALE[1]
            )
        };
        if remainder == 0 {
            return result;
        }
        if remainder < 100 {
            result.push(' ');
            result.push_str(&build_ordinal_tens_ones(remainder));
            return result;
        }
        let rem_hundreds = (remainder / 100) as usize;
        let rem_tens_ones = remainder % 100;
        result.push(' ');
        result.push_str(HUNDREDS[rem_hundreds]);
        if rem_tens_ones > 0 {
            result.push(' ');
            result.push_str(&build_ordinal_tens_ones(rem_tens_ones));
        }
        return result;
    }
    // Millions: the multiplier is a single 0-999 segment (hence the ceiling).
    let millions = (n / 1_000_000) as u32;
    let remainder = n % 1_000_000;
    let mut result = if millions == 1 {
        SCALE[2].to_string()
    } else {
        format!(
            "{} {}",
            build_scale_segment(millions, AND_WORD, &MASCULINE),
            SCALE_PLURAL[2]
        )
    };
    if remainder > 0 {
        result.push(' ');
        result.push_str(&integer_to_ordinal(remainder));
    }
    result
}

/// Spells a positive integer as a Biblical Hebrew ordinal (masculine).
///
/// ```
/// # use n2words::hbo_il::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "ראשון");
/// assert_eq!(to_ordinal(2000000)?, "שניים מיליונים");
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
    // Below the 10^9 ceiling the value always fits a machine word.
    let n = n.to_u64().ok_or(Error::OutOfRange { max: ORDINAL_MAX })?;
    Ok(integer_to_ordinal(n))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::ILS, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in shekels and gerah.
///
/// ```
/// # use n2words::hbo_il::to_currency;
/// assert_eq!(to_currency(1)?, "שקל אחד");
/// assert_eq!(to_currency(2.5)?, "שניים שקלים חמישים גרות");
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

/// Spells an amount in any currency Biblical Hebrew has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Biblical Hebrew has no words for and
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

    let mut parts: Vec<String> = Vec::new();
    if money.negative {
        parts.push(NEGATIVE.to_string());
    }
    // Shekels (masculine): the noun precedes "one", "two" takes the dual.
    if !money.major.is_zero() || money.minor == 0 {
        parts.push(match money.major.to_u64() {
            Some(1) => format!("{} {}", words.major_at(0), ONES_MASC[1]),
            Some(2) => format!("{} {}", ONES_MASC[2], words.major_at(1)),
            _ => format!(
                "{} {}",
                integer_to_words(&money.major, Gender::Masculine, AND_WORD),
                words.major_at(1)
            ),
        });
    }
    // Gerah (feminine subunit)
    if money.minor > 0 {
        parts.push(if money.minor == 1 {
            format!("{} {}", words.minor_at(0), ONES_FEM[1])
        } else {
            format!(
                "{} {}",
                integer_to_words(&BigUint::from(money.minor), Gender::Feminine, AND_WORD),
                words.minor_at(1)
            )
        });
    }
    Ok(parts.join(" "))
}

/// Registry entry for `hbo-IL`.
pub static LANGUAGE: Language = Language {
    code: "hbo-IL",
    name: "Biblical Hebrew (Israel)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
