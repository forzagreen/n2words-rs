//! Latvian (Latvia) (`lv-LV`).
//!
//! - Two-form pluralization (singular for 1 except 11, plural otherwise)
//! - Gender agreement (masculine/feminine for numbers < 1000)
//! - Special hundreds forms (simts/simti/simtu)
//! - Omit "one" before scale words
//! - Long scale naming
//!
//! ```
//! use n2words::lv_lv::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "četrdesmit divi");
//! assert_eq!(to_ordinal(42)?, "četrdesmit otrais");
//! assert_eq!(to_currency(42.5)?, "četrdesmit divi eiro piecdesmit centu");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::LV as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{
    Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal, split_groups,
};
use crate::options::{Gender, options};
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

const ONES_MASC: [&str; 10] = [
    "", "viens", "divi", "trīs", "četri", "pieci", "seši", "septiņi", "astoņi", "deviņi",
];
const ONES_FEM: [&str; 10] = [
    "",
    "viena",
    "divas",
    "trīs",
    "četras",
    "piecas",
    "sešas",
    "septiņas",
    "astoņas",
    "deviņas",
];

const TEENS: [&str; 10] = [
    "desmit",
    "vienpadsmit",
    "divpadsmit",
    "trīspadsmit",
    "četrpadsmit",
    "piecpadsmit",
    "sešpadsmit",
    "septiņpadsmit",
    "astoņpadsmit",
    "deviņpadsmit",
];
const TENS: [&str; 10] = [
    "",
    "",
    "divdesmit",
    "trīsdesmit",
    "četrdesmit",
    "piecdesmit",
    "sešdesmit",
    "septiņdesmit",
    "astoņdesmit",
    "deviņdesmit",
];

// Hundreds: simts (100, 110-199), simti (200-999), simtu (101-109)
const HUNDRED_SINGULAR: &str = "simts";
const HUNDRED_PLURAL: &str = "simti";
const HUNDRED_GENITIVE: &str = "simtu";

const ZERO: &str = "nulle";
const NEGATIVE: &str = "mīnus";
const DECIMAL_SEP: &str = "komats";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

// Latvian ordinals (masculine nominative singular)
const ORDINAL_ONES: [&str; 10] = [
    "",
    "pirmais",
    "otrais",
    "trešais",
    "ceturtais",
    "piektais",
    "sestais",
    "septītais",
    "astotais",
    "devītais",
];

// Ordinal teens
const ORDINAL_TEENS: [&str; 10] = [
    "desmitais",
    "vienpadsmitais",
    "divpadsmitais",
    "trīspadsmitais",
    "četrpadsmitais",
    "piecpadsmitais",
    "sešpadsmitais",
    "septiņpadsmitais",
    "astoņpadsmitais",
    "deviņpadsmitais",
];

// Ordinal tens
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "divdesmitais",
    "trīsdesmitais",
    "četrdesmitais",
    "piecdesmitais",
    "sešdesmitais",
    "septiņdesmitais",
    "astoņdesmitais",
    "deviņdesmitais",
];

// Ordinal hundreds (complete forms with suffix)
const ORDINAL_HUNDREDS: [&str; 10] = [
    "",
    "simtais",
    "divsimtais",
    "trīssimtais",
    "četrsimtais",
    "piecsimtais",
    "sešsimtais",
    "septiņsimtais",
    "astoņsimtais",
    "deviņsimtais",
];

// Scale ordinals
const ORDINAL_SCALES: &[&str] = &["tūkstošais", "miljonais", "miljardais", "triljonais"];

// ============================================================================
// Scale Vocabulary
// ============================================================================

// Scale words: [singular, plural, genitive]
const SCALE_FORMS: &[[&str; 3]] = &[
    ["tūkstotis", "tūkstoši", "tūkstošu"],
    ["miljons", "miljoni", "miljonu"],
    ["miljards", "miljardi", "miljardu"],
    ["triljons", "triljoni", "triljonu"],
    ["kvadriljons", "kvadriljoni", "kvadriljonu"],
    ["kvintiljons", "kvintiljoni", "kvintiljonu"],
    ["sikstiljons", "sikstiljoni", "sikstiljonu"],
    ["septiljons", "septiljoni", "septiljonu"],
    ["oktiljons", "oktiljoni", "oktiljonu"],
];

// Supported magnitude ceilings (checked at the public entry points). Both
// tables are indexed [scaleIndex - 1] (units separate), so the cardinal/
// currency ceiling is 10^((SCALE_FORMS.length + 1) * 3) = 10^30, and the
// shorter ORDINAL_SCALES bounds ordinals at 10^((ORDINAL_SCALES.length + 1) *
// 3) = 10^15.

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_FORMS.len() as u32);
/// The ordinal ceiling, derived from the ordinal scale table: `10^15`.
pub const ORDINAL_MAX: Max = Max::western(ORDINAL_SCALES.len() as u32);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_FORMS.len() as u32);

