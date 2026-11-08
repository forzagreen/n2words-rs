//! Czech (Czechia) (`cs-CZ`).
//!
//! - Three-form pluralization: 1 = singular, 2-4 = few, 5+ = many
//!   ("jedna koruna", "dva koruny", "pět korun")
//! - Irregular hundreds: "sto", "dvě stě", "tři sta", "čtyři sta", "pět set"
//! - "one" is omitted before scale words: 1000 → "tisíc", not "jedna tisíc"
//! - Dynamic decimal separator agreeing with the integer part: "celá" after
//!   0 and 1, "celé" after 2-4, "celých" after 5 and up
//! - Long scale with -arda forms: miliarda, biliarda, triliarda
//! - Ordinals in the masculine nominative; only the last component is
//!   ordinal: 121 → "sto dvacet první"
//!
//! ```
//! use n2words::cs_cz::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "dvacet jedna");
//! assert_eq!(to_ordinal(42)?, "čtyřicet druhý");
//! assert_eq!(to_currency(42.5)?, "čtyřicet dva koruny padesát haléřů");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::CS as CURRENCY_VOCAB;
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

// Ones words (the single, counting form)
const ONES: [&str; 10] = [
    "", "jedna", "dva", "tři", "čtyři", "pět", "šest", "sedm", "osm", "devět",
];
const TEENS: [&str; 10] = [
    "deset",
    "jedenáct",
    "dvanáct",
    "třináct",
    "čtrnáct",
    "patnáct",
    "šestnáct",
    "sedmnáct",
    "osmnáct",
    "devatenáct",
];
const TENS: [&str; 10] = [
    "",
    "",
    "dvacet",
    "třicet",
    "čtyřicet",
    "padesát",
    "šedesát",
    "sedmdesát",
    "osmdesát",
    "devadesát",
];
// Irregular hundreds
const HUNDREDS: [&str; 10] = [
    "",
    "sto",
    "dvě stě",
    "tři sta",
    "čtyři sta",
    "pět set",
    "šest set",
    "sedm set",
    "osm set",
    "devět set",
];

// Scale words `[singular, few (2-4), many (5+)]`, index 0 = thousands.
const SCALE_FORMS: [[&str; 3]; 9] = [
    ["tisíc", "tisíce", "tisíc"],
    ["milion", "miliony", "milionů"],
    ["miliarda", "miliardy", "miliard"],
    ["bilion", "biliony", "bilionů"],
    ["biliarda", "biliardy", "biliard"],
    ["trilion", "triliony", "trilionů"],
    ["triliarda", "triliardy", "triliard"],
    ["kvadrilion", "kvadriliony", "kvadrilionů"],
    ["kvadriliarda", "kvadriliardy", "kvadriliard"],
];

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_FORMS.len() as u32);
/// The ordinal ceiling: ordinals reuse the cardinal scale table, so `10^30`.
pub const ORDINAL_MAX: Max = Max::western(SCALE_FORMS.len() as u32);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_FORMS.len() as u32);

const ZERO: &str = "nula";
const NEGATIVE: &str = "mínus";

// ============================================================================
// Ordinal vocabulary (masculine nominative singular)
// ============================================================================

const ORDINAL_ONES: [&str; 10] = [
    "", "první", "druhý", "třetí", "čtvrtý", "pátý", "šestý", "sedmý", "osmý", "devátý",
];
const ORDINAL_TEENS: [&str; 10] = [
    "desátý",
    "jedenáctý",
    "dvanáctý",
    "třináctý",
    "čtrnáctý",
    "patnáctý",
    "šestnáctý",
    "sedmnáctý",
    "osmnáctý",
    "devatenáctý",
];
// Exact tens
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "dvacátý",
    "třicátý",
    "čtyřicátý",
    "padesátý",
    "šedesátý",
    "sedmdesátý",
    "osmdesátý",
    "devadesátý",
];
// Exact hundreds
const ORDINAL_HUNDREDS: [&str; 10] = [
    "",
    "stý",
    "dvoustý",
    "třístý",
    "čtyřstý",
    "pětistý",
    "šestistý",
    "sedmistý",
    "osmistý",
    "devítistý",
];
// Scale ordinals; past this table the singular cardinal scale word stands in
const ORDINAL_SCALES: [&str; 4] = ["tisící", "miliontý", "miliardtý", "biliontý"];

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

/// Cardinal words for a 0-999 segment: irregular hundreds ("dvě stě",
/// "tři sta"), then teens or tens and ones. The JavaScript source has two
/// builders for this (one meant for the units segment); their code is
/// identical, so one serves both.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<&str> = Vec::with_capacity(3);
    if hundreds > 0 {
        parts.push(HUNDREDS[hundreds]);
    }
    if tens == 1 {
        // Teens
        parts.push(TEENS[ones]);
    } else if tens >= 2 {
        parts.push(TENS[tens]);
        if ones > 0 {
            parts.push(ONES[ones]);
        }
    } else if ones > 0 {
        parts.push(ONES[ones]);
    }
    parts.join(" ")
}

