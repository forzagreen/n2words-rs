//! Brazilian Portuguese (`pt-BR`).
//!
//! Portuguese-specific rules:
//!
//! - "e" conjunction between units, tens and hundreds: vinte e um, cento e um
//! - "cem" for exact 100, "cento" for 100 + remainder
//! - Irregular hundreds: duzentos, trezentos, quatrocentos, etc.
//! - Short scale: milhão (10^6), bilhão (10^9), trilhão (10^12)
//! - Omit "um" before "mil"
//!
//! ```
//! use n2words::pt_br::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(1001)?, "mil e um");
//! assert_eq!(to_ordinal(2345)?, "dois milésimo tricentésimo quadragésimo quinto");
//! assert_eq!(to_currency(42.5)?, "quarenta e dois reais e cinquenta centavos");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::PT_BR as CURRENCY_VOCAB;
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
    "quatorze",
    "quinze",
    "dezesseis",
    "dezessete",
    "dezoito",
    "dezenove",
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

// Scale words (short scale), singular and plural, by 3-digit group:
// index 1 = thousands, 2 = millions, 3 = billions (10^9), ...
const SCALE_WORDS_SINGULAR: &[&str] = &[
    "", // 0 unused
    THOUSAND,
    "milhão",     // 2: 10^6
    "bilhão",     // 3: 10^9
    "trilhão",    // 4: 10^12
    "quatrilhão", // 5: 10^15
    "quintilhão", // 6: 10^18
    "sextilhão",  // 7: 10^21
    "setilhão",   // 8: 10^24
];
const SCALE_WORDS_PLURAL: &[&str] = &[
    "", // 0 unused
    THOUSAND,
    "milhões",     // 2: 10^6
    "bilhões",     // 3: 10^9
    "trilhões",    // 4: 10^12
    "quatrilhões", // 5: 10^15
    "quintilhões", // 6: 10^18
    "sextilhões",  // 7: 10^21
    "setilhões",   // 8: 10^24
];

// Scale ordinal words by 3-digit group (short scale).
const SCALE_ORDINAL: &[&str] = &[
    "",
    "milésimo",
    "milionésimo",
    "bilionésimo",
    "trilionésimo",
    "quatrilionésimo",
    "quintilionésimo",
    "sextilionésimo",
];

/// The cardinal ceiling, derived from the scale table: `10^27`.
///
/// Cardinal scale words reach "setilhão" (10^24), so cardinals must stay
/// below 10^27.
pub const CARDINAL_MAX: Max = Max::western(SCALE_WORDS_SINGULAR.len() as u32 - 1);
/// The ordinal ceiling: `10^24`.
///
/// The ordinal of a number whose lowest non-zero group is a scale group uses
/// `SCALE_ORDINAL`, which is shorter than the cardinal table.
pub const ORDINAL_MAX: Max = Max::western(SCALE_ORDINAL.len() as u32 - 1);
/// The currency ceiling: `10^27`.
pub const CURRENCY_MAX: Max = Max::western(SCALE_WORDS_SINGULAR.len() as u32 - 1);

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
    /// round hundred ("mil e um", "mil e quinhentos", but "mil quinhentos e um").
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

    if let Some(small) = n.to_u64() {
        // Fast path: numbers < 1000
        if small < 1000 {
            return build_segment(small as u32).word;
        }

        // Fast path: numbers < 1,000,000 (thousands)
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;

            // "mil" not "um mil"
            let mut result = if thousands == 1 {
                THOUSAND.to_string()
            } else {
                format!("{} {THOUSAND}", build_segment(thousands).word)
            };

            if remainder > 0 {
                let segment = build_segment(remainder);
                // "e" rule: below 100 or a round hundred (e.g. 500)
                result.push_str(if segment.takes_e() { " e " } else { " " });
                result.push_str(&segment.word);
            }

            return result;
        }
    }

    build_large_number_words(n)
}

