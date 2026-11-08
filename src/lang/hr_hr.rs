//! Croatian (Croatia) (`hr-HR`).
//!
//! - Three-form pluralization (one/few/many)
//! - Gender by scale word: tisuća and the -arda forms (milijarda, bilijarda, ...)
//!   are feminine; the -un forms (milijun, bilijun, ...) are masculine
//! - Irregular hundreds (dvjesto, tristo, etc.)
//! - Long scale naming with -ard forms
//!
//! ```
//! use n2words::hr_hr::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "četrdeset dva");
//! assert_eq!(to_ordinal(42)?, "četrdeset drugi");
//! assert_eq!(to_currency(42.5)?, "četrdeset dva eura pedeset centi");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::HR as CURRENCY_VOCAB;
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
    "", "jedna", "dvije", "tri", "četiri", "pet", "šest", "sedam", "osam", "devet",
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

// Croatian has irregular hundreds
const HUNDREDS: [&str; 10] = [
    "",
    "sto",
    "dvjesto",
    "tristo",
    "četiristo",
    "petsto",
    "šesto",
    "sedamsto",
    "osamsto",
    "devetsto",
];

const ZERO: &str = "nula";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "zarez";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

// Croatian ordinals (masculine nominative singular)
const ORDINAL_ONES: [&str; 10] = [
    "", "prvi", "drugi", "treći", "četvrti", "peti", "šesti", "sedmi", "osmi", "deveti",
];

// Ordinal teens
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

// Ordinal tens (for exact tens)
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

// Ordinal hundreds (for exact hundreds)
const ORDINAL_HUNDREDS: [&str; 10] = [
    "",
    "stoti",
    "dvjestoti",
    "tristoti",
    "četiristoti",
    "petstoti",
    "šeststoti",
    "sedamstoti",
    "osamstoti",
    "devetstoti",
];

// Scale ordinals
const ORDINAL_SCALES: &[&str] = &["tisućiti", "milijunti", "milijarditi", "bilijunti"];

// ============================================================================
// Scale Vocabulary
// ============================================================================

// Scale words: [singular, few, many]
// Indexed [scaleIndex - 1]; odd scales (tisuća, milijarda, bilijarda, ...) are
// feminine, the -un forms (milijun, bilijun, ...) masculine
const SCALE_FORMS: &[[&str; 3]] = &[
    ["tisuća", "tisuće", "tisuća"],
    ["milijun", "milijuna", "milijuna"],
    ["milijarda", "milijarde", "milijarda"],
    ["bilijun", "bilijuna", "bilijuna"],
    ["bilijarda", "bilijarde", "bilijarda"],
    ["trilijun", "trilijuna", "trilijuna"],
    ["trilijarda", "trilijarde", "trilijarda"],
    ["kvadrilijun", "kvadrilijuna", "kvadrilijuna"],
    ["kvadrilijarda", "kvadrilijarde", "kvadrilijarda"],
];

// Supported magnitude ceilings (checked at the public entry points). Both
// tables are indexed [scaleIndex - 1] (units separate), so the ceiling is
// 10^((length + 1) * 3): cardinal/currency 10^30, ordinal 10^15.

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_FORMS.len() as u32);
/// The ordinal ceiling, derived from the ordinal scale table: `10^15`.
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

/// Gender of the scale word at `scale_index`: tisuća and the -arda forms
/// (milijarda, bilijarda, ...) are feminine, the -un forms masculine.
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
            // tisuća and the -arda forms (milijarda, bilijarda, ...) are feminine; -un words masculine
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

/// Spells a number in Croatian words, in the masculine.
///
/// ```
/// # use n2words::hr_hr::to_cardinal;
/// assert_eq!(to_cardinal(2385)?, "dvije tisuće tristo osamdeset pet");
/// assert_eq!(to_cardinal(-17.42)?, "minus sedamnaest zarez četrdeset dva");
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

/// Spells a number in Croatian words, with options.
///
/// ```
/// # use n2words::{hr_hr::{to_cardinal_with, CardinalOptions}, Gender};
/// let opts = CardinalOptions { gender: Gender::Feminine };
/// assert_eq!(to_cardinal_with(22, &opts)?, "dvadeset dvije");
/// assert_eq!(to_cardinal_with(2001, &opts)?, "dvije tisuće jedna");
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
/// exact hundreds have their own form, otherwise cardinal hundreds precede
/// the ordinal remainder.
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
                return format!(
                    "{} {}",
                    build_segment(thousands, Gender::Feminine),
                    ORDINAL_SCALES[0]
                );
            }

            let scale_word = SCALE_FORMS[0][plural_index(thousands)];
            let thousands_word = build_segment(thousands, Gender::Feminine);
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
            if segment == 1 {
                parts.push(ORDINAL_SCALES[scale_index - 1].to_string());
            } else {
                // feminine at odd scales (tisuća, milijarda, ...)
                let segment_word = build_segment(segment, scale_gender(scale_index));
                parts.push(format!(
                    "{segment_word} {}",
                    ORDINAL_SCALES[scale_index - 1]
                ));
            }
        } else {
            let scale_word = SCALE_FORMS[scale_index - 1][plural_index(segment)];
            let segment_word = build_segment(segment, scale_gender(scale_index));
            parts.push(format!("{segment_word} {scale_word}"));
        }
    }

    parts.join(" ")
}

/// Spells a positive integer as a Croatian ordinal (masculine nominative).
///
/// ```
/// # use n2words::hr_hr::to_ordinal;
/// assert_eq!(to_ordinal(21)?, "dvadeset prvi");
/// assert_eq!(to_ordinal(100)?, "stoti");
/// assert_eq!(to_ordinal(1000)?, "tisućiti");
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
/// # use n2words::hr_hr::to_currency;
/// assert_eq!(to_currency(1)?, "jedan euro");
/// assert_eq!(to_currency(1.5)?, "jedan euro pedeset centi");
/// assert_eq!(to_currency(-5)?, "minus pet eura");
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

/// Spells an amount in any currency Croatian has words for.
///
/// ```
/// # use n2words::{hr_hr::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::EUR };
/// assert_eq!(to_currency_with(2.02, &opts)?, "dva eura dva centa");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Croatian has no words for and
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
    // Every hr entry sets both gender fields; the fallback only guards the type.
    let major_gender = words.major_gender.unwrap_or(Gender::Masculine);
    let minor_gender = words.minor_gender.unwrap_or(Gender::Masculine);

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
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor), minor_gender));
        result.push(' ');
        result.push_str(words.minor_at(plural_index(money.minor)));
    }

    Ok(result)
}

/// Registry entry for `hr-HR`.
pub static LANGUAGE: Language = Language {
    code: "hr-HR",
    name: "Croatian (Croatia)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
