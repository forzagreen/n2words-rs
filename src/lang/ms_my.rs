//! Malay (Malaysia) (`ms-MY`).
//!
//! - "Se-" prefix for ALL singular scale units (seratus, seribu, sejuta, sebilion)
//! - Regular patterns (puluh for tens, ratus for hundreds)
//! - Teens with "belas" suffix
//! - Note: "lapan" (8) differs from Indonesian "delapan"
//!
//! ```
//! use n2words::ms_my::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(1234)?, "seribu dua ratus tiga puluh empat");
//! assert_eq!(to_ordinal(8)?, "kelapan");
//! assert_eq!(to_currency(42.5)?, "empat puluh dua ringgit lima puluh sen");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::MS as CURRENCY_VOCAB;
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

const ONES: [&str; 10] = [
    "", "satu", "dua", "tiga", "empat", "lima", "enam", "tujuh", "lapan", "sembilan",
];
const TEENS: [&str; 10] = [
    "sepuluh",
    "sebelas",
    "dua belas",
    "tiga belas",
    "empat belas",
    "lima belas",
    "enam belas",
    "tujuh belas",
    "lapan belas",
    "sembilan belas",
];
const TENS: [&str; 10] = [
    "",
    "",
    "dua puluh",
    "tiga puluh",
    "empat puluh",
    "lima puluh",
    "enam puluh",
    "tujuh puluh",
    "lapan puluh",
    "sembilan puluh",
];

const HUNDRED_WORD: &str = "ratus";
const THOUSAND_WORD: &str = "ribu";
const SCALE_WORDS: &[&str] = &["juta", "bilion", "trilion"];

/// The cardinal ceiling, derived from the scale table (plus "ribu"): `10^15`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 + 1);
/// The ordinal ceiling: `10^15`.
pub const ORDINAL_MAX: Max = Max::western(SCALE_WORDS.len() as u32 + 1);
/// The currency ceiling: `10^15`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_WORDS.len() as u32 + 1);

const ZERO: &str = "sifar";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "perpuluhan";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

const ORDINAL_PREFIX: &str = "ke";
// First is special: "pertama" (not "kesatu")
const ORDINAL_FIRST: &str = "pertama";

// ============================================================================
// Segment Building
// ============================================================================

/// Words for a 0-999 segment.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones_digit = (n % 10) as usize;
    let tens_digit = ((n / 10) % 10) as usize;
    let hundreds_digit = (n / 100) as usize;

    let mut parts: Vec<String> = Vec::new();

    // Hundreds: seratus (100) or N ratus (200-900)
    if hundreds_digit > 0 {
        if hundreds_digit == 1 {
            parts.push(format!("se{HUNDRED_WORD}"));
        } else {
            parts.push(format!("{} {HUNDRED_WORD}", ONES[hundreds_digit]));
        }
    }

    // Tens and ones
    let tens_ones = (n % 100) as usize;

    if tens_ones == 0 {
        // Just hundreds
    } else if tens_ones < 10 {
        parts.push(ONES[tens_ones].to_string());
    } else if tens_ones < 20 {
        parts.push(TEENS[tens_ones - 10].to_string());
    } else if ones_digit == 0 {
        parts.push(TENS[tens_digit].to_string());
    } else {
        parts.push(format!("{} {}", TENS[tens_digit], ONES[ones_digit]));
    }

    parts.join(" ")
}

// ============================================================================
// Conversion Functions
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

            let mut result = if thousands == 1 {
                format!("se{THOUSAND_WORD}")
            } else {
                format!("{} {THOUSAND_WORD}", build_segment(thousands))
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

/// Words for one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);

    let mut parts: Vec<String> = Vec::new();

    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if scale_index == 0 {
            parts.push(build_segment(segment));
        } else if scale_index == 1 {
            if segment == 1 {
                parts.push(format!("se{THOUSAND_WORD}"));
            } else {
                parts.push(format!("{} {THOUSAND_WORD}", build_segment(segment)));
            }
        } else {
            // Malay: "se-" prefix for ALL scale words when segment is 1
            let scale_word = SCALE_WORDS[scale_index - 2];
            if segment == 1 {
                parts.push(format!("se{scale_word}"));
            } else {
                parts.push(format!("{} {scale_word}", build_segment(segment)));
            }
        }
    }

    parts.join(" ")
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

/// Spells a number in Malay words.
///
/// ```
/// # use n2words::ms_my::to_cardinal;
/// assert_eq!(to_cardinal(1000000)?, "sejuta");
/// assert_eq!(to_cardinal(3.14)?, "tiga perpuluhan empat belas");
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
    CARDINAL_MAX.check(&number.integer, number.fraction.as_deref())?;

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

/// Ordinal words for a positive integer.
///
/// Malay ordinals use the "ke-" prefix + cardinal number.
/// Special case: "pertama" for 1st (not "kesatu").
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special case: 1st is "pertama"
    if n.is_one() {
        return ORDINAL_FIRST.to_string();
    }

    // All others: "ke" + cardinal (no hyphen in Malay)
    format!("{ORDINAL_PREFIX}{}", integer_to_words(n))
}

/// Spells a positive integer as a Malay ordinal.
///
/// ```
/// # use n2words::ms_my::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "pertama");
/// assert_eq!(to_ordinal(2)?, "kedua");
/// assert_eq!(to_ordinal(10)?, "kesepuluh");
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
        currency: Currency = Currency::MYR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Malaysian ringgit (100 sen = 1 ringgit).
///
/// ```
/// # use n2words::ms_my::to_currency;
/// assert_eq!(to_currency(42)?, "empat puluh dua ringgit");
/// assert_eq!(to_currency(1.5)?, "satu ringgit lima puluh sen");
/// assert_eq!(to_currency(-5)?, "minus lima ringgit");
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

/// Spells an amount in any currency Malay has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Malay has no words for and
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

    // Ringgit part - show if non-zero, or if no sen. Ringgit and sen are
    // invariable, so the first form serves every count.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(0));
    }

    // Sen part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(0));
    }

    Ok(result)
}

/// Registry entry for `ms-MY`.
pub static LANGUAGE: Language = Language {
    code: "ms-MY",
    name: "Malay (Malaysia)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
