//! Finnish (Finland) (`fi-FI`).
//!
//! - Compound tens+ones without spaces: "kaksikymmentäyksi" (21)
//! - Teens with "-toista" suffix
//! - Omit "yksi" before sata/tuhat but keep before miljoona+
//! - Long scale: miljoona, miljardi, biljoona
//! - Per-digit decimal reading
//!
//! ```
//! use n2words::fi_fi::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "neljäkymmentäkaksi");
//! assert_eq!(to_ordinal(42)?, "neljäkymmentätoinen");
//! assert_eq!(to_currency(42.5)?, "neljäkymmentäkaksi euroa viisikymmentä senttiä");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::FI as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, parse_cardinal, parse_currency, parse_ordinal, split_groups};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

const ONES: [&str; 10] = [
    "",
    "yksi",
    "kaksi",
    "kolme",
    "neljä",
    "viisi",
    "kuusi",
    "seitsemän",
    "kahdeksan",
    "yhdeksän",
];

const TEENS: [&str; 10] = [
    "kymmenen",
    "yksitoista",
    "kaksitoista",
    "kolmetoista",
    "neljätoista",
    "viisitoista",
    "kuusitoista",
    "seitsemäntoista",
    "kahdeksantoista",
    "yhdeksäntoista",
];

// Tens use "kymmentä" suffix
const TENS: [&str; 10] = [
    "",
    "",
    "kaksikymmentä",
    "kolmekymmentä",
    "neljäkymmentä",
    "viisikymmentä",
    "kuusikymmentä",
    "seitsemänkymmentä",
    "kahdeksankymmentä",
    "yhdeksänkymmentä",
];

const HUNDRED: &str = "sata";
const THOUSAND: &str = "tuhat";

const ZERO: &str = "nolla";
const NEGATIVE: &str = "miinus";
const DECIMAL_SEP: &str = "pilkku";

// Long scale
const SCALES: &[&str] = &["miljoona", "miljardi", "biljoona", "triljoona"];

// Supported magnitude ceiling (checked at the public entry points). SCALES is
// indexed [scale_index - 2] (units and thousands separate), so the ceiling is
// 10^((SCALES.len() + 2) * 3) = 10^18. Ordinal (cardinal + suffix) and currency
// build on the cardinal, so they share it. Decimals are read digit-by-digit
// (no ceiling).

/// The cardinal ceiling, derived from the scale table: `10^18`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32 + 1);
/// The ordinal ceiling: `10^18`.
pub const ORDINAL_MAX: Max = Max::western(SCALES.len() as u32 + 1);
/// The currency ceiling: `10^18`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32 + 1);

// ============================================================================
// Ordinal vocabulary
// ============================================================================

// Finnish ordinals: special forms for 1st-2nd, then -s suffix
const ORDINAL_FIRST: &str = "ensimmäinen";
const ORDINAL_SECOND: &str = "toinen";
const ORDINAL_SUFFIX: &str = "s"; // e.g., kolmas, neljäs

// Ordinal forms for basic numbers
const ORDINAL_ONES: [&str; 10] = [
    "",
    "ensimmäinen",
    "toinen",
    "kolmas",
    "neljäs",
    "viides",
    "kuudes",
    "seitsemäs",
    "kahdeksas",
    "yhdeksäs",
];
const ORDINAL_TEENS: [&str; 10] = [
    "kymmenes",
    "yhdestoista",
    "kahdestoista",
    "kolmastoista",
    "neljästoista",
    "viidestoista",
    "kuudestoista",
    "seitsemästoista",
    "kahdeksastoista",
    "yhdeksästoista",
];
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "kahdeskymmenes",
    "kolmaskymmenes",
    "neljäskymmenes",
    "viideskymmenes",
    "kuudeskymmenes",
    "seitsemäskymmenes",
    "kahdeksaskymmenes",
    "yhdeksäskymmenes",
];

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment. Omits "yksi" before "sata" (hundred).
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<String> = Vec::new();

    // Hundreds - omit "yksi" before sata
    if hundreds > 0 {
        if hundreds == 1 {
            parts.push(HUNDRED.to_string());
        } else {
            parts.push(format!("{} {HUNDRED}", ONES[hundreds]));
        }
    }

    // Tens and ones
    let tens_ones = n % 100;

    if tens_ones == 0 {
        // Just hundreds
    } else if tens_ones < 10 {
        parts.push(ONES[ones].to_string());
    } else if tens_ones < 20 {
        parts.push(TEENS[ones].to_string());
    } else if ones == 0 {
        parts.push(TENS[tens].to_string());
    } else {
        // Compound: "kaksikymmentäyksi" (no space)
        parts.push(format!("{}{}", TENS[tens], ONES[ones]));
    }

    parts.join(" ")
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
        // Fast path: numbers < 1000 (direct lookup)
        if small < 1000 {
            return build_segment(small as u32);
        }

        // Fast path: numbers < 1,000,000 (thousands)
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            // Omit "yksi" before tuhat
            let mut result = if thousands == 1 {
                THOUSAND.to_string()
            } else {
                format!("{} {THOUSAND}", build_segment(thousands))
            };

            if remainder > 0 {
                result.push(' ');
                result.push_str(&build_segment(remainder));
            }

            return result;
        }
    }

    // For numbers >= 1,000,000, use scale decomposition
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
        let segment_word = build_segment(segment);

        if scale_index == 0 {
            // Units segment
            parts.push(segment_word);
        } else if scale_index == 1 {
            // Thousands - omit "yksi" before tuhat
            if segment == 1 {
                parts.push(THOUSAND.to_string());
            } else {
                parts.push(format!("{segment_word} {THOUSAND}"));
            }
        } else {
            // Millions+ - keep "yksi" before scale words
            parts.push(format!("{segment_word} {}", SCALES[scale_index - 2]));
        }
    }

    parts.join(" ")
}

