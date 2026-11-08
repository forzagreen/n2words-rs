//! Hungarian (Hungary) (`hu-HU`).
//!
//! - Agglutinative structure (no spaces between compound parts)
//! - Special handling for "egy" (one) omission
//! - Pre-composed twenties (huszonegy through huszonkilenc)
//! - "két" instead of "kettő" in compound forms
//! - No fixed scale ceiling: the speller composes every magnitude
//!
//! ```
//! use n2words::hu_hu::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "negyvenkettő");
//! assert_eq!(to_ordinal(42)?, "negyvenkettődik");
//! assert_eq!(to_currency(42.5)?, "negyvenkettő forint ötven fillér");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::HU as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::{Max, pow10};
use crate::number::{Number, digits_to_biguint, parse_cardinal, parse_currency, parse_ordinal};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

// No fixed scale ceiling — the speller composes every magnitude.

/// The cardinal ceiling: none, every magnitude is composed.
pub const CARDINAL_MAX: Max = Max::Unbounded;
/// The ordinal ceiling: none.
pub const ORDINAL_MAX: Max = Max::Unbounded;
/// The currency ceiling: none.
pub const CURRENCY_MAX: Max = Max::Unbounded;

// ============================================================================
// Vocabulary
// ============================================================================

/// Words for 0-29: ones, teens and the pre-composed twenties.
const WORDS: [&str; 30] = [
    "nulla",
    "egy",
    "kettő",
    "három",
    "négy",
    "öt",
    "hat",
    "hét",
    "nyolc",
    "kilenc",
    "tíz",
    "tizenegy",
    "tizenkettő",
    "tizenhárom",
    "tizennégy",
    "tizenöt",
    "tizenhat",
    "tizenhét",
    "tizennyolc",
    "tizenkilenc",
    "húsz",
    "huszonegy",
    "huszonkettő",
    "huszonhárom",
    "huszonnégy",
    "huszonöt",
    "huszonhat",
    "huszonhét",
    "huszonnyolc",
    "huszonkilenc",
];

/// Tens from 30 to 90, indexed by the tens digit.
const TENS: [&str; 10] = [
    "",
    "",
    "",
    "harminc",
    "negyven",
    "ötven",
    "hatvan",
    "hetven",
    "nyolcvan",
    "kilencven",
];

const HUNDRED: &str = "száz";
const THOUSAND: &str = "ezer";
/// "két" replaces "kettő" in compound forms.
const TWO_COMPOUND: &str = "két";

/// Scale words with their base-10 exponent, from a million upwards.
const SCALES: [(u32, &str); 8] = [
    (6, "millió"),
    (9, "milliárd"),
    (12, "billió"),
    (15, "billiárd"),
    (18, "trillió"),
    (21, "trilliárd"),
    (24, "quadrillió"),
    (27, "quadrilliárd"),
];

const ZERO: &str = "nulla";
const NEGATIVE: &str = "mínusz";
const DECIMAL_SEP: &str = "egész";

// ============================================================================
// Ordinal vocabulary
// ============================================================================

// Hungarian ordinals: special forms 1-10, then cardinal + -dik/-ik suffix
// Vowel harmony determines -dik vs -ik
const ORDINAL_SPECIAL: [(u64, &str); 20] = [
    (1, "első"),
    (2, "második"),
    (3, "harmadik"),
    (4, "negyedik"),
    (5, "ötödik"),
    (6, "hatodik"),
    (7, "hetedik"),
    (8, "nyolcadik"),
    (9, "kilencedik"),
    (10, "tizedik"),
    (20, "huszadik"),
    (30, "harmincadik"),
    (40, "negyvenedik"),
    (50, "ötvenedik"),
    (60, "hatvanadik"),
    (70, "hetvenedik"),
    (80, "nyolcvanadik"),
    (90, "kilencvenedik"),
    (100, "századik"),
    (1000, "ezredik"),
];

// ============================================================================
// Cardinal
// ============================================================================

/// Words for an integer under 100.
fn tens_to_cardinal(n: u32) -> String {
    let tens = (n / 10) as usize;
    let units = n % 10;
    if units == 0 {
        return TENS[tens].to_string();
    }
    format!("{}{}", TENS[tens], small_to_words(units, ZERO))
}

/// Words for an integer under 1000.
fn hundreds_to_cardinal(n: u32) -> String {
    let hundreds = n / 100;
    let mut prefix = String::new();
    if hundreds != 1 {
        prefix.push_str(&small_to_words(hundreds, ""));
    }
    prefix.push_str(HUNDRED);
    let postfix = small_to_words(n % 100, "");
    prefix + &postfix
}

/// Words for an integer under one million.
fn thousands_to_cardinal(n: u32) -> String {
    let thousands = n / 1000;
    let mut prefix = String::new();
    if thousands != 1 {
        prefix.push_str(&small_to_words(thousands, ""));
    }
    prefix.push_str(THOUSAND);
    let postfix = small_to_words(n % 1000, "");
    let middle = if n <= 2000 || postfix.is_empty() {
        ""
    } else {
        "-"
    };
    prefix + middle + &postfix
}

