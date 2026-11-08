//! Russian (Russia) (`ru-RU`).
//!
//! - Three-form pluralization (one/few/many): "один рубль", "два рубля",
//!   "пять рублей"
//! - Gender: thousands are feminine ("одна тысяча"), millions and up are
//!   masculine ("один миллион")
//! - Irregular hundreds: "двести", "триста", "пятьсот"
//! - Scale words миллион, миллиард, триллион, ... up to нониллион (10^30)
//! - Ordinals in the masculine nominative; only the last component is
//!   ordinal: 121 → "сто двадцать первый"
//!
//! ```
//! use n2words::ru_ru::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "двадцать один");
//! assert_eq!(to_ordinal(42)?, "сорок второй");
//! assert_eq!(to_currency(42.5)?, "сорок два рубля и пятьдесят копеек");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::RU as CURRENCY_VOCAB;
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
    "четыре",
    "пять",
    "шесть",
    "семь",
    "восемь",
    "девять",
];
const ONES_FEM: [&str; 10] = [
    "",
    "одна",
    "две",
    "три",
    "четыре",
    "пять",
    "шесть",
    "семь",
    "восемь",
    "девять",
];
const TEENS: [&str; 10] = [
    "десять",
    "одиннадцать",
    "двенадцать",
    "тринадцать",
    "четырнадцать",
    "пятнадцать",
    "шестнадцать",
    "семнадцать",
    "восемнадцать",
    "девятнадцать",
];
const TENS: [&str; 10] = [
    "",
    "",
    "двадцать",
    "тридцать",
    "сорок",
    "пятьдесят",
    "шестьдесят",
    "семьдесят",
    "восемьдесят",
    "девяносто",
];
// Irregular hundreds
const HUNDREDS: [&str; 10] = [
    "",
    "сто",
    "двести",
    "триста",
    "четыреста",
    "пятьсот",
    "шестьсот",
    "семьсот",
    "восемьсот",
    "девятьсот",
];

const ZERO: &str = "ноль";
const NEGATIVE: &str = "минус";
const DECIMAL_SEP: &str = "запятая";

// Scale words `[one, few, many]`; thousands (index 0) are feminine, the
// rest masculine.
const SCALE_FORMS: [[&str; 3]; 10] = [
    ["тысяча", "тысячи", "тысяч"],
    ["миллион", "миллиона", "миллионов"],
    ["миллиард", "миллиарда", "миллиардов"],
    ["триллион", "триллиона", "триллионов"],
    ["квадриллион", "квадриллиона", "квадриллионов"],
    ["квинтиллион", "квинтиллиона", "квинтиллионов"],
    ["секстиллион", "секстиллиона", "секстиллионов"],
    ["септиллион", "септиллиона", "септиллионов"],
    ["октиллион", "октиллиона", "октиллионов"],
    ["нониллион", "нониллиона", "нониллионов"],
];

// ============================================================================
// Ordinal vocabulary (masculine nominative)
// ============================================================================

const ORDINAL_ONES: [&str; 10] = [
    "",
    "первый",
    "второй",
    "третий",
    "четвёртый",
    "пятый",
    "шестой",
    "седьмой",
    "восьмой",
    "девятый",
];
const ORDINAL_TEENS: [&str; 10] = [
    "десятый",
    "одиннадцатый",
    "двенадцатый",
    "тринадцатый",
    "четырнадцатый",
    "пятнадцатый",
    "шестнадцатый",
    "семнадцатый",
    "восемнадцатый",
    "девятнадцатый",
];
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "двадцатый",
    "тридцатый",
    "сороковой",
    "пятидесятый",
    "шестидесятый",
    "семидесятый",
    "восьмидесятый",
    "девяностый",
];
const ORDINAL_HUNDREDS: [&str; 10] = [
    "",
    "сотый",
    "двухсотый",
    "трёхсотый",
    "четырёхсотый",
    "пятисотый",
    "шестисотый",
    "семисотый",
    "восьмисотый",
    "девятисотый",
];
// Ordinal scale words (тысячный, миллионный, ...)
const ORDINAL_SCALES: [&str; 10] = [
    "тысячный",
    "миллионный",
    "миллиардный",
    "триллионный",
    "квадриллионный",
    "квинтиллионный",
    "секстиллионный",
    "септиллионный",
    "октиллионный",
    "нониллионный",
];
// Prefixes for compound ordinal thousands (двух-, трёх-, ... + тысячный)
const THOUSAND_PREFIXES: [&str; 10] = [
    "",
    "",
    "двух",
    "трёх",
    "четырёх",
    "пяти",
    "шести",
    "семи",
    "восьми",
    "девяти",
];

