//! Romanian (Romania) (`ro-RO`).
//!
//! - Gender agreement (unu/una, doi/două)
//! - "De" preposition insertion for groups >= 20
//! - Complex scale word handling (mie/mii, milion/milioane)
//! - Feminine units for thousands
//!
//! ```
//! use n2words::ro_ro::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "douăzeci și unu");
//! assert_eq!(to_ordinal(21)?, "douăzeci și primul");
//! assert_eq!(to_currency(1)?, "un leu");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::RO as CURRENCY_VOCAB;
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
    "", "unu", "doi", "trei", "patru", "cinci", "șase", "șapte", "opt", "nouă",
];
const ONES_FEM: [&str; 10] = [
    "", "una", "două", "trei", "patru", "cinci", "șase", "șapte", "opt", "nouă",
];

const TEENS: [&str; 10] = [
    "zece",
    "unsprezece",
    "douăsprezece",
    "treisprezece",
    "paisprezece",
    "cincisprezece",
    "șaisprezece",
    "șaptesprezece",
    "optsprezece",
    "nouăsprezece",
];
const TEENS_MASC: [&str; 10] = [
    "zece",
    "unsprezece",
    "doisprezece",
    "treisprezece",
    "paisprezece",
    "cincisprezece",
    "șaisprezece",
    "șaptesprezece",
    "optsprezece",
    "nouăsprezece",
];

const TWENTIES: [&str; 10] = [
    "",
    "",
    "douăzeci",
    "treizeci",
    "patruzeci",
    "cincizeci",
    "șaizeci",
    "șaptezeci",
    "optzeci",
    "nouăzeci",
];

const HUNDREDS: [&str; 10] = [
    "",
    "o sută",
    "două sute",
    "trei sute",
    "patru sute",
    "cinci sute",
    "șase sute",
    "șapte sute",
    "opt sute",
    "nouă sute",
];

const ZERO: &str = "zero";
const NEGATIVE: &str = "minus";
const DECIMAL_SEP: &str = "virgulă";

// Ordinal vocabulary (masculine forms)
const ORDINAL_ONES: [&str; 10] = [
    "",
    "primul",
    "al doilea",
    "al treilea",
    "al patrulea",
    "al cincilea",
    "al șaselea",
    "al șaptelea",
    "al optulea",
    "al nouălea",
];
const ORDINAL_TEENS: [&str; 10] = [
    "al zecelea",
    "al unsprezecelea",
    "al doisprezecelea",
    "al treisprezecelea",
    "al paisprezecelea",
    "al cincisprezecelea",
    "al șaisprezecelea",
    "al șaptesprezecelea",
    "al optsprezecelea",
    "al nouăsprezecelea",
];
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "al douăzecilea",
    "al treizecilea",
    "al patruzecilea",
    "al cincizecilea",
    "al șaizecilea",
    "al șaptezecilea",
    "al optzecilea",
    "al nouăzecilea",
];
const ORDINAL_HUNDRED: &str = "al sutălea";
const ORDINAL_THOUSAND: &str = "al miilea";
const ORDINAL_MILLION: &str = "al milionulea";

/// Scale metadata.
struct ScaleMeta {
    /// Form for 1.
    singular: &'static str,
    /// Form for 2+.
    plural: &'static str,
    /// "o" for feminine, "un" for masculine.
    article: &'static str,
    /// Whether "de" is inserted for a segment >= 20.
    needs_de: bool,
}

const fn scale(singular: &'static str, plural: &'static str, article: &'static str) -> ScaleMeta {
    ScaleMeta {
        singular,
        plural,
        article,
        needs_de: true,
    }
}

const SCALE_META: &[ScaleMeta] = &[
    scale("mie", "mii", "o"),
    scale("milion", "milioane", "un"),
    scale("miliard", "miliarde", "un"),
    scale("trilion", "trilioane", "un"),
    scale("cvadrilion", "cvadrilioane", "un"),
    scale("cvintilion", "cvintilioane", "un"),
    scale("sextilion", "sextilioane", "un"),
    scale("septilion", "septilioane", "un"),
    scale("octilion", "octilioane", "un"),
];

// Cardinal scale words run thousand..octilion; past SCALE_META the builder
// drops the scale word. The ordinal spells its millions multiplier as a 0-999
// segment, so it overflows once that multiplier reaches 1000 — n must stay
// below 10^9.

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::western(SCALE_META.len() as u32);
/// The ordinal ceiling: `10^9`.
pub const ORDINAL_MAX: Max = Max::bounded(9);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_META.len() as u32);

// ============================================================================
// Helpers
// ============================================================================

/// Spells a number under 100.
fn spell_under_100(n: u32, feminine: bool, masculine_teens: bool) -> String {
    let n = n as usize;
    if n == 0 {
        return String::new();
    }
    let ones = if feminine { &ONES_FEM } else { &ONES_MASC };
    if n < 10 {
        return ones[n].to_string();
    }
    if n < 20 {
        let teens = if masculine_teens { &TEENS_MASC } else { &TEENS };
        return teens[n - 10].to_string();
    }
    let t = n / 10;
    let u = n % 10;
    if u == 0 {
        return TWENTIES[t].to_string();
    }
    format!("{} și {}", TWENTIES[t], ones[u])
}

