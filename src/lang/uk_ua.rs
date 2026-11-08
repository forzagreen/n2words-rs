//! Ukrainian (Ukraine) (`uk-UA`).
//!
//! - Three-form pluralization (one/few/many): "одна гривня", "двi гривнi",
//!   "п'ять гривень"
//! - Gender: thousands are feminine ("одна тисяча"), millions and up are
//!   masculine ("один мiльйон")
//! - Irregular hundreds: "двiстi", "триста", "п'ятсот"
//! - Scale words мiльйон, мiльярд, трильйон, ... up to октильйон (10^27)
//! - Ordinals in the masculine nominative; only the last component is
//!   ordinal: 121 → "сто двадцять перший"
//!
//! The vocabulary is spelled as in the JavaScript source, which writes the
//! Cyrillic letter і with the ASCII letter `i`; the fixtures use the same
//! spelling.
//!
//! ```
//! use n2words::uk_ua::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "двадцять один");
//! assert_eq!(to_ordinal(42)?, "сорок другий");
//! assert_eq!(to_currency(42.5)?, "сорок двi гривнi п'ятдесят копiйок");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::UK as CURRENCY_VOCAB;
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
    "",
    "один",
    "два",
    "три",
    "чотири",
    "п'ять",
    "шiсть",
    "сiм",
    "вiсiм",
    "дев'ять",
];
const ONES_FEM: [&str; 10] = [
    "",
    "одна",
    "двi",
    "три",
    "чотири",
    "п'ять",
    "шiсть",
    "сiм",
    "вiсiм",
    "дев'ять",
];
const TEENS: [&str; 10] = [
    "десять",
    "одинадцять",
    "дванадцять",
    "тринадцять",
    "чотирнадцять",
    "п'ятнадцять",
    "шiстнадцять",
    "сiмнадцять",
    "вiсiмнадцять",
    "дев'ятнадцять",
];
const TENS: [&str; 10] = [
    "",
    "",
    "двадцять",
    "тридцять",
    "сорок",
    "п'ятдесят",
    "шiстдесят",
    "сiмдесят",
    "вiсiмдесят",
    "дев'яносто",
];
// Irregular hundreds
const HUNDREDS: [&str; 10] = [
    "",
    "сто",
    "двiстi",
    "триста",
    "чотириста",
    "п'ятсот",
    "шiстсот",
    "сiмсот",
    "вiсiмсот",
    "дев'ятсот",
];

const ZERO: &str = "нуль";
const NEGATIVE: &str = "мiнус";
const DECIMAL_SEP: &str = "кома";

// Scale words `[one, few, many]`; thousands (index 0) are feminine, the
// rest masculine.
const SCALE_FORMS: [[&str; 3]; 9] = [
    ["тисяча", "тисячi", "тисяч"],
    ["мiльйон", "мiльйони", "мiльйонiв"],
    ["мiльярд", "мiльярди", "мiльярдiв"],
    ["трильйон", "трильйони", "трильйонiв"],
    ["квадрильйон", "квадрильйони", "квадрильйонiв"],
    ["квiнтильйон", "квiнтильйони", "квiнтильйонiв"],
    ["секстильйон", "секстильйони", "секстильйонiв"],
    ["септильйон", "септильйони", "септильйонiв"],
    ["октильйон", "октильйони", "октильйонiв"],
];

// ============================================================================
// Ordinal vocabulary (masculine nominative singular)
// ============================================================================

const ORDINAL_ONES: [&str; 10] = [
    "",
    "перший",
    "другий",
    "третiй",
    "четвертий",
    "п'ятий",
    "шостий",
    "сьомий",
    "восьмий",
    "дев'ятий",
];
const ORDINAL_TEENS: [&str; 10] = [
    "десятий",
    "одинадцятий",
    "дванадцятий",
    "тринадцятий",
    "чотирнадцятий",
    "п'ятнадцятий",
    "шiстнадцятий",
    "сiмнадцятий",
    "вiсiмнадцятий",
    "дев'ятнадцятий",
];
// Exact tens
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "двадцятий",
    "тридцятий",
    "сороковий",
    "п'ятдесятий",
    "шiстдесятий",
    "сiмдесятий",
    "вiсiмдесятий",
    "дев'яностий",
];
// Exact hundreds
const ORDINAL_HUNDREDS: [&str; 10] = [
    "",
    "сотий",
    "двохсотий",
    "трьохсотий",
    "чотирьохсотий",
    "п'ятисотий",
    "шестисотий",
    "семисотий",
    "восьмисотий",
    "дев'ятисотий",
];
// Scale ordinals
const ORDINAL_SCALES: [&str; 4] = ["тисячний", "мiльйонний", "мiльярдний", "трильйонний"];

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_FORMS.len() as u32);
/// The ordinal ceiling, derived from the ordinal scale table: `10^15`.
pub const ORDINAL_MAX: Max = Max::western(ORDINAL_SCALES.len() as u32);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_FORMS.len() as u32);

// ============================================================================
// Shared helpers
// ============================================================================

