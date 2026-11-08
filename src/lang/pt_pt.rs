//! European Portuguese (`pt-PT`).
//!
//! Portuguese-specific rules:
//!
//! - "e" conjunction everywhere: vinte e um, cento e um, mil e um
//! - "cem" for exact 100, "cento" for 100 + remainder
//! - Irregular hundreds: duzentos, trezentos, quatrocentos, etc.
//! - Long scale: milhão (10^6), mil milhões (10^9), bilião (10^12)
//! - Omit "um" before "mil", including "mil milhões"
//!
//! ```
//! use n2words::pt_pt::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(1_000_000_000)?, "mil milhões");
//! assert_eq!(to_ordinal(21)?, "vigésimo primeiro");
//! assert_eq!(to_currency(42.5)?, "quarenta e dois euros e cinquenta cêntimos");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::PT_PT as CURRENCY_VOCAB;
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
    "", "um", "dois", "três", "quatro", "cinco", "seis", "sete", "oito", "nove",
];
const TEENS: [&str; 10] = [
    "dez",
    "onze",
    "doze",
    "treze",
    "catorze",
    "quinze",
    "dezasseis",
    "dezassete",
    "dezoito",
    "dezanove",
];
const TENS: [&str; 10] = [
    "",
    "",
    "vinte",
    "trinta",
    "quarenta",
    "cinquenta",
    "sessenta",
    "setenta",
    "oitenta",
    "noventa",
];

// Irregular hundreds
const HUNDREDS: [&str; 10] = [
    "",
    "cento",
    "duzentos",
    "trezentos",
    "quatrocentos",
    "quinhentos",
    "seiscentos",
    "setecentos",
    "oitocentos",
    "novecentos",
];

const THOUSAND: &str = "mil";
const ZERO: &str = "zero";
const NEGATIVE: &str = "menos";
const DECIMAL_SEP: &str = "vírgula";

// Ordinal vocabulary
const ORDINAL_ONES: [&str; 10] = [
    "", "primeiro", "segundo", "terceiro", "quarto", "quinto", "sexto", "sétimo", "oitavo", "nono",
];
const ORDINAL_TEENS: [&str; 10] = [
    "décimo",
    "décimo primeiro",
    "décimo segundo",
    "décimo terceiro",
    "décimo quarto",
    "décimo quinto",
    "décimo sexto",
    "décimo sétimo",
    "décimo oitavo",
    "décimo nono",
];
const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "vigésimo",
    "trigésimo",
    "quadragésimo",
    "quinquagésimo",
    "sexagésimo",
    "septuagésimo",
    "octogésimo",
    "nonagésimo",
];
const ORDINAL_HUNDREDS: [&str; 10] = [
    "",
    "centésimo",
    "ducentésimo",
    "tricentésimo",
    "quadringentésimo",
    "quingentésimo",
    "sexcentésimo",
    "septingentésimo",
    "octingentésimo",
    "nongentésimo",
];

// Long scale: each word is a million times the previous one, so the number in
// front of it runs up to 999 999 ("mil e quinhentos milhões" for 1.5 × 10^9).
// Index 0 = 10^6, 1 = 10^12, 2 = 10^18, 3 = 10^24.
const SCALES_SINGULAR: &[&str] = &["milhão", "bilião", "trilião", "quatrilião"];
const SCALES_PLURAL: &[&str] = &["milhões", "biliões", "triliões", "quatriliões"];

// Scale ordinal words by 3-digit group (long scale).
const SCALE_ORDINAL: &[&str] = &[
    "",
    "milésimo",
    "milionésimo",
    "mil milionésimo",
    "bilionésimo",
    "mil bilionésimo",
    "trilionésimo",
];

/// The cardinal ceiling, derived from the scale table: `10^30`.
///
/// The last scale word (quatrilião, 10^24) takes a multiplier up to 999 999.
pub const CARDINAL_MAX: Max = Max::long_scale(SCALES_SINGULAR.len() as u32);
/// The ordinal ceiling: `10^21`.
///
/// The ordinal of a number whose lowest non-zero group is a scale group uses
/// `SCALE_ORDINAL`, which is shorter than the cardinal table.
pub const ORDINAL_MAX: Max = Max::western(SCALE_ORDINAL.len() as u32 - 1);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::long_scale(SCALES_SINGULAR.len() as u32);

// ============================================================================
// Segment building
// ============================================================================

/// A 0-999 segment's words, with the flags the "e" rule needs.
struct Segment {
    word: String,
    /// A round hundred: "cem", "duzentos", ...
    is_exact_hundred: bool,
    /// 100 or more.
    starts_with_hundreds: bool,
}

impl Segment {
    /// Whether "e" precedes this segment after a scale word: below 100 or a
    /// round hundred ("mil e um", "mil e oitocentos", but "mil oitocentos e um").
    fn takes_e(&self) -> bool {
        !self.starts_with_hundreds || self.is_exact_hundred
    }
}