/// The decimal separator agrees with the integer part: "celá" after 0 and
/// 1, "celé" after 2-4, "celých" otherwise.
fn decimal_separator(integer: &BigUint) -> &'static str {
    match integer.to_u64() {
        Some(0 | 1) => "celá",
        Some(2..=4) => "celé",
        _ => "celých",
    }
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

        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            let scale_word = pluralize(thousands, &SCALE_FORMS[0]);
            let mut result = if thousands == 1 {
                // Omit "one" before tisíc
                scale_word.to_string()
            } else {
                format!("{} {scale_word}", build_segment(thousands))
            };
            if remainder > 0 {
                result.push(' ');
                result.push_str(&build_segment(remainder));
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
            result.push_str(&build_segment(segment));
        } else {
            let scale_word = pluralize(segment, &SCALE_FORMS[i - 1]);
            if segment == 1 {
                // Omit "one" before scale words
                result.push_str(scale_word);
            } else {
                result.push_str(&build_segment(segment));
                result.push(' ');
                result.push_str(scale_word);
            }
        }
    }
    result
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

/// Spells a number in Czech words.
///
/// ```
/// # use n2words::cs_cz::to_cardinal;
/// assert_eq!(to_cardinal(2000)?, "dva tisíce");
/// assert_eq!(to_cardinal(1.7)?, "jedna celá sedm");
/// assert_eq!(to_cardinal(17.42)?, "sedmnáct celých čtyřicet dva");
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
        result.push_str(decimal_separator(&number.integer));
        result.push(' ');
        result.push_str(&decimal_part_to_words(fraction));
    }
    Ok(result)
}

// ============================================================================
// Ordinal
// ============================================================================

/// Ordinal words for a 0-99 final segment: "první", "dvacet první".
fn build_ordinal_tens_ones(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = (n / 10) as usize;

    if tens == 0 {
        return ORDINAL_ONES[ones].to_string();
    }
    if tens == 1 {
        return ORDINAL_TEENS[ones].to_string();
    }
    if ones == 0 {
        return ORDINAL_TENS[tens].to_string();
    }
    // Compound: cardinal tens, ordinal ones
    format!("{} {}", TENS[tens], ORDINAL_ONES[ones])
}

/// Ordinal words for a 0-999 final segment.
fn build_ordinal_segment(n: u32) -> String {
    if n < 100 {
        return build_ordinal_tens_ones(n);
    }
    let hundreds = (n / 100) as usize;
    let remainder = n % 100;
    if remainder == 0 {
        return ORDINAL_HUNDREDS[hundreds].to_string();
    }
    format!(
        "{} {}",
        HUNDREDS[hundreds],
        build_ordinal_tens_ones(remainder)
    )
}

/// Ordinal words for a positive integer (masculine nominative). Only the
/// last component becomes ordinal.
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_ordinal_segment(small as u32);
        }

        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            if remainder == 0 {
                if thousands == 1 {
                    return ORDINAL_SCALES[0].to_string();
                }
                return format!("{} {}", build_segment(thousands), ORDINAL_SCALES[0]);
            }

            // Cardinal thousands (no "one" before tisíc) + ordinal remainder
            let scale_word = pluralize(thousands, &SCALE_FORMS[0]);
            let thousands_word = if thousands == 1 {
                String::new()
            } else {
                format!("{} ", build_segment(thousands))
            };
            return format!(
                "{thousands_word}{scale_word} {}",
                build_ordinal_segment(remainder)
            );
        }
    }

    build_large_ordinal(n)
}

/// The ordinal scale word for scale position `i` (1 = thousands); past the
/// ordinal table the singular cardinal scale word stands in.
fn ordinal_scale(i: usize) -> &'static str {
    ORDINAL_SCALES
        .get(i - 1)
        .copied()
        .unwrap_or(SCALE_FORMS[i - 1][0])
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
                result.push_str(ordinal_scale(i));
            } else {
                result.push_str(&build_segment(segment));
                result.push(' ');
                result.push_str(ordinal_scale(i));
            }
        } else {
            let scale_word = pluralize(segment, &SCALE_FORMS[i - 1]);
            if segment == 1 {
                result.push_str(scale_word);
            } else {
                result.push_str(&build_segment(segment));
                result.push(' ');
                result.push_str(scale_word);
            }
        }
    }
    result
}

/// Spells a positive integer as a Czech ordinal (masculine nominative).
///
/// ```
/// # use n2words::cs_cz::to_ordinal;
/// assert_eq!(to_ordinal(21)?, "dvacet první");
/// assert_eq!(to_ordinal(100)?, "stý");
/// assert_eq!(to_ordinal(1000)?, "tisící");
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
    // Ordinals reuse the cardinal scale table, so they share its ceiling.
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
        currency: Currency = Currency::CZK, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Czech koruna.
///
/// ```
/// # use n2words::cs_cz::to_currency;
/// assert_eq!(to_currency(1)?, "jedna koruna");
/// assert_eq!(to_currency(1.5)?, "jedna koruna padesát haléřů");
/// assert_eq!(to_currency(-5)?, "mínus pět korun");
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

/// Spells an amount in any currency Czech has words for.
///
/// ```
/// # use n2words::{cs_cz::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::CZK };
/// assert_eq!(to_currency_with(0.99, &opts)?, "devadesát devět haléřů");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Czech has no words for and
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
    // Koruna part
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(plural_index_big(&money.major)));
    }
    // Haléř part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(plural_index(money.minor)));
    }
    Ok(result)
}

/// Registry entry for `cs-CZ`.
pub static LANGUAGE: Language = Language {
    code: "cs-CZ",
    name: "Czech (Czechia)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