/// The cardinal ceiling, derived from the scale table: `10^33`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_FORMS.len() as u32);
/// The ordinal ceiling, derived from the ordinal scale table: `10^33`.
pub const ORDINAL_MAX: Max = Max::western(ORDINAL_SCALES.len() as u32);
/// The currency ceiling: `10^33`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_FORMS.len() as u32);

// ============================================================================
// Shared helpers
// ============================================================================

/// The `[one, few, many]` form index for a count, by the Russian rules:
/// 11-19 take "many"; otherwise a final 1 takes "one", a final 2-4 "few",
/// and a final 0 or 5-9 "many".
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
/// tens and the gendered ones ("двадцать один" / "двадцать одна"). Only the
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

        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            // Thousands are always feminine in Russian
            let mut result = format!(
                "{} {}",
                build_segment(thousands, Gender::Feminine),
                pluralize(thousands, &SCALE_FORMS[0])
            );
            if remainder > 0 {
                result.push(' ');
                result.push_str(&build_segment(remainder, gender));
            }
            return result;
        }
    }

    build_large_number_words(n, gender)
}

/// Words for a number of one million or more, by 3-digit scale segments.
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

/// Spells a number in Russian words (masculine).
///
/// ```
/// # use n2words::ru_ru::to_cardinal;
/// assert_eq!(to_cardinal(1000)?, "одна тысяча");
/// assert_eq!(to_cardinal(-1.7)?, "минус один запятая семь");
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

/// Spells a number in Russian words, with options.
///
/// ```
/// # use n2words::{ru_ru::{to_cardinal_with, CardinalOptions}, Gender};
/// let feminine = CardinalOptions { gender: Gender::Feminine };
/// assert_eq!(to_cardinal_with(21, &feminine)?, "двадцать одна");
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

/// Ordinal words for a 0-99 final segment: "первый", "двадцать первый".
fn build_ordinal_tens_ones(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    let ones = (n % 10) as usize;
    let tens = (n / 10) as usize;

    if tens == 0 {
        // Single digit: первый, второй, ...
        return ORDINAL_ONES[ones].to_string();
    }
    if tens == 1 {
        // Teens: десятый, одиннадцатый, ...
        return ORDINAL_TEENS[ones].to_string();
    }
    if ones == 0 {
        // Round tens: двадцатый, тридцатый, ...
        return ORDINAL_TENS[tens].to_string();
    }
    // Compound: cardinal tens, ordinal ones (двадцать первый)
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
        // Exact hundreds: сотый, двухсотый, ...
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
/// Only the last component becomes ordinal: 121 → "сто двадцать первый".
fn integer_to_ordinal(n: &BigUint) -> String {
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_ordinal_segment(small as u32);
        }

        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            if remainder == 0 {
                // Exact thousands: тысячный, двухтысячный, ..., then
                // cardinal + тысячный from ten thousand up
                return match thousands {
                    1 => ORDINAL_SCALES[0].to_string(),
                    2..=9 => format!(
                        "{}{}",
                        THOUSAND_PREFIXES[thousands as usize], ORDINAL_SCALES[0]
                    ),
                    _ => format!(
                        "{} {}",
                        build_segment(thousands, Gender::Feminine),
                        ORDINAL_SCALES[0]
                    ),
                };
            }

            // Cardinal thousands + ordinal remainder
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
            // This scale position is the final, ordinal one
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

/// Spells a positive integer as a Russian ordinal (masculine nominative).
///
/// ```
/// # use n2words::ru_ru::to_ordinal;
/// assert_eq!(to_ordinal(21)?, "двадцать первый");
/// assert_eq!(to_ordinal(100)?, "сотый");
/// assert_eq!(to_ordinal(2000)?, "двухтысячный");
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
        /// Use "и" between rubles and kopecks
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::RUB, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Russian rubles.
///
/// ```
/// # use n2words::ru_ru::to_currency;
/// assert_eq!(to_currency(1)?, "один рубль");
/// assert_eq!(to_currency(0.01)?, "одна копейка");
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

/// Spells an amount in any currency Russian has words for, with options.
///
/// ```
/// # use n2words::ru_ru::{to_currency_with, CurrencyOptions};
/// let plain = CurrencyOptions { and: false, ..Default::default() };
/// assert_eq!(to_currency_with(42.5, &plain)?, "сорок два рубля пятьдесят копеек");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Russian has no words for and
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
    // Every Russian entry carries both genders; the fallbacks are the default
    // nouns' own (рубль masculine, копейка feminine).
    let major_gender = words.major_gender.unwrap_or(Gender::Masculine);
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
            result.push_str(if options.and { " и " } else { " " });
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor), minor_gender));
        result.push(' ');
        result.push_str(words.minor_at(plural_index(money.minor)));
    }
    Ok(result)
}

/// Registry entry for `ru-RU`.
pub static LANGUAGE: Language = Language {
    code: "ru-RU",
    name: "Russian (Russia)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
