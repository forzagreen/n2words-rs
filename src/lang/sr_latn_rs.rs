//! Serbian (Latin, Serbia) (`sr-Latn-RS`).
//!
//! - Three-form pluralization (one/few/many)
//! - Gender by scale word: the -arda forms (hiljada, milijarda, bilijarda, ...)
//!   are feminine; the -ion forms (milion, bilion, ...) are masculine
//! - Irregular hundreds (dvesta, trista, etc.)
//! - Long scale naming with -ard forms
//! - Latin script
//!
//! ```
//! use n2words::sr_latn_rs::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "četrdeset dva");
//! assert_eq!(to_ordinal(42)?, "četrdeset drugi");
//! assert_eq!(to_currency(42.5)?, "četrdeset dva dinara i pedeset para");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::SR_LATN as CURRENCY_VOCAB;
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
    "", "jedan", "dva", "tri", "četiri", "pet", "šest", "sedam", "osam", "devet",
];
const ONES_FEM: [&str; 10] = [
    "", "jedna", "dve", "tri", "četiri", "pet", "šest", "sedam", "osam", "devet",
];
const TEENS: [&str; 10] = [
    "deset",
    "jedanaest",
    "dvanaest",
    "trinaest",
    "četrnaest",
    "petnaest",
    "šesnaest",
    "sedamnaest",
    "osamnaest",
    "devetnaest",
];
const TENS: [&str; 10] = [
    "",
    "",
    "dvadeset",
    "trideset",
    "četrdeset",
    "pedeset",
    "šezdeset",
    "sedamdeset",
    "osamdeset",
    "devedeset",
];
const HUNDREDS: [&str; 10] = [
    "",
    "sto",
    "dvesta",
    "trista",
    "četiristo",
    "petsto",
    "šesto",
    "sedamsto",
    "osamsto",
    "devetsto",
];

const ZERO: &str = "nula";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "zapeta";
const AND: &str = "i";

// ============================================================================
// Ordinal Vocabulary (masculine nominative)
// ============================================================================

// Ordinal ones: prvi, drugi, treći...
const ORDINAL_ONES: [&str; 10] = [
    "", "prvi", "drugi", "treći", "četvrti", "peti", "šesti", "sedmi", "osmi", "deveti",
];

// Ordinal teens: deseti, jedanaesti...
const ORDINAL_TEENS: [&str; 10] = [
    "deseti",
    "jedanaesti",
    "dvanaesti",
    "trinaesti",
    "četrnaesti",
    "petnaesti",
    "šesnaesti",
    "sedamnaesti",
    "osamnaesti",
    "devetnaesti",
];

// Ordinal tens: dvadeseti, trideseti...
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "dvadeseti",
    "trideseti",
    "četrdeseti",
    "pedeseti",
    "šezdeseti",
    "sedamdeseti",
    "osamdeseti",
    "devedeseti",
];

// Ordinal hundreds: stoti, dvestoti...
const ORDINAL_HUNDREDS: [&str; 10] = [
    "",
    "stoti",
    "dvestoti",
    "tristoti",
    "četiristoti",
    "petstoti",
    "šestoti",
    "sedamstoti",
    "osamstoti",
    "devetstoti",
];

// Ordinal scale words (hiljaditi, milioniti, etc.)
const ORDINAL_SCALES: &[&str] = &[
    "hiljaditi",
    "milioniti",
    "milijarditi",
    "bilioniti",
    "bilijarditi",
    "trilioniti",
    "trilijarditi",
    "kvadrilioniti",
    "kvadrilijarditi",
];

// ============================================================================
// Scale Vocabulary
// ============================================================================

// Scale words: [singular, few, many]
const SCALE_FORMS: &[[&str; 3]] = &[
    ["hiljada", "hiljade", "hiljada"],
    ["milion", "miliona", "miliona"],
    ["milijarda", "milijarde", "milijarda"],
    ["bilion", "biliona", "biliona"],
    ["bilijarda", "bilijarde", "bilijarda"],
    ["trilion", "triliona", "triliona"],
    ["trilijarda", "trilijarde", "trilijarda"],
    ["kvadrilion", "kvadriliona", "kvadriliona"],
    ["kvadrilijarda", "kvadrilijarde", "kvadrilijarda"],
];

// Supported magnitude ceilings (checked at the public entry points). Both the
// cardinal SCALE_FORMS and the ORDINAL_SCALES tables cover units + N scale
// groups, so values must stay below 10^((length + 1) * 3) = 10^30.

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_FORMS.len() as u32);
/// The ordinal ceiling, derived from the ordinal scale table: `10^30`.
pub const ORDINAL_MAX: Max = Max::western(ORDINAL_SCALES.len() as u32);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_FORMS.len() as u32);

