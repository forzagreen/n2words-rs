//! Polish (Poland) (`pl-PL`).
//!
//! - Three-form pluralization: 1 = singular, 2-4 = few, 5+ = many
//!   ("jeden złoty", "dwa złote", "pięć złotych")
//! - Gender agreement (masculine/feminine) for numbers below 1000:
//!   "jeden" / "jedna", "dwa" / "dwie"
//! - "jeden" is omitted before scale words: 1000 → "tysiąc", not
//!   "jeden tysiąc"
//! - Irregular hundreds: "dwieście", "trzysta", "czterysta", "pięćset"
//! - Long scale with -ard forms: miliard, biliard, tryliard
//! - Ordinals in the masculine nominative; only the last component is
//!   ordinal, and compound tens are ordinal too: 121 → "sto dwudziesty
//!   pierwszy"
//!
//! ```
//! use n2words::pl_pl::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "dwadzieścia jeden");
//! assert_eq!(to_ordinal(42)?, "czterdziesty drugi");
//! assert_eq!(to_currency(42.5)?, "czterdzieści dwa złote pięćdziesiąt groszy");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::PL as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{
    Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal, split_groups,
};
use crate::options::{Gender, options};
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

const ONES_MASC: [&str; 10] = [
    "",
    "jeden",
    "dwa",
    "trzy",
    "cztery",
    "pięć",
    "sześć",
    "siedem",
    "osiem",
    "dziewięć",
];
const ONES_FEM: [&str; 10] = [
    "",
    "jedna",
    "dwie",
    "trzy",
    "cztery",
    "pięć",
    "sześć",
    "siedem",
    "osiem",
    "dziewięć",
];
const TEENS: [&str; 10] = [
    "dziesięć",
    "jedenaście",
    "dwanaście",
    "trzynaście",
    "czternaście",
    "piętnaście",
    "szesnaście",
    "siedemnaście",
    "osiemnaście",
    "dziewiętnaście",
];
const TENS: [&str; 10] = [
    "",
    "",
    "dwadzieścia",
    "trzydzieści",
    "czterdzieści",
    "pięćdziesiąt",
    "sześćdziesiąt",
    "siedemdziesiąt",
    "osiemdziesiąt",
    "dziewięćdziesiąt",
];
// Irregular hundreds
const HUNDREDS: [&str; 10] = [
    "",
    "sto",
    "dwieście",
    "trzysta",
    "czterysta",
    "pięćset",
    "sześćset",
    "siedemset",
    "osiemset",
    "dziewięćset",
];

// Scale words `[singular, few (2-4), many (5+)]`, index 0 = thousands. The
// forms are the JavaScript source's, including its "kwaryliard" singular.
const SCALE_FORMS: [[&str; 3]; 10] = [
    ["tysiąc", "tysiące", "tysięcy"],
    ["milion", "miliony", "milionów"],
    ["miliard", "miliardy", "miliardów"],
    ["bilion", "biliony", "bilionów"],
    ["biliard", "biliardy", "biliardów"],
    ["trylion", "tryliony", "trylionów"],
    ["tryliard", "tryliardy", "tryliardów"],
    ["kwadrylion", "kwadryliony", "kwadrylionów"],
    ["kwaryliard", "kwadryliardy", "kwadryliardów"],
    ["kwintylion", "kwintyliony", "kwintylionów"],
];

const ZERO: &str = "zero";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "przecinek";

// ============================================================================
// Ordinal vocabulary (masculine nominative singular)
// ============================================================================

const ORDINAL_ONES: [&str; 10] = [
    "",
    "pierwszy",
    "drugi",
    "trzeci",
    "czwarty",
    "piąty",
    "szósty",
    "siódmy",
    "ósmy",
    "dziewiąty",
];
const ORDINAL_TEENS: [&str; 10] = [
    "dziesiąty",
    "jedenasty",
    "dwunasty",
    "trzynasty",
    "czternasty",
    "piętnasty",
    "szesnasty",
    "siedemnasty",
    "osiemnasty",
    "dziewiętnasty",
];
// Ordinal tens, also used in compounds ("dwudziesty pierwszy")
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "dwudziesty",
    "trzydziesty",
    "czterdziesty",
    "pięćdziesiąty",
    "sześćdziesiąty",
    "siedemdziesiąty",
    "osiemdziesiąty",
    "dziewięćdziesiąty",
];
// Exact hundreds
const ORDINAL_HUNDREDS: [&str; 10] = [
    "",
    "setny",
    "dwusetny",
    "trzechsetny",
    "czterechsetny",
    "pięćsetny",
    "sześćsetny",
    "siedemsetny",
    "osiemsetny",
    "dziewięćsetny",
];
// Scale ordinals
const ORDINAL_SCALES: [&str; 7] = [
    "tysięczny",
    "milionowy",
    "miliardowy",
    "bilionowy",
    "biliardowy",
    "trylionowy",
    "tryliardowy",
];