/// Spells a number under 1000.
fn spell_under_1000(n: u32, feminine: bool, masculine_teens: bool) -> String {
    if n < 100 {
        return spell_under_100(n, feminine, masculine_teens);
    }
    let hundred_word = HUNDREDS[(n / 100) as usize];
    let r = n % 100;
    if r == 0 {
        return hundred_word.to_string();
    }
    format!(
        "{hundred_word} {}",
        spell_under_100(r, feminine, masculine_teens)
    )
}

/// Builds a scale phrase with proper pluralization and "de" insertion.
/// Romanian always uses feminine forms (două, not doi) when counting scale
/// words.
fn build_scale_phrase(segment: u32, scale_index: usize) -> String {
    let Some(meta) = SCALE_META.get(scale_index - 1) else {
        return spell_under_1000(segment, true, false);
    };

    if segment == 1 {
        return format!("{} {}", meta.article, meta.singular);
    }

    // Special case: 21 with scale words uses feminine "una"
    if segment == 21 && meta.needs_de {
        return format!("douăzeci și una de {}", meta.plural);
    }

    // Romanian always uses feminine when counting scale words (două milioane,
    // not doi milioane)
    let words = spell_under_1000(segment, true, false);

    // "de" after >= 20
    let separator = if meta.needs_de && segment >= 20 {
        " de "
    } else {
        " "
    };

    format!("{words}{separator}{}", meta.plural)
}

// ============================================================================
// Cardinal
// ============================================================================

/// Converts a non-negative integer to Romanian words.
fn integer_to_words(n: &BigUint, gender: Gender) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Fast path: numbers < 1000
    if let Some(small) = n.to_u64().filter(|&small| small < 1000) {
        return spell_under_1000(small as u32, gender == Gender::Feminine, false);
    }

    build_large_number_words(n, gender)
}

/// Builds words for numbers >= 1000.
fn build_large_number_words(n: &BigUint, gender: Gender) -> String {
    let mut result = String::new();

    for (i, &segment) in split_groups(n, 1000).iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }

        let segment_words = if i == 0 {
            // Units segment - use gender from options
            spell_under_1000(segment, gender == Gender::Feminine, false)
        } else {
            build_scale_phrase(segment, i)
        };

        if !segment_words.is_empty() {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&segment_words);
        }
    }

    result
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number. Decimals always use masculine forms.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut result = String::new();

    // Leading zeros
    let zeros = decimal.bytes().take_while(|&b| b == b'0').count();
    for _ in 0..zeros {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(ZERO);
    }

    // The rest as a single number (masculine, with masculine teens)
    let remainder = &decimal[zeros..];
    if !remainder.is_empty() {
        if !result.is_empty() {
            result.push(' ');
        }
        let n = digits_to_biguint(remainder);
        match n.to_u64().filter(|&small| small < 1000) {
            Some(small) => result.push_str(&spell_under_1000(small as u32, false, true)),
            None => result.push_str(&integer_to_words(&n, Gender::Masculine)),
        }
    }

    result
}

options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Gender for numbers
        gender: Gender = Gender::Masculine;
    }
}

/// Spells a number in Romanian words.
///
/// ```
/// # use n2words::ro_ro::to_cardinal;
/// assert_eq!(to_cardinal(21)?, "douăzeci și unu");
/// assert_eq!(to_cardinal(1000)?, "o mie");
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