/// The `[one, few, many]` form index for a count: 11-19 take "many";
/// otherwise a final 1 takes "one", a final 2-4 "few", and a final 0 or 5-9
/// "many".
fn plural_index(n: u32) -> usize {
    if (11..=19).contains(&(n % 100)) {
        return 2;
    }
    match n % 10 {
        1 => 0,
        2..=4 => 1,
        _ => 2,
    }
}

/// [`plural_index`] for an arbitrarily large count: only its last two digits
/// matter.
fn plural_index_big(n: &BigUint) -> usize {
    plural_index((n % 100u32).to_u32().unwrap_or(0))
}

/// Picks the `[one, few, many]` form for `n`.
fn pluralize<'a>(n: u32, forms: &[&'a str; 3]) -> &'a str {
    forms[plural_index(n)]
}

/// Cardinal words for a 0-999 segment: irregular hundreds, then teens or
/// tens and the gendered ones ("двадцять один" / "двадцять одна"). Only the
/// ones differ by gender, so one builder serves both.
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
    if tens > 1 {
        parts.push(TENS[tens]);
    }
    if tens == 1 {
        parts.push(TEENS[ones]);
    } else if ones > 0 {
        parts.push(ones_words[ones]);
    }
    parts.join(" ")
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer, with `gender` applied to the units
/// segment.
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

/// Words for a number of one thousand or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint, gender: Gender) -> String {
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
            result.push_str(&build_segment(segment, gender));
        } else {
            // Thousands (i == 1) are feminine, the other scales masculine
            let scale_gender = if i == 1 {
                Gender::Feminine
            } else {
                Gender::Masculine
            };
            result.push_str(&build_segment(segment, scale_gender));
            result.push(' ');
            result.push_str(pluralize(segment, &SCALE_FORMS[i - 1]));
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
        /// Grammatical gender
        gender: Gender = Gender::Masculine;
    }
}

/// Spells a number in Ukrainian words (masculine).
///
/// ```
/// # use n2words::uk_ua::to_cardinal;
/// assert_eq!(to_cardinal(1000)?, "одна тисяча");
/// assert_eq!(to_cardinal(1.7)?, "один кома сiм");
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

/// Spells a number in Ukrainian words, with options.
///
/// ```
/// # use n2words::{uk_ua::{to_cardinal_with, CardinalOptions}, Gender};
/// let feminine = CardinalOptions { gender: Gender::Feminine };
/// assert_eq!(to_cardinal_with(22, &feminine)?, "двадцять двi");
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

/// Ordinal words for a 0-99 final segment: "перший", "двадцять перший".
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
                return format!(
                    "{} {}",
                    build_segment(thousands, Gender::Feminine),
                    ORDINAL_SCALES[0]
                );
            }

            return format!(
                "{} {} {}",
                build_segment(thousands, Gender::Feminine),
                pluralize(thousands, &SCALE_FORMS[0]),
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
        // Thousands are feminine, the other scales masculine
        let scale_gender = if i == 1 {
            Gender::Feminine
        } else {
            Gender::Masculine
        };
        if i == lowest_non_zero {
            if segment == 1 {
                result.push_str(ORDINAL_SCALES[i - 1]);
            } else {
                result.push_str(&build_segment(segment, scale_gender));
                result.push(' ');
                result.push_str(ORDINAL_SCALES[i - 1]);
            }
        } else {
            result.push_str(&build_segment(segment, scale_gender));
            result.push(' ');
            result.push_str(pluralize(segment, &SCALE_FORMS[i - 1]));
        }
    }
    result
}

/// Spells a positive integer as a Ukrainian ordinal (masculine nominative).
///
/// ```
/// # use n2words::uk_ua::to_ordinal;
/// assert_eq!(to_ordinal(21)?, "двадцять перший");
/// assert_eq!(to_ordinal(100)?, "сотий");
/// assert_eq!(to_ordinal(1000)?, "тисячний");
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
        currency: Currency = Currency::UAH, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Ukrainian hryvnias.
///
/// ```
/// # use n2words::uk_ua::to_currency;
/// assert_eq!(to_currency(1)?, "одна гривня");
/// assert_eq!(to_currency(1.5)?, "одна гривня п'ятдесят копiйок");
/// assert_eq!(to_currency(-5)?, "мiнус п'ять гривень");
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

/// Spells an amount in any currency Ukrainian has words for.
///
/// ```
/// # use n2words::{uk_ua::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::UAH };
/// assert_eq!(to_currency_with(0.01, &opts)?, "одна копiйка");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Ukrainian has no words for and
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
    // Every Ukrainian entry carries both genders; the fallbacks are the
    // default nouns' own (гривня and копiйка are both feminine).
    let major_gender = words.major_gender.unwrap_or(Gender::Feminine);
    let minor_gender = words.minor_gender.unwrap_or(Gender::Feminine);

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

/// Registry entry for `uk-UA`.
pub static LANGUAGE: Language = Language {
    code: "uk-UA",
    name: "Ukrainian (Ukraine)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