/// The cardinal ceiling, derived from the scale table: `10^33`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_FORMS.len() as u32);
/// The ordinal ceiling, derived from the shorter ordinal scale table: `10^24`.
pub const ORDINAL_MAX: Max = Max::western(ORDINAL_SCALES.len() as u32);
/// The currency ceiling: `10^33`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_FORMS.len() as u32);

// ============================================================================
// Shared helpers
// ============================================================================

/// The `[singular, few, many]` form index for a count other than 1, from its
/// last two digits: a final 2-4 outside 12-14 is "few", everything else
/// (0, 5-9, 11-19, 21, ...) "many".
fn plural_index_tail(last_two: u32) -> usize {
    if (2..=4).contains(&(last_two % 10)) && !(10..=20).contains(&last_two) {
        1
    } else {
        2
    }
}

/// The `[singular, few, many]` form index for a count: exactly 1 is the
/// singular, otherwise [`plural_index_tail`] decides.
fn plural_index(n: u32) -> usize {
    if n == 1 {
        0
    } else {
        plural_index_tail(n % 100)
    }
}

/// [`plural_index`] for an arbitrarily large count.
fn plural_index_big(n: &BigUint) -> usize {
    if n.is_one() {
        0
    } else {
        plural_index_tail((n % 100u32).to_u32().unwrap_or(0))
    }
}

/// Picks the `[singular, few, many]` form for `n`.
fn pluralize<'a>(n: u32, forms: &[&'a str; 3]) -> &'a str {
    forms[plural_index(n)]
}

/// Cardinal words for a 0-999 segment: irregular hundreds, then teens or
/// tens and ones. Only the ones differ by gender ("jeden" / "jedna",
/// "dwa" / "dwie"), so one builder serves both.
fn build_segment(n: u32, gender: Gender) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;
    let ones_words = match gender {
        Gender::Masculine => &ONES_MASC,
        Gender::Feminine => &ONES_FEM,
    };

    let mut parts: Vec<&str> = Vec::with_capacity(3);
    if hundreds > 0 {
        parts.push(HUNDREDS[hundreds]);
    }
    if tens == 1 {
        // Teens (10-19)
        parts.push(TEENS[ones]);
    } else {
        if tens >= 2 {
            parts.push(TENS[tens]);
        }
        if ones > 0 {
            parts.push(ones_words[ones]);
        }
    }
    parts.join(" ")
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer. `gender` applies below 1000 only; from
/// a thousand up every segment is masculine, as in the JavaScript source.
fn integer_to_words(n: &BigUint, gender: Gender) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32, gender);
        }

        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            let scale_word = pluralize(thousands, &SCALE_FORMS[0]);
            let mut result = if thousands == 1 {
                // Omit "jeden" before tysiąc
                scale_word.to_string()
            } else {
                format!(
                    "{} {scale_word}",
                    build_segment(thousands, Gender::Masculine)
                )
            };
            if remainder > 0 {
                result.push(' ');
                result.push_str(&build_segment(remainder, Gender::Masculine));
            }
            return result;
        }
    }

    build_large_number_words(n)
}

/// Words for a number of one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if !result.is_empty() {
            result.push(' ');
        }
        if i == 0 {
            result.push_str(&build_segment(segment, Gender::Masculine));
        } else {
            let scale_word = pluralize(segment, &SCALE_FORMS[i - 1]);
            if segment == 1 {
                // Omit "jeden" before scale words
                result.push_str(scale_word);
            } else {
                result.push_str(&build_segment(segment, Gender::Masculine));
                result.push(' ');
                result.push_str(scale_word);
            }
        }
    }
    result
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
fn decimal_part_to_words(decimal: &str, gender: Gender) -> String {
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
        result.push_str(&integer_to_words(&digits_to_biguint(remainder), gender));
    }
    result
}

options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Gender for numbers < 1000
        gender: Gender = Gender::Masculine;
    }
}

/// Spells a number in Polish words (masculine).
///
/// ```
/// # use n2words::pl_pl::to_cardinal;
/// assert_eq!(to_cardinal(1000)?, "tysiąc");
/// assert_eq!(to_cardinal(1.7)?, "jeden przecinek siedem");
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

/// Spells a number in Polish words, with options.
///
/// ```
/// # use n2words::{pl_pl::{to_cardinal_with, CardinalOptions}, Gender};
/// let feminine = CardinalOptions { gender: Gender::Feminine };
/// assert_eq!(to_cardinal_with(22, &feminine)?, "dwadzieścia dwie");
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

    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }
    result.push_str(&integer_to_words(&number.integer, options.gender));
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