/// Words for a 0-999 segment with the Portuguese "e" rules.
fn build_segment(n: u32) -> Segment {
    if n == 0 {
        return Segment {
            word: String::new(),
            is_exact_hundred: false,
            starts_with_hundreds: false,
        };
    }

    // Special case: exact 100 is "cem"
    if n == 100 {
        return Segment {
            word: "cem".to_string(),
            is_exact_hundred: true,
            starts_with_hundreds: true,
        };
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut word = String::new();

    // Hundreds
    if hundreds > 0 {
        word.push_str(HUNDREDS[hundreds]);
    }

    // Tens and ones
    let tens_ones = if tens == 1 {
        // Teens (10-19)
        TEENS[ones].to_string()
    } else if tens >= 2 {
        if ones > 0 {
            // Tens + ones with "e": "vinte e um"
            format!("{} e {}", TENS[tens], ONES[ones])
        } else {
            TENS[tens].to_string()
        }
    } else if ones > 0 {
        ONES[ones].to_string()
    } else {
        String::new()
    };

    // Join hundreds with "e": "cento e um", "duzentos e trinta e um"
    if !tens_ones.is_empty() {
        if !word.is_empty() {
            word.push_str(" e ");
        }
        word.push_str(&tens_ones);
    }

    Segment {
        word,
        is_exact_hundred: hundreds > 0 && tens == 0 && ones == 0,
        starts_with_hundreds: n >= 100,
    }
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Fast path: numbers < 1,000,000
    if let Some(small) = n.to_u64() {
        if small < 1_000_000 {
            return build_group(small as u32);
        }
    }

    build_large_number_words(n)
}

/// Words for 1-999 999, the multiplier in front of a scale word.
fn build_group(n: u32) -> String {
    if n < 1000 {
        return build_segment(n).word;
    }

    let thousands = n / 1000;
    let remainder = n % 1000;

    // "mil" not "um mil"
    let mut result = if thousands == 1 {
        THOUSAND.to_string()
    } else {
        format!("{} {THOUSAND}", build_segment(thousands).word)
    };

    if remainder > 0 {
        let segment = build_segment(remainder);
        // "e" before a remainder below 100 or a round hundred: "mil e um",
        // "mil e oitocentos"
        result.push_str(if segment.takes_e() { " e " } else { " " });
        result.push_str(&segment.word);
    }

    result
}

/// Words for one million or more, by 6-digit long-scale groups.
fn build_large_number_words(n: &BigUint) -> String {
    // 6-digit groups, least significant first (index 0 = below a million,
    // 1 = milhões, 2 = biliões, ...)
    let groups = split_groups(n, 1_000_000);
    let lowest = groups.iter().position(|&g| g != 0).unwrap_or(0);

    let mut result = String::new();

    for (i, &group) in groups.iter().enumerate().rev() {
        if group == 0 {
            continue;
        }

        if !result.is_empty() {
            // "e" before the last group when its only non-zero 3 digits are
            // below 100 or a round hundred: "um milhão e quinhentos mil",
            // "mil milhões e cem"
            let upper = group / 1000;
            let lower = group % 1000;
            let single = if upper == 0 {
                lower
            } else if lower == 0 {
                upper
            } else {
                0
            };
            let takes_e = i == lowest && single > 0 && (single < 100 || single % 100 == 0);
            result.push_str(if takes_e { " e " } else { " " });
        }

        if i == 0 {
            result.push_str(&build_group(group));
        } else if group == 1 {
            result.push_str("um ");
            result.push_str(SCALES_SINGULAR[i - 1]);
        } else {
            result.push_str(&build_group(group));
            result.push(' ');
            result.push_str(SCALES_PLURAL[i - 1]);
        }
    }

    result
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

/// Spells a number in European Portuguese words.
///
/// ```
/// # use n2words::pt_pt::to_cardinal;
/// assert_eq!(to_cardinal(21)?, "vinte e um");
/// assert_eq!(to_cardinal(1800)?, "mil e oitocentos");
/// assert_eq!(to_cardinal(1_500_000_000)?, "mil e quinhentos milhões");
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
    // Both the integer part and the fraction's significant digits are spelled
    // via the scale builder, so both must clear the ceiling.
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

/// Ordinal words for a 0-999 segment.
fn build_ordinal_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<&str> = Vec::with_capacity(3);

    // Hundreds ordinal
    if hundreds > 0 {
        parts.push(ORDINAL_HUNDREDS[hundreds]);
    }

    // Tens and ones
    if tens == 1 {
        // 10-19: décimo, décimo primeiro, ...
        parts.push(ORDINAL_TEENS[ones]);
    } else if tens >= 2 {
        parts.push(ORDINAL_TENS[tens]);
        if ones > 0 {
            parts.push(ORDINAL_ONES[ones]);
        }
    } else if ones > 0 {
        parts.push(ORDINAL_ONES[ones]);
    }

    parts.join(" ")
}

/// Ordinal words for 1-999 999 after another ordinal: "milésimo centésimo".
fn build_ordinal_group(n: u32) -> String {
    if n < 1000 {
        return build_ordinal_segment(n);
    }

    let thousands = n / 1000;
    let remainder = n % 1000;
    let words = if thousands == 1 {
        SCALE_ORDINAL[1].to_string()
    } else {
        format!("{} {}", build_ordinal_segment(thousands), SCALE_ORDINAL[1])
    };
    if remainder == 0 {
        words
    } else {
        format!("{words} {}", build_ordinal_segment(remainder))
    }
}

/// Ordinal words for 1000 or more. From 2000 on, the first numeral is a
/// cardinal and every later one an ordinal ("dois milésimo tricentésimo
/// quadragésimo quinto"); a round number keeps an ordinal multiplier
/// ("segundo milésimo", "décimo milésimo").
fn build_large_ordinal(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let lowest = segments.iter().position(|&s| s != 0).unwrap_or(0);

    // A "mil" group of the long scale (odd index from 3: mil milhões, mil
    // biliões) joins the group below it, as in the cardinal: 1.5 × 10^9 is
    // "mil e quinhentos milionésimo".
    let mut i = segments.len() - 1;
    let mut first = segments[i];
    if i >= 3 && i % 2 == 1 && segments[i - 1] != 0 {
        i -= 1;
        first = first * 1000 + segments[i];
    } else if i == lowest {
        return if first == 1 {
            SCALE_ORDINAL[i].to_string()
        } else {
            format!("{} {}", build_ordinal_segment(first), SCALE_ORDINAL[i])
        };
    }

    let mut result = SCALE_ORDINAL[i].to_string();
    if first != 1 {
        result = format!("{} {result}", build_group(first));
    }

    // After the first numeral, a "mil" group always reads with the group below
    // it, as an ordinal of thousands: "bilionésimo milésimo milionésimo",
    // "bilionésimo milésimo quingentésimo milionésimo".
    while i > lowest {
        i -= 1;
        let mut value = segments[i];
        if i >= 3 && i % 2 == 1 {
            i -= 1;
            value = value * 1000 + segments[i];
        }
        if value == 0 {
            continue;
        }
        result.push(' ');
        if i == 0 {
            result.push_str(&build_ordinal_segment(value));
        } else if value == 1 {
            result.push_str(SCALE_ORDINAL[i]);
        } else {
            result.push_str(&build_ordinal_group(value));
            result.push(' ');
            result.push_str(SCALE_ORDINAL[i]);
        }
    }

    result
}

/// Spells a positive integer as a European Portuguese ordinal.
///
/// ```
/// # use n2words::pt_pt::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "primeiro");
/// assert_eq!(to_ordinal(2345)?, "dois milésimo tricentésimo quadragésimo quinto");
/// assert_eq!(to_ordinal(1_000_000_000)?, "mil milionésimo");
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

    // Fast path: 1-999
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return Ok(build_ordinal_segment(small as u32));
        }
    }

    Ok(build_large_ordinal(&n))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// Include "e" between euros and cents
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::EUR, table = CURRENCY_VOCAB;
    }
}