// ============================================================================
// Segment Building
// ============================================================================

/// Words for a 0-999 segment in the given gender (only the ones differ; the
/// hundreds are always masculine).
///
/// Does NOT omit "one" before a scale word; that is handled at join time.
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

    let mut parts: Vec<&str> = Vec::with_capacity(4);

    // Hundreds - Latvian has special forms
    if hundreds > 0 {
        if hundreds == 1 && tens == 0 && ones > 0 {
            // 101-109: use genitive form "simtu"
            parts.push(HUNDRED_GENITIVE);
        } else if hundreds > 1 {
            // 200-999: use plural "simti"
            parts.push(ONES_MASC[hundreds]);
            parts.push(HUNDRED_PLURAL);
        } else {
            // 100, 110-199: use singular "simts"
            parts.push(HUNDRED_SINGULAR);
        }
    }

    // Tens
    if tens > 1 {
        parts.push(TENS[tens]);
    }

    // Teens or ones - feminine for ones only
    if tens == 1 {
        parts.push(TEENS[ones]);
    } else if ones > 0 {
        parts.push(ones_words[ones]);
    }

    parts.join(" ")
}

// ============================================================================
// Pluralization
// ============================================================================

/// Index of the plural form for a scale-word count, simpler than Slavic:
/// 0 (singular) for a count ending in 1 except 11, 2 (genitive) for zero,
/// 1 (plural) for everything else.
fn plural_index(n: u32) -> usize {
    if n == 0 {
        return 2;
    }

    let last_digit = n % 10;
    let last_two_digits = n % 100;

    if last_digit == 1 && last_two_digits != 11 {
        return 0;
    }

    1
}

/// Index of the plural form for a currency count: genitive (2) for zero,
/// 10-19 and multiples of 10, singular (0) for a count ending in 1, plural
/// (1) otherwise.
fn plural_index_currency(n: u32) -> usize {
    if n == 0 {
        return 2;
    }

    let last_digit = n % 10;
    let last_two_digits = n % 100;

    // 10-19 use genitive
    if (10..=19).contains(&last_two_digits) {
        return 2;
    }

    // Ends in 0 uses genitive
    if last_digit == 0 {
        return 2;
    }

    // Ends in 1 uses singular
    if last_digit == 1 {
        return 0;
    }

    // 2-9 uses plural
    1
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer; the gender applies below 1000 only.
fn integer_to_words(n: &BigUint, gender: Gender) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Fast path: numbers < 1000
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32, gender);
        }
    }

    // For numbers >= 1000 every segment is masculine.
    build_large_number_words(n)
}

/// Words for 1000 or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let mut parts: Vec<String> = Vec::with_capacity(segments.len());

    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let segment_word = build_segment(segment, Gender::Masculine);

        if scale_index == 0 {
            // Units segment - masculine (feminine doesn't apply when n >= 1000)
            parts.push(segment_word);
        } else {
            // Segment with scale word
            let scale_word = SCALE_FORMS[scale_index - 1][plural_index(segment)];

            // Latvian omits "one" before scale words
            if segment == 1 {
                parts.push(scale_word.to_string());
            } else {
                parts.push(format!("{segment_word} {scale_word}"));
            }
        }
    }

    parts.join(" ")
}

/// Words for the digits after the decimal point.
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

/// Spells a number in Latvian words, in the masculine.
///
/// ```
/// # use n2words::lv_lv::to_cardinal;
/// assert_eq!(to_cardinal(1000)?, "tūkstotis");
/// assert_eq!(to_cardinal(-17.42)?, "mīnus septiņpadsmit komats četrdesmit divi");
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

/// Spells a number in Latvian words, with options.
///
/// ```
/// # use n2words::{lv_lv::{to_cardinal_with, CardinalOptions}, Gender};
/// let opts = CardinalOptions { gender: Gender::Feminine };
/// assert_eq!(to_cardinal_with(1, &opts)?, "viena");
/// assert_eq!(to_cardinal_with(102, &opts)?, "simtu divas");
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
    // Both the integer part and the decimal's significant digits are spelled via
    // the scale builder, so both must clear the ceiling.
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

/// Ordinal words for a 0-99 segment when it is the final (ordinal) part.
fn build_ordinal_tens_ones(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones_digit = (n % 10) as usize;
    let tens_digit = (n / 10) as usize;

    if tens_digit == 0 {
        return ORDINAL_ONES[ones_digit].to_string();
    }

    if tens_digit == 1 {
        return ORDINAL_TEENS[ones_digit].to_string();
    }

    if ones_digit == 0 {
        return ORDINAL_TENS[tens_digit].to_string();
    }

    format!("{} {}", TENS[tens_digit], ORDINAL_ONES[ones_digit])
}