// ============================================================================
// Segment Building
// ============================================================================

/// Index of the plural form for a count: 0 (one), 1 (few, 2-4) or 2 (many).
///
/// Only the last two digits matter: 11-19 are always "many", otherwise the
/// last digit decides.
fn plural_index(n: u32) -> usize {
    let last_digit = n % 10;
    let last_two_digits = n % 100;

    if (11..=19).contains(&last_two_digits) {
        return 2;
    }

    if last_digit == 1 {
        return 0;
    }
    if (2..=4).contains(&last_digit) {
        return 1;
    }
    2
}

/// The last two digits of a count, enough for [`plural_index`].
fn last_two_digits(n: &BigUint) -> u32 {
    (n % 100u32).to_u32().unwrap_or(0)
}

/// Gender of the scale word at `scale_index`: the -arda forms (hiljada,
/// milijarda, ...) are feminine, the -ion forms masculine.
fn scale_gender(scale_index: usize) -> Gender {
    if scale_index % 2 == 1 {
        Gender::Feminine
    } else {
        Gender::Masculine
    }
}

/// Words for a 0-999 segment in the given gender (only the ones differ).
fn build_segment(n: u32, gender: Gender) -> String {
    if n == 0 {
        return String::new();
    }

    let ones_digit = (n % 10) as usize;
    let tens_digit = ((n / 10) % 10) as usize;
    let hundreds_digit = (n / 100) as usize;
    let ones = match gender {
        Gender::Masculine => &ONES_MASC,
        Gender::Feminine => &ONES_FEM,
    };

    let mut parts: Vec<&str> = Vec::with_capacity(3);

    if hundreds_digit > 0 {
        parts.push(HUNDREDS[hundreds_digit]);
    }

    if tens_digit > 1 {
        parts.push(TENS[tens_digit]);
    }

    if tens_digit == 1 {
        parts.push(TEENS[ones_digit]);
    } else if ones_digit > 0 {
        parts.push(ones[ones_digit]);
    }

    parts.join(" ")
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint, gender: Gender) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32, gender);
        }
    }

    build_large_number_words(n, gender)
}

/// Words for 1000 or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint, gender: Gender) -> String {
    let segments = split_groups(n, 1000);
    let mut parts: Vec<String> = Vec::with_capacity(segments.len());

    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if scale_index == 0 {
            parts.push(build_segment(segment, gender));
        } else {
            let scale_word = SCALE_FORMS[scale_index - 1][plural_index(segment)];
            // -arda scale words (hiljada, milijarda, ...) are feminine; -ion words masculine
            let segment_word = build_segment(segment, scale_gender(scale_index));
            parts.push(format!("{segment_word} {scale_word}"));
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
        /// Grammatical gender
        gender: Gender = Gender::Masculine;
    }
}

/// Spells a number in Serbian (Latin) words, in the masculine.
///
/// ```
/// # use n2words::sr_latn_rs::to_cardinal;
/// assert_eq!(to_cardinal(2385)?, "dve hiljade trista osamdeset pet");
/// assert_eq!(to_cardinal(-17.42)?, "minus sedamnaest zapeta četrdeset dva");
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

/// Spells a number in Serbian (Latin) words, with options.
///
/// ```
/// # use n2words::{sr_latn_rs::{to_cardinal_with, CardinalOptions}, Gender};
/// let opts = CardinalOptions { gender: Gender::Feminine };
/// assert_eq!(to_cardinal_with(2, &opts)?, "dve");
/// assert_eq!(to_cardinal_with(1001, &opts)?, "jedna hiljada jedna");
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

/// Ordinal words for a 0-99 segment when it is the final (ordinal) part:
/// "prvi", "dvadeset prvi", etc.
fn build_ordinal_tens_ones(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones_digit = (n % 10) as usize;
    let tens_digit = (n / 10) as usize;

    if tens_digit == 0 {
        // Single digit: prvi, drugi, etc.
        return ORDINAL_ONES[ones_digit].to_string();
    }

    if tens_digit == 1 {
        // Teens: deseti, jedanaesti, etc.
        return ORDINAL_TEENS[ones_digit].to_string();
    }

    // Tens >= 20
    if ones_digit == 0 {
        // Round tens: dvadeseti, trideseti, etc.
        return ORDINAL_TENS[tens_digit].to_string();
    }

    // Compound: dvadeset prvi, trideset drugi, etc.
    format!("{} {}", TENS[tens_digit], ORDINAL_ONES[ones_digit])
}