/// Words for one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let first_non_zero = segments.iter().position(|&s| s != 0).unwrap_or(0);

    let mut result = String::new();
    let mut prev_was_scale = false;

    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }

        let words = build_segment(segment);
        let is_last = i == first_non_zero;

        // "e" rule: before the last segment when it is below 100 or a round
        // hundred (e.g. 500)
        if !result.is_empty() && is_last && prev_was_scale && words.takes_e() {
            result.push_str(" e");
        }

        if !result.is_empty() {
            result.push(' ');
        }

        if i == 0 {
            // Units segment
            result.push_str(&words.word);
            prev_was_scale = false;
        } else if i == 1 {
            // Thousands
            if segment != 1 {
                result.push_str(&words.word);
                result.push(' ');
            }
            result.push_str(THOUSAND);
            prev_was_scale = true;
        } else {
            // Million and above
            if segment == 1 {
                result.push_str("um ");
                result.push_str(SCALE_WORDS_SINGULAR[i]);
            } else {
                result.push_str(&words.word);
                result.push(' ');
                result.push_str(SCALE_WORDS_PLURAL[i]);
            }
            prev_was_scale = true;
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

/// Spells a number in Brazilian Portuguese words.
///
/// ```
/// # use n2words::pt_br::to_cardinal;
/// assert_eq!(to_cardinal(1100)?, "mil e cem");
/// assert_eq!(to_cardinal(-17.42)?, "menos dezessete vírgula quarenta e dois");
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

/// Ordinal words for 1000 or more. From 2000 on, the first numeral is a
/// cardinal and every later one an ordinal ("dois milésimo tricentésimo
/// quadragésimo quinto"); a round number keeps an ordinal multiplier
/// ("segundo milésimo", "décimo milésimo").
fn build_large_ordinal(n: &BigUint) -> String {
    let segments = split_groups(n, 1000);
    let lowest = segments.iter().position(|&s| s != 0).unwrap_or(0);

    let top = segments.len() - 1;
    let first = segments[top];

    if top == lowest {
        return if first == 1 {
            SCALE_ORDINAL[top].to_string()
        } else {
            format!("{} {}", build_ordinal_segment(first), SCALE_ORDINAL[top])
        };
    }

    let mut result = SCALE_ORDINAL[top].to_string();
    if first != 1 {
        result = format!("{} {result}", build_segment(first).word);
    }

    // Segments below `lowest` are zero and skipped like any other.
    for (i, &segment) in segments[..top].iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        result.push(' ');
        if i == 0 {
            result.push_str(&build_ordinal_segment(segment));
        } else if segment == 1 {
            result.push_str(SCALE_ORDINAL[i]);
        } else {
            result.push_str(&build_ordinal_segment(segment));
            result.push(' ');
            result.push_str(SCALE_ORDINAL[i]);
        }
    }

    result
}

/// Spells a positive integer as a Brazilian Portuguese ordinal.
///
/// ```
/// # use n2words::pt_br::to_ordinal;
/// assert_eq!(to_ordinal(21)?, "vigésimo primeiro");
/// assert_eq!(to_ordinal(2000)?, "segundo milésimo");
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
        /// Include "e" between major and minor units
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::BRL, table = CURRENCY_VOCAB;
    }
}

/// Whether `n` is a whole number of millions. Milhão and the scale words
/// above it are nouns: a round number of them takes "de" before what it
/// counts ("um milhão de reais"), unlike "um milhão e quinhentos mil reais".
fn is_round_millions(n: &BigUint) -> bool {
    *n >= BigUint::from(1_000_000u32) && (n % 1_000_000u32).is_zero()
}

/// Spells an amount in Brazilian reais.
///
/// ```
/// # use n2words::pt_br::to_currency;
/// assert_eq!(to_currency(1)?, "um real");
/// assert_eq!(to_currency(0.01)?, "um centavo");
/// assert_eq!(to_currency(1_000_000)?, "um milhão de reais");
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

/// Spells an amount in any currency Brazilian Portuguese has words for.
///
/// ```
/// # use n2words::{pt_br::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::USD, ..Default::default() };
/// assert_eq!(to_currency_with(42.5, &opts)?, "quarenta e dois dólares e cinquenta centavos");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Brazilian Portuguese has no words for and
/// [`Error::NoMinorUnit`](crate::Error::NoMinorUnit) for a fractional amount
/// in a currency without one (the yen).
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

    // Whole part (reais, dólares, ...)
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

    // Fractional part (centavos, ...). `assert_minor_unit` already guarantees
    // the minor amount is zero whenever the currency has no minor unit, so
    // reaching here means the minor forms exist.
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

/// Registry entry for `pt-BR`.
pub static LANGUAGE: Language = Language {
    code: "pt-BR",
    name: "Brazilian Portuguese",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