/// Whether `n` is a whole number of millions. Milhão and the scale words
/// above it are nouns: a round number of them takes "de" before what it
/// counts ("um milhão de euros"), unlike "um milhão e quinhentos mil euros".
fn is_round_millions(n: &BigUint) -> bool {
    *n >= BigUint::from(1_000_000u32) && (n % 1_000_000u32).is_zero()
}

/// Spells an amount in euros.
///
/// ```
/// # use n2words::pt_pt::to_currency;
/// assert_eq!(to_currency(1)?, "um euro");
/// assert_eq!(to_currency(0.01)?, "um cêntimo");
/// assert_eq!(to_currency(1_000_000_000)?, "mil milhões de euros");
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

/// Spells an amount in any currency European Portuguese has words for.
///
/// ```
/// # use n2words::pt_pt::{to_currency_with, CurrencyOptions};
/// let opts = CurrencyOptions { and: false, ..Default::default() };
/// assert_eq!(to_currency_with(42.5, &opts)?, "quarenta e dois euros cinquenta cêntimos");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency European Portuguese has no words for and
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

    let has_major = !money.major.is_zero();
    let has_minor = money.minor > 0;

    if !has_major && !has_minor {
        return Ok(format!("{ZERO} {}", words.major_at(1)));
    }

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }

    if has_major {
        let preposition = if is_round_millions(&money.major) {
            " de "
        } else {
            " "
        };
        result.push_str(&integer_to_words(&money.major));
        result.push_str(preposition);
        result.push_str(words.major_at(if money.major.is_one() { 0 } else { 1 }));
    }

    // `assert_minor_unit` already guarantees the minor amount is zero whenever
    // the currency has no minor unit, so reaching here means the forms exist.
    if has_minor {
        if has_major {
            result.push_str(if options.and { " e " } else { " " });
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(if money.minor == 1 { 0 } else { 1 }));
    }

    Ok(result)
}

/// Registry entry for `pt-PT`.
pub static LANGUAGE: Language = Language {
    code: "pt-PT",
    name: "European Portuguese",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