/// Ordinal words for a 1-999 segment when it is the final (ordinal) part:
/// exact hundreds have their own form (stoti, dvestoti, ...), otherwise
/// cardinal hundreds precede the ordinal remainder.
fn build_ordinal_segment(n: u32) -> String {
    if n < 100 {
        return build_ordinal_tens_ones(n);
    }

    let hundreds_digit = (n / 100) as usize;
    let remainder = n % 100;

    if remainder == 0 {
        return ORDINAL_HUNDREDS[hundreds_digit].to_string();
    }

    format!(
        "{} {}",
        HUNDREDS[hundreds_digit],
        build_ordinal_tens_ones(remainder)
    )
}

/// Ordinal words for a positive integer (masculine nominative).
///
/// In Serbian ordinals, only the LAST component becomes ordinal:
/// 121 = "sto dvadeset prvi".
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_ordinal_segment(small as u32);
        }

        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            if remainder == 0 {
                // Exact thousands: hiljaditi, etc.
                if thousands == 1 {
                    return ORDINAL_SCALES[0].to_string();
                }
                // Use cardinal segment + ordinal scale
                return format!(
                    "{} {}",
                    build_segment(thousands, Gender::Feminine),
                    ORDINAL_SCALES[0]
                );
            }

            // Has remainder: cardinal thousands + ordinal remainder
            let thousands_word = build_segment(thousands, Gender::Feminine);
            let scale_word = SCALE_FORMS[0][plural_index(thousands)];
            return format!(
                "{thousands_word} {scale_word} {}",
                build_ordinal_segment(remainder)
            );
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
            // This scale position is the final ordinal
            if segment == 1 {
                parts.push(ORDINAL_SCALES[scale_index - 1].to_string());
            } else {
                // feminine at -arda scales (hiljada, milijarda, ...)
                let segment_word = build_segment(segment, scale_gender(scale_index));
                parts.push(format!(
                    "{segment_word} {}",
                    ORDINAL_SCALES[scale_index - 1]
                ));
            }
        } else {
            // Not the final segment: use cardinal
            let scale_word = SCALE_FORMS[scale_index - 1][plural_index(segment)];
            let segment_word = build_segment(segment, scale_gender(scale_index));
            parts.push(format!("{segment_word} {scale_word}"));
        }
    }

    parts.join(" ")
}

/// Spells a positive integer as a Serbian (Latin) ordinal (masculine
/// nominative).
///
/// ```
/// # use n2words::sr_latn_rs::to_ordinal;
/// assert_eq!(to_ordinal(21)?, "dvadeset prvi");
/// assert_eq!(to_ordinal(100)?, "stoti");
/// assert_eq!(to_ordinal(1000)?, "hiljaditi");
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
        /// Use "i" between dinars and para
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::RSD, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Serbian dinars.
///
/// ```
/// # use n2words::sr_latn_rs::to_currency;
/// assert_eq!(to_currency(1)?, "jedan dinar");
/// assert_eq!(to_currency(0.99)?, "devedeset devet para");
/// assert_eq!(to_currency(0.01)?, "jedna para");
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

/// Spells an amount in any currency Serbian (Latin) has words for.
///
/// ```
/// # use n2words::{sr_latn_rs::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { and: false, currency: Currency::RSD };
/// assert_eq!(to_currency_with(42.5, &opts)?, "četrdeset dva dinara pedeset para");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Serbian has no words for and
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
    // Every sr entry sets both gender fields; the fallback only guards the type.
    let major_gender = words.major_gender.unwrap_or(Gender::Masculine);
    let minor_gender = words.minor_gender.unwrap_or(Gender::Feminine);

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }

    // Major-unit noun's own gender drives agreement.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major, major_gender));
        result.push(' ');
        result.push_str(words.major_at(plural_index(last_two_digits(&money.major))));
    }

    // Minor-unit noun's own gender.
    if money.minor > 0 {
        if !money.major.is_zero() {
            if options.and {
                result.push(' ');
                result.push_str(AND);
            }
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor), minor_gender));
        result.push(' ');
        result.push_str(words.minor_at(plural_index(money.minor)));
    }

    Ok(result)
}

/// Registry entry for `sr-Latn-RS`.
pub static LANGUAGE: Language = Language {
    code: "sr-Latn-RS",
    name: "Serbian (Latin, Serbia)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