/// Words for an integer of one million or more: the largest scale word the
/// number reaches, with the quotient composed before it and the remainder
/// hyphenated after it.
fn big_number_to_cardinal(n: &BigUint) -> String {
    // Find the appropriate scale: the largest one the number reaches, a
    // million at least. `n >= 10^e` exactly when it has more than `e` digits.
    let digits = n.to_string().len();
    let (exponent, scale_word) = SCALES
        .iter()
        .rev()
        .find(|(exponent, _)| digits > *exponent as usize)
        .unwrap_or(&SCALES[0]);
    let exp = pow10(*exponent);

    let prefix = integer_to_words_inner(&(n / &exp), "");
    let rest = integer_to_words_inner(&(n % &exp), "");
    let mut result = prefix;
    result.push_str(scale_word);
    if !rest.is_empty() {
        result.push('-');
        result.push_str(&rest);
    }
    result
}

/// Words for an integer under one million. `zero_word` is the word for zero:
/// empty in compound contexts, where two is also "két" rather than "kettő".
fn small_to_words(n: u32, zero_word: &str) -> String {
    if n == 0 {
        return zero_word.to_string();
    }
    if zero_word.is_empty() && n == 2 {
        return TWO_COMPOUND.to_string();
    }
    if n < 30 {
        return WORDS[n as usize].to_string();
    }
    if n < 100 {
        return tens_to_cardinal(n);
    }
    if n < 1000 {
        return hundreds_to_cardinal(n);
    }
    thousands_to_cardinal(n)
}

/// Words for a non-negative integer, with `zero_word` for zero (empty in
/// compound contexts).
fn integer_to_words_inner(n: &BigUint, zero_word: &str) -> String {
    match n.to_u64() {
        Some(small) if small < 1_000_000 => small_to_words(small as u32, zero_word),
        _ => big_number_to_cardinal(n),
    }
}

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint) -> String {
    integer_to_words_inner(n, ZERO)
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

/// Spells a number in Hungarian words.
///
/// ```
/// # use n2words::hu_hu::to_cardinal;
/// assert_eq!(to_cardinal(2385)?, "kétezer-háromszáznyolcvanöt");
/// assert_eq!(to_cardinal(1000001)?, "egymillió-egy");
/// assert_eq!(to_cardinal(-17.42)?, "mínusz tizenhét egész negyvenkettő");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value. There is no ceiling ([`CARDINAL_MAX`] is unbounded).
pub fn to_cardinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    let number = parse_cardinal(&value.into())?;
    // Unbounded, so this never fails; kept so the form reads like the others.
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
/// Hungarian ordinals: első (1st), második (2nd), harmadik (3rd), etc.
/// 1-10 have special forms, others use cardinal + -dik/-edik/-adik/-ödik suffix.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Exact special forms
    if let Some(small) = n.to_u64() {
        if let Some((_, word)) = ORDINAL_SPECIAL.iter().find(|(value, _)| *value == small) {
            return (*word).to_string();
        }
    }

    // For compound numbers, get cardinal and add suffix
    let mut cardinal = integer_to_words(n);

    // Determine suffix based on last vowel (vowel harmony)
    // Back vowels (a,á,o,ó,u,ú) -> -adik
    // Front unrounded (e,é,i,í) -> -edik
    // Front rounded (ö,ő,ü,ű) -> -ödik
    // If ends in consonant, add linking vowel + dik
    const VOWELS: &str = "aáoóuúeéiíöőüű";
    const BACK_VOWELS: &str = "aáoóuú";
    const FRONT_ROUNDED_VOWELS: &str = "öőüű";
    const FRONT_UNROUNDED_VOWELS: &str = "eéií";

    // If ends in vowel, just add -dik
    if cardinal.chars().last().is_some_and(|c| VOWELS.contains(c)) {
        cardinal.push_str("dik");
        return cardinal;
    }

    // If ends in consonant, find last vowel for harmony
    for c in cardinal.chars().rev() {
        if BACK_VOWELS.contains(c) {
            cardinal.push_str("adik");
            return cardinal;
        }
        if FRONT_ROUNDED_VOWELS.contains(c) {
            cardinal.push_str("ödik");
            return cardinal;
        }
        if FRONT_UNROUNDED_VOWELS.contains(c) {
            cardinal.push_str("edik");
            return cardinal;
        }
    }

    // Default to -edik
    cardinal.push_str("edik");
    cardinal
}

/// Spells a positive integer as a Hungarian ordinal.
///
/// ```
/// # use n2words::hu_hu::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "első");
/// assert_eq!(to_ordinal(2)?, "második");
/// assert_eq!(to_ordinal(21)?, "huszonegyedik");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidOrdinal`](crate::Error::InvalidOrdinal) for zero, a
/// negative or a fraction. There is no ceiling ([`ORDINAL_MAX`] is unbounded).
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
        currency: Currency = Currency::HUF, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Hungarian forint.
///
/// Uses forint (no plural form needed in Hungarian).
/// Fillér (1/100) is rarely used but included for completeness.
///
/// ```
/// # use n2words::hu_hu::to_currency;
/// assert_eq!(to_currency(42)?, "negyvenkettő forint");
/// assert_eq!(to_currency(1.5)?, "egy forint ötven fillér");
/// assert_eq!(to_currency(-5)?, "mínusz öt forint");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value. There is no ceiling ([`CURRENCY_MAX`] is unbounded).
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency Hungarian has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Hungarian has no words for and
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

    // Forint part
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(0));
    }

    // Fillér part
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

/// Registry entry for `hu-HU`.
pub static LANGUAGE: Language = Language {
    code: "hu-HU",
    name: "Hungarian (Hungary)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