/// Ordinal words for a 0-99 final segment: "pierwszy", "dwudziesty pierwszy".
fn build_ordinal_tens_ones(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = (n / 10) as usize;

    if tens == 0 {
        // Single digit: pierwszy, drugi, ...
        return ORDINAL_ONES[ones].to_string();
    }
    if tens == 1 {
        // Teens: dziesiąty, jedenasty, ...
        return ORDINAL_TEENS[ones].to_string();
    }
    if ones == 0 {
        // Round tens: dwudziesty, trzydziesty, ...
        return ORDINAL_TENS[tens].to_string();
    }
    // Compound: both parts ordinal (dwudziesty pierwszy)
    format!("{} {}", ORDINAL_TENS[tens], ORDINAL_ONES[ones])
}

/// Ordinal words for a 0-999 final segment.
fn build_ordinal_segment(n: u32) -> String {
    if n < 100 {
        return build_ordinal_tens_ones(n);
    }
    let hundreds = (n / 100) as usize;
    let remainder = n % 100;
    if remainder == 0 {
        // Exact hundreds: setny, dwusetny, ...
        return ORDINAL_HUNDREDS[hundreds].to_string();
    }
    // Cardinal hundreds + ordinal remainder
    format!(
        "{} {}",
        HUNDREDS[hundreds],
        build_ordinal_tens_ones(remainder)
    )
}

/// Ordinal words for a positive integer (masculine nominative).
///
/// Only the last component becomes ordinal: 121 → "sto dwudziesty pierwszy".
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_ordinal_segment(small as u32);
        }

        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            if remainder == 0 {
                // Exact thousands: tysięczny, then cardinal + tysięczny
                if thousands == 1 {
                    return ORDINAL_SCALES[0].to_string();
                }
                return format!(
                    "{} {}",
                    build_segment(thousands, Gender::Masculine),
                    ORDINAL_SCALES[0]
                );
            }

            // Cardinal thousands (no "jeden" before tysiąc) + ordinal remainder
            let scale_word = pluralize(thousands, &SCALE_FORMS[0]);
            let thousands_word = if thousands == 1 {
                String::new()
            } else {
                format!("{} ", build_segment(thousands, Gender::Masculine))
            };
            return format!(
                "{thousands_word}{scale_word} {}",
                build_ordinal_segment(remainder)
            );
        }
    }

    build_large_ordinal(n)
}

/// Ordinal words for one million or more: cardinal segments, then the lowest
/// non-zero segment in ordinal form.
fn build_large_ordinal(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let lowest_non_zero = segments.iter().position(|&s| s != 0).unwrap_or(0);

    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if !result.is_empty() {
            result.push(' ');
        }
        if i == 0 {
            // The units segment, when present, is always the final one
            result.push_str(&build_ordinal_segment(segment));
            continue;
        }
        if i == lowest_non_zero {
            // This scale position is the final, ordinal one
            if segment == 1 {
                result.push_str(ORDINAL_SCALES[i - 1]);
            } else {
                result.push_str(&build_segment(segment, Gender::Masculine));
                result.push(' ');
                result.push_str(ORDINAL_SCALES[i - 1]);
            }
        } else {
            let scale_word = pluralize(segment, &SCALE_FORMS[i - 1]);
            if segment == 1 {
                result.push_str(scale_word);
            } else {
                result.push_str(&build_segment(segment, Gender::Masculine));
                result.push(' ');
                result.push_str(scale_word);
            }
        }
    }
    result
}

/// Spells a positive integer as a Polish ordinal (masculine nominative).
///
/// ```
/// # use n2words::pl_pl::to_ordinal;
/// assert_eq!(to_ordinal(21)?, "dwudziesty pierwszy");
/// assert_eq!(to_ordinal(100)?, "setny");
/// assert_eq!(to_ordinal(1000)?, "tysięczny");
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
        currency: Currency = Currency::PLN, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Polish złoty.
///
/// ```
/// # use n2words::pl_pl::to_currency;
/// assert_eq!(to_currency(1)?, "jeden złoty");
/// assert_eq!(to_currency(1.5)?, "jeden złoty pięćdziesiąt groszy");
/// assert_eq!(to_currency(-5)?, "minus pięć złotych");
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

/// Spells an amount in any currency Polish has words for.
///
/// ```
/// # use n2words::{pl_pl::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::PLN };
/// assert_eq!(to_currency_with(0.01, &opts)?, "jeden grosz");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Polish has no words for and
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
    // Every Polish entry carries both genders; the fallbacks are the default
    // nouns' own (złoty and grosz are both masculine).
    let major_gender = words.major_gender.unwrap_or(Gender::Masculine);
    let minor_gender = words.minor_gender.unwrap_or(Gender::Masculine);

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }
    // The major-unit noun's own gender drives agreement.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major, major_gender));
        result.push(' ');
        result.push_str(words.major_at(plural_index_big(&money.major)));
    }
    // The minor-unit noun's own gender likewise.
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor), minor_gender));
        result.push(' ');
        result.push_str(words.minor_at(plural_index(money.minor)));
    }
    Ok(result)
}

/// Registry entry for `pl-PL`.
pub static LANGUAGE: Language = Language {
    code: "pl-PL",
    name: "Polish (Poland)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