/// Words for the digits after the decimal point, one digit at a time.
fn decimal_part_to_words(decimal: &str) -> String {
    decimal
        .bytes()
        .map(|digit| {
            let d = (digit - b'0') as usize;
            if d == 0 { ZERO } else { ONES[d] }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Spells a number in Finnish words.
///
/// ```
/// # use n2words::fi_fi::to_cardinal;
/// assert_eq!(to_cardinal(21)?, "kaksikymmentäyksi");
/// assert_eq!(to_cardinal(1000)?, "tuhat");
/// assert_eq!(to_cardinal("3.14")?, "kolme pilkku yksi neljä");
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
    CARDINAL_MAX.check(&number.integer, None)?;

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

/// Ordinal words for 0-99. Finnish ordinals have special forms.
fn build_ordinal_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    if n < 10 {
        return ORDINAL_ONES[n as usize].to_string();
    }
    if n < 20 {
        return ORDINAL_TEENS[(n - 10) as usize].to_string();
    }

    let ones = (n % 10) as usize;
    let tens = (n / 10) as usize;

    if ones == 0 {
        return ORDINAL_TENS[tens].to_string();
    }

    // Compound: kahdeskymmenes + first/second/third etc
    // For compound ordinals, only the last part is ordinal form
    format!("{}{}", TENS[tens], ORDINAL_ONES[ones])
}

/// Ordinal words for a positive integer.
///
/// Finnish ordinals: ensimmäinen (1st), toinen (2nd), kolmas (3rd), etc.
/// For larger numbers, use cardinal + ordinal ending.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special cases
    if n.is_one() {
        return ORDINAL_FIRST.to_string();
    }
    if *n == BigUint::from(2u32) {
        return ORDINAL_SECOND.to_string();
    }

    // For numbers < 100, use ordinal forms
    if let Some(small) = n.to_u64().filter(|&small| small < 100) {
        return build_ordinal_segment(small as u32);
    }

    // For larger numbers, use cardinal form with ordinal suffix on last part
    // This is a simplification - full Finnish ordinal grammar is complex
    let mut cardinal = integer_to_words(n);
    // Add ordinal suffix approximation
    cardinal.push_str(ORDINAL_SUFFIX);
    cardinal
}

/// Spells a positive integer as a Finnish ordinal.
///
/// ```
/// # use n2words::fi_fi::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "ensimmäinen");
/// assert_eq!(to_ordinal(2)?, "toinen");
/// assert_eq!(to_ordinal(3)?, "kolmas");
/// assert_eq!(to_ordinal(10)?, "kymmenes");
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
/// Euro uses sentti as subunit (100 senttiä = 1 euro).
/// Finnish has singular/plural: 1 euro vs 2 euroa, 1 sentti vs 2 senttiä.
///
/// ```
/// # use n2words::fi_fi::to_currency;
/// assert_eq!(to_currency(1)?, "yksi euro");
/// assert_eq!(to_currency(42)?, "neljäkymmentäkaksi euroa");
/// assert_eq!(to_currency(1.5)?, "yksi euro viisikymmentä senttiä");
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

/// Spells an amount in any currency Finnish has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Finnish has no words for and
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

    // Euro part - always show if non-zero, or if no cents
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(if money.major.is_one() { 0 } else { 1 }));
    }

    // Cent part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(if money.minor == 1 { 0 } else { 1 }));
    }

    Ok(result)
}

/// Registry entry for `fi-FI`.
pub static LANGUAGE: Language = Language {
    code: "fi-FI",
    name: "Finnish (Finland)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
