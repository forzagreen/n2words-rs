//! Spanish (United States) (`es-US`).
//!
//! Uses the short scale numbering system (like US English):
//!
//! - 10^6 = millón
//! - 10^9 = billón (billion)
//! - 10^12 = trillón (trillion)
//!
//! Spanish-specific rules:
//!
//! - Gender agreement: uno/una, veintiuno/veintiuna, hundreds
//! - Special twenties: veinte, veintiuno, veintidós, ... veintinueve
//! - "y" conjunction: treinta y uno (only 30-99 with ones)
//! - "cien" for exact 100, "ciento" otherwise ("ciento" is invariable; only
//!   200-900 agree)
//! - Irregular hundreds: quinientos, setecientos, novecientos
//! - Apocopation: "uno" shortens to "un" before a masculine noun, and a scale
//!   word or currency noun is exactly that: "veintiún mil", "treinta y un
//!   millones", "veintiún dólares". A multiplier of exactly 1 is dropped
//!   before "mil" ("mil", not "un mil") but kept before "millón" ("un millón").
//!
//! The 0-999 segment builder, the vocabulary and the ordinal (identical
//! across Spanish regions) are shared with [`es_es`](crate::es_es); only the
//! scale grouping and the default currency differ.
//!
//! ```
//! use n2words::es_us::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(1_000_000_000)?, "un billón");
//! assert_eq!(to_ordinal(21)?, "vigesimoprimero");
//! assert_eq!(to_currency(42.5)?, "cuarenta y dos dólares con cincuenta centavos");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::ES as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::es_es::{
    CURRENCY_CONNECTOR, DECIMAL_SEP, NEGATIVE, ZERO, build_segment, is_round_millions,
};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, digits_to_biguint, parse_cardinal, parse_currency, split_groups};
use crate::options::{Gender, options};
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

// Ordinal vocabulary and builders are identical across Spanish regions (as
// the RAE writes them, Ortografía 2010, DPD "ordinales"), and so is the
// ordinal ceiling (`10^9`: the millions multiplier is a single fused segment,
// "dosmillonésimo").
pub use crate::es_es::{ORDINAL_MAX, OrdinalOptions, to_ordinal, to_ordinal_with};

// ============================================================================
// Vocabulary
// ============================================================================

// Scale words (short scale: each scale is 10^3 apart)
const SCALES: &[&str] = &[
    "mil",
    "millón",
    "billón",
    "trillón",
    "cuatrillón",
    "quintillón",
];
const SCALES_PLURAL: &[&str] = &[
    "mil",
    "millones",
    "billones",
    "trillones",
    "cuatrillones",
    "quintillones",
];