/// Spells a number in Romanian words, with options.
///
/// ```
/// # use n2words::{ro_ro::{to_cardinal_with, CardinalOptions}, Gender};
/// let feminine = CardinalOptions { gender: Gender::Feminine };
/// assert_eq!(to_cardinal_with(1, &feminine)?, "una");
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

    if let Some(fraction) = number.fraction.as_deref() {
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

/// Builds the ordinal for tens and ones (0-99).
fn build_ordinal_tens_ones(n: u32) -> String {
    let n = n as usize;
    if n == 0 {
        return String::new();
    }
    if n < 10 {
        return ORDINAL_ONES[n].to_string();
    }
    if n < 20 {
        return ORDINAL_TEENS[n - 10].to_string();
    }

    let ones = n % 10;
    let tens = n / 10;
    if ones == 0 {
        return ORDINAL_TENS[tens].to_string();
    }
    // Compound: cardinal tens + ordinal ones (only last is ordinal)
    format!("{} și {}", TWENTIES[tens], ORDINAL_ONES[ones])
}

/// Converts an integer below 10^9 to Romanian ordinal words.
fn integer_to_ordinal(n: u64) -> String {
    if n == 0 {
        return String::new();
    }
    if n == 1 {
        return ORDINAL_ONES[1].to_string();
    }

    // Numbers < 100
    if n < 100 {
        return build_ordinal_tens_ones(n as u32);
    }

    // Numbers < 1000
    if n < 1000 {
        let hundreds = (n / 100) as usize;
        let remainder = (n % 100) as u32;
        if remainder == 0 {
            return ORDINAL_HUNDRED.to_string();
        }
        return format!(
            "{} {}",
            HUNDREDS[hundreds],
            build_ordinal_tens_ones(remainder)
        );
    }

    // Numbers < 1,000,000
    if n < 1_000_000 {
        let thousands = (n / 1000) as u32;
        let remainder = (n % 1000) as u32;

        if remainder == 0 {
            if thousands == 1 {
                return ORDINAL_THOUSAND.to_string();
            }
            return format!("{} {ORDINAL_THOUSAND}", build_scale_phrase(thousands, 1));
        }

        // Cardinal thousands + ordinal remainder
        let result = if thousands == 1 {
            "o mie".to_string()
        } else {
            build_scale_phrase(thousands, 1)
        };

        if remainder < 100 {
            return format!("{result} {}", build_ordinal_tens_ones(remainder));
        }

        let rem_hundreds = (remainder / 100) as usize;
        let rem_tens_ones = remainder % 100;
        if rem_tens_ones == 0 {
            return format!("{result} {ORDINAL_HUNDRED}");
        }
        return format!(
            "{result} {} {}",
            HUNDREDS[rem_hundreds],
            build_ordinal_tens_ones(rem_tens_ones)
        );
    }

    // Numbers >= 1,000,000
    let millions = (n / 1_000_000) as u32;
    let remainder = n % 1_000_000;

    if remainder == 0 {
        if millions == 1 {
            return ORDINAL_MILLION.to_string();
        }
        return format!("{} {ORDINAL_MILLION}", build_scale_phrase(millions, 2));
    }

    // Cardinal millions + ordinal remainder
    let result = if millions == 1 {
        "un milion".to_string()
    } else {
        build_scale_phrase(millions, 2)
    };

    format!("{result} {}", integer_to_ordinal(remainder))
}

/// Spells a number as a Romanian ordinal (masculine form).
///
/// ```
/// # use n2words::ro_ro::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "primul");
/// assert_eq!(to_ordinal(2)?, "al doilea");
/// assert_eq!(to_ordinal(21)?, "douăzeci și primul");
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
    // Below 10^9 after the check, so it fits a u64.
    Ok(integer_to_ordinal(n.to_u64().unwrap_or_default()))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::RON, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Romanian lei.
///
/// ```
/// # use n2words::ro_ro::to_currency;
/// assert_eq!(to_currency(1)?, "un leu");
/// assert_eq!(to_currency(2.5)?, "doi lei cincizeci de bani");
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

/// Spells an amount in any currency Romanian has words for.
///
/// ```
/// # use n2words::{ro_ro::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::RON };
/// assert_eq!(to_currency_with(20, &opts)?, "douăzeci de lei");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Romanian has no words for and
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
    // Every ro entry sets both gender fields.
    let major_gender = words.major_gender.unwrap_or(Gender::Masculine);
    let minor_feminine = words.minor_gender == Some(Gender::Feminine);

    let mut parts: Vec<String> = Vec::new();

    if money.negative {
        parts.push(NEGATIVE.to_string());
    }

    // Major-unit noun's own gender drives agreement.
    if !money.major.is_zero() || money.minor == 0 {
        if money.major.is_one() {
            parts.push(format!("un {}", words.major_at(0)));
        } else {
            let leu_word = integer_to_words(&money.major, major_gender);
            // Romanian inserts "de" before the noun for the CLDR `other`
            // category: count >= 20 whose last two digits are 00 or 20-99 —
            // "douăzeci de lei", "o sută de lei", but "o sută unu lei" (101)
            // and no "de" below 20. The bani path below applies the same rule;
            // build_scale_phrase uses a related per-segment predicate for scale
            // words.
            let m = (&money.major % 100u32).to_u32().unwrap_or_default();
            let needs_de = money.major >= BigUint::from(20u32) && (m == 0 || m >= 20);
            let sep = if needs_de { " de " } else { " " };
            parts.push(format!("{leu_word}{sep}{}", words.major_at(1)));
        }
    }

    // Minor-unit noun's own gender.
    if money.minor > 0 {
        let cents = money.minor;
        if cents == 1 {
            parts.push(format!("un {}", words.minor_at(0)));
        } else {
            // Every ro currency has a 2-digit minor unit, so this is 2-99;
            // spell_under_1000 matches spell_under_100 there and can't overflow.
            let ban_word = spell_under_1000(cents, minor_feminine, false);
            let sep = if cents >= 20 { " de " } else { " " };
            parts.push(format!("{ban_word}{sep}{}", words.minor_at(1)));
        }
    }

    Ok(parts.join(" "))
}

/// Registry entry for `ro-RO`.
pub static LANGUAGE: Language = Language {
    code: "ro-RO",
    name: "Romanian (Romania)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