/// Ordinal words for a 1-999 segment when it is the final (ordinal) part:
/// exact hundreds have their own form, otherwise cardinal hundreds ("simts",
/// "divi simti", ...) precede the ordinal remainder.
fn build_ordinal_segment(n: u32) -> String {
    if n < 100 {
        return build_ordinal_tens_ones(n);
    }

    let hundreds_digit = (n / 100) as usize;
    let remainder = n % 100;

    if remainder == 0 {
        return ORDINAL_HUNDREDS[hundreds_digit].to_string();
    }

    // Cardinal hundreds + ordinal remainder
    let hundred_word = if hundreds_digit == 1 {
        HUNDRED_SINGULAR.to_string()
    } else {
        format!("{} {HUNDRED_PLURAL}", ONES_MASC[hundreds_digit])
    };
    format!("{hundred_word} {}", build_ordinal_tens_ones(remainder))
}

/// Ordinal words for a positive integer (masculine nominative).
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_ordinal_segment(small as u32);
        }

        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;
            let scale_word = SCALE_FORMS[0][plural_index(thousands)];

            if remainder == 0 {
                if thousands == 1 {
                    return ORDINAL_SCALES[0].to_string();
                }
                // For 2000+, include cardinal scale word before ordinal
                return format!(
                    "{} {scale_word} {}",
                    build_segment(thousands, Gender::Masculine),
                    ORDINAL_SCALES[0]
                );
            }

            // "one" is omitted before the scale word, as in the cardinal
            let thousands_word = if thousands == 1 {
                scale_word.to_string()
            } else {
                format!(
                    "{} {scale_word}",
                    build_segment(thousands, Gender::Masculine)
                )
            };
            return format!("{thousands_word} {}", build_ordinal_segment(remainder));
        }
    }

    build_large_ordinal(n)
}

/// Ordinal words for one million or more: every segment is cardinal except
/// the last non-zero one, which becomes ordinal.
fn build_large_ordinal(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let lowest_non_zero = segments.iter().position(|&s| s != 0).unwrap_or(0);
    let mut parts: Vec<String> = Vec::with_capacity(segments.len());

    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let is_last_non_zero = scale_index == lowest_non_zero;

        if scale_index == 0 {
            // A non-zero units segment is always the last non-zero one.
            parts.push(build_ordinal_segment(segment));
        } else if is_last_non_zero {
            if segment == 1 {
                parts.push(ORDINAL_SCALES[scale_index - 1].to_string());
            } else {
                // For 2+, include cardinal scale word before ordinal
                let cardinal_scale_word = SCALE_FORMS[scale_index - 1][plural_index(segment)];
                parts.push(format!(
                    "{} {cardinal_scale_word} {}",
                    build_segment(segment, Gender::Masculine),
                    ORDINAL_SCALES[scale_index - 1]
                ));
            }
        } else {
            let scale_word = SCALE_FORMS[scale_index - 1][plural_index(segment)];
            if segment == 1 {
                parts.push(scale_word.to_string());
            } else {
                parts.push(format!(
                    "{} {scale_word}",
                    build_segment(segment, Gender::Masculine)
                ));
            }
        }
    }

    parts.join(" ")
}

/// Spells a positive integer as a Latvian ordinal (masculine nominative).
///
/// ```
/// # use n2words::lv_lv::to_ordinal;
/// assert_eq!(to_ordinal(21)?, "divdesmit pirmais");
/// assert_eq!(to_ordinal(100)?, "simtais");
/// assert_eq!(to_ordinal(1000)?, "tūkstošais");
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
        currency: Currency = Currency::EUR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in euros.
///
/// ```
/// # use n2words::lv_lv::to_currency;
/// assert_eq!(to_currency(1)?, "viens eiro");
/// assert_eq!(to_currency(1.5)?, "viens eiro piecdesmit centu");
/// assert_eq!(to_currency(-5)?, "mīnus pieci eiro");
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

/// Spells an amount in any currency Latvian has words for.
///
/// ```
/// # use n2words::{lv_lv::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::EUR };
/// assert_eq!(to_currency_with(2.02, &opts)?, "divi eiro divi centi");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Latvian has no words for and
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
    // Every lv entry sets both gender fields; the fallback only guards the type.
    let major_gender = words.major_gender.unwrap_or(Gender::Masculine);
    let minor_gender = words.minor_gender.unwrap_or(Gender::Masculine);

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }

    // Euro part (eiro is indeclinable, hence the single major form). Gender
    // still drives the numeral itself.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major, major_gender));
        result.push(' ');
        result.push_str(words.major_at(0));
    }

    // Minor-unit noun's own gender.
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor), minor_gender));
        result.push(' ');
        result.push_str(words.minor_at(plural_index_currency(money.minor)));
    }

    Ok(result)
}

/// Registry entry for `lv-LV`.
pub static LANGUAGE: Language = Language {
    code: "lv-LV",
    name: "Latvian (Latvia)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