/// The cardinal ceiling, derived from the scale table: `10^21`.
///
/// Short scale: `SCALES` covers 10^3..10^18, plus the units group, so a
/// cardinal spans at most `SCALES.len() + 1` groups of 3 digits.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32);
/// The currency ceiling: `10^21`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32);

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer (short scale).
///
/// `before_noun` means the whole number is followed by a noun (a currency
/// name), so its trailing segment apocopates: "veintiún dólares". A
/// standalone numeral doesn't: `to_cardinal(21)` is "veintiuno".
fn integer_to_words(n: &BigUint, feminine: bool, before_noun: bool) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Fast path: numbers < 1000
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32, feminine, before_noun);
        }
    }

    // Each segment is 3 digits; the short scale increments by 10^3.
    // Callers guard the magnitude (CARDINAL_MAX) so the scale index stays in range.
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
            // Units segment: the requested gender
            result.push_str(&build_segment(segment, feminine, before_noun));
        } else if i == 1 {
            // Thousands: "mil" not "un mil". Otherwise the multiplier treats
            // "mil" as the masculine noun it is: masculine forms whatever
            // gender was requested, and apocopated: "veintiún mil",
            // "treinta y un mil".
            if segment != 1 {
                result.push_str(&build_segment(segment, false, true));
                result.push(' ');
            }
            result.push_str(SCALES[0]);
        } else {
            // Millions and above: "un millón", "veintiún millones", "ciento un
            // millones". The multiplier is kept and apocopated.
            let scale_index = i - 1; // SCALES[1] = millón, SCALES[2] = billón, ...
            result.push_str(&build_segment(segment, false, true));
            result.push(' ');
            result.push_str(if segment == 1 {
                SCALES[scale_index]
            } else {
                SCALES_PLURAL[scale_index]
            });
        }
    }

    result
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
fn decimal_part_to_words(decimal: &str, feminine: bool) -> String {
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
        result.push_str(&integer_to_words(
            &digits_to_biguint(remainder),
            feminine,
            false,
        ));
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

/// Spells a number in Spanish words (US short scale).
///
/// ```
/// # use n2words::es_us::to_cardinal;
/// assert_eq!(to_cardinal(21)?, "veintiuno");
/// assert_eq!(to_cardinal(2_000_000_000)?, "dos billones");
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

/// Spells a number in Spanish words (US short scale), with options.
///
/// ```
/// # use n2words::{es_us::{to_cardinal_with, CardinalOptions}, Gender};
/// let feminine = CardinalOptions { gender: Gender::Feminine };
/// assert_eq!(to_cardinal_with(21, &feminine)?, "veintiuna");
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

    let feminine = options.gender == Gender::Feminine;

    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }
    result.push_str(&integer_to_words(&number.integer, feminine, false));
    if let Some(fraction) = &number.fraction {
        result.push(' ');
        result.push_str(DECIMAL_SEP);
        result.push(' ');
        result.push_str(&decimal_part_to_words(fraction, feminine));
    }
    Ok(result)
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// Use "con" between dollars and cents
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::USD, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in US dollars.
///
/// US Dollar uses masculine gender for dólares (el dólar) and masculine for
/// centavos (el centavo).
///
/// ```
/// # use n2words::es_us::to_currency;
/// assert_eq!(to_currency(1)?, "un dólar");
/// assert_eq!(to_currency(0.99)?, "noventa y nueve centavos");
/// assert_eq!(to_currency(1_000_000_000)?, "un billón de dólares");
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

/// Spells an amount in any currency Spanish has words for.
///
/// ```
/// # use n2words::{es_us::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::MXN, and: false };
/// assert_eq!(to_currency_with(42.5, &opts)?, "cuarenta y dos pesos cincuenta centavos");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Spanish has no words for and
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

    // Every Spanish entry carries both genders. Every currency named today is
    // masculine (euro, dólar, peso), but a future feminine one (e.g. libra)
    // must not silently get "un libra".
    let major_feminine = words.major_gender == Some(Gender::Feminine);
    let minor_feminine = words.minor_gender == Some(Gender::Feminine);

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }

    // The major-unit noun's own gender drives both the "un"/"una" article and
    // the numeral itself.
    if !money.major.is_zero() || money.minor == 0 {
        if money.major.is_one() {
            result.push_str(if major_feminine { "una " } else { "un " });
            result.push_str(words.major_at(0));
        } else {
            // Millón and its multiples are nouns, so a round amount of them
            // takes "de" before what it counts: "un millón de dólares".
            let preposition = if is_round_millions(&money.major) {
                " de "
            } else {
                " "
            };
            result.push_str(&integer_to_words(&money.major, major_feminine, true));
            result.push_str(preposition);
            result.push_str(words.major_at(1));
        }
    }

    // The minor-unit noun's own gender.
    if money.minor > 0 {
        if !money.major.is_zero() {
            if options.and {
                result.push(' ');
                result.push_str(CURRENCY_CONNECTOR);
            }
            result.push(' ');
        }
        if money.minor == 1 {
            result.push_str(if minor_feminine { "una " } else { "un " });
            result.push_str(words.minor_at(0));
        } else {
            result.push_str(&integer_to_words(
                &BigUint::from(money.minor),
                minor_feminine,
                true,
            ));
            result.push(' ');
            result.push_str(words.minor_at(1));
        }
    }

    Ok(result)
}

/// Registry entry for `es-US`.
pub static LANGUAGE: Language = Language {
    code: "es-US",
    name: "Spanish (United States)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(with_options to_ordinal_with, OrdinalOptions, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
