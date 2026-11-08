//! European Spanish (`es-ES`).
//!
//! Uses the European long scale numbering system:
//!
//! - 10^6 = millón
//! - 10^9 = mil millones (thousand millions)
//! - 10^12 = billón
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
//!   millones", "veintiún euros". A multiplier of exactly 1 is dropped before
//!   "mil" ("mil", not "un mil") but kept before "millón" ("un millón").
//!
//! ```
//! use n2words::es_es::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "veintiuno");
//! assert_eq!(to_ordinal(21)?, "vigesimoprimero");
//! assert_eq!(to_currency(42.5)?, "cuarenta y dos euros con cincuenta céntimos");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::ES as CURRENCY_VOCAB;
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
    "", "uno", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve",
];
const ONES_FEM: [&str; 10] = [
    "", "una", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve",
];
// Apocopated masculine: only the ones digit changes, and only before a noun.
// The feminine forms never apocopate ("veintiuna casas", "treinta y una casas").
const ONES_MASC_APOC: [&str; 10] = [
    "", "un", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve",
];

const TEENS: [&str; 10] = [
    "diez",
    "once",
    "doce",
    "trece",
    "catorce",
    "quince",
    "dieciséis",
    "diecisiete",
    "dieciocho",
    "diecinueve",
];

// 20-29 have special compound forms
const TWENTIES_MASC: [&str; 10] = [
    "veinte",
    "veintiuno",
    "veintidós",
    "veintitrés",
    "veinticuatro",
    "veinticinco",
    "veintiséis",
    "veintisiete",
    "veintiocho",
    "veintinueve",
];
const TWENTIES_FEM: [&str; 10] = [
    "veinte",
    "veintiuna",
    "veintidós",
    "veintitrés",
    "veinticuatro",
    "veinticinco",
    "veintiséis",
    "veintisiete",
    "veintiocho",
    "veintinueve",
];
const TWENTIES_MASC_APOC: [&str; 10] = [
    "veinte",
    "veintiún",
    "veintidós",
    "veintitrés",
    "veinticuatro",
    "veinticinco",
    "veintiséis",
    "veintisiete",
    "veintiocho",
    "veintinueve",
];

const TENS: [&str; 10] = [
    "",
    "",
    "",
    "treinta",
    "cuarenta",
    "cincuenta",
    "sesenta",
    "setenta",
    "ochenta",
    "noventa",
];

// Irregular hundreds
const HUNDREDS_MASC: [&str; 10] = [
    "",
    "ciento",
    "doscientos",
    "trescientos",
    "cuatrocientos",
    "quinientos",
    "seiscientos",
    "setecientos",
    "ochocientos",
    "novecientos",
];
const HUNDREDS_FEM: [&str; 10] = [
    "",
    "ciento",
    "doscientas",
    "trescientas",
    "cuatrocientas",
    "quinientas",
    "seiscientas",
    "setecientas",
    "ochocientas",
    "novecientas",
];

// Scale words (compound long scale)
const SCALES: &[&str] = &["millón", "billón", "trillón", "cuatrillón"];
const SCALES_PLURAL: &[&str] = &["millones", "billones", "trillones", "cuatrillones"];

/// The cardinal ceiling, derived from the scale table: `10^30`.
///
/// Each `SCALES` entry spans two 3-digit groups (X and "mil X"); with the
/// units group that is `2 * SCALES.len() + 2` groups.
pub const CARDINAL_MAX: Max = Max::long_scale(SCALES.len() as u32);
/// The ordinal ceiling: `10^9`.
///
/// Bounded lower than the cardinal: the millions multiplier is a single fused
/// segment (0-999, "dosmillonésimo").
pub const ORDINAL_MAX: Max = Max::bounded(9);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::long_scale(SCALES.len() as u32);

const THOUSAND: &str = "mil";
const ZERO: &str = "cero";
const NEGATIVE: &str = "menos";
const DECIMAL_SEP: &str = "punto";

// Ordinal vocabulary, as the RAE writes it (Ortografía 2010, DPD "ordinales").
// Spanish ordinals agree in gender: primero/primera, segundo/segunda.
// 11-29 are single words in which only the last element inflects
// (decimosexta, vigesimoprimera); from 30 on, tens and units are separate
// words that both inflect (trigésima primera).
const ORDINAL_ONES_MASC: [&str; 10] = [
    "", "primero", "segundo", "tercero", "cuarto", "quinto", "sexto", "séptimo", "octavo", "noveno",
];
const ORDINAL_ONES_FEM: [&str; 10] = [
    "", "primera", "segunda", "tercera", "cuarta", "quinta", "sexta", "séptima", "octava", "novena",
];
const ORDINAL_TEENS_MASC: [&str; 10] = [
    "décimo",
    "undécimo",
    "duodécimo",
    "decimotercero",
    "decimocuarto",
    "decimoquinto",
    "decimosexto",
    "decimoséptimo",
    "decimoctavo",
    "decimonoveno",
];
const ORDINAL_TEENS_FEM: [&str; 10] = [
    "décima",
    "undécima",
    "duodécima",
    "decimotercera",
    "decimocuarta",
    "decimoquinta",
    "decimosexta",
    "decimoséptima",
    "decimoctava",
    "decimonovena",
];
const ORDINAL_TWENTIES_MASC: [&str; 10] = [
    "vigésimo",
    "vigesimoprimero",
    "vigesimosegundo",
    "vigesimotercero",
    "vigesimocuarto",
    "vigesimoquinto",
    "vigesimosexto",
    "vigesimoséptimo",
    "vigesimoctavo",
    "vigesimonoveno",
];
const ORDINAL_TWENTIES_FEM: [&str; 10] = [
    "vigésima",
    "vigesimoprimera",
    "vigesimosegunda",
    "vigesimotercera",
    "vigesimocuarta",
    "vigesimoquinta",
    "vigesimosexta",
    "vigesimoséptima",
    "vigesimoctava",
    "vigesimonovena",
];
const ORDINAL_TENS_MASC: [&str; 10] = [
    "",
    "décimo",
    "vigésimo",
    "trigésimo",
    "cuadragésimo",
    "quincuagésimo",
    "sexagésimo",
    "septuagésimo",
    "octogésimo",
    "nonagésimo",
];
const ORDINAL_TENS_FEM: [&str; 10] = [
    "",
    "décima",
    "vigésima",
    "trigésima",
    "cuadragésima",
    "quincuagésima",
    "sexagésima",
    "septuagésima",
    "octogésima",
    "nonagésima",
];
const ORDINAL_HUNDREDS_MASC: [&str; 10] = [
    "",
    "centésimo",
    "ducentésimo",
    "tricentésimo",
    "cuadringentésimo",
    "quingentésimo",
    "sexcentésimo",
    "septingentésimo",
    "octingentésimo",
    "noningentésimo",
];
const ORDINAL_HUNDREDS_FEM: [&str; 10] = [
    "",
    "centésima",
    "ducentésima",
    "tricentésima",
    "cuadringentésima",
    "quingentésima",
    "sexcentésima",
    "septingentésima",
    "octingentésima",
    "noningentésima",
];
// Cardinal multipliers fused into "milésimo"/"millonésimo" (dosmilésimo,
// veintidosmilésimo): masculine, apocopated, and without written accents.
const FUSED_ONES: [&str; 10] = [
    "", "un", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve",
];
const FUSED_TEENS: [&str; 10] = [
    "diez",
    "once",
    "doce",
    "trece",
    "catorce",
    "quince",
    "dieciseis",
    "diecisiete",
    "dieciocho",
    "diecinueve",
];
const FUSED_TWENTIES: [&str; 10] = [
    "veinte",
    "veintiun",
    "veintidos",
    "veintitres",
    "veinticuatro",
    "veinticinco",
    "veintiseis",
    "veintisiete",
    "veintiocho",
    "veintinueve",
];
const ORDINAL_THOUSAND_MASC: &str = "milésimo";
const ORDINAL_THOUSAND_FEM: &str = "milésima";
const ORDINAL_MILLION_MASC: &str = "millonésimo";
const ORDINAL_MILLION_FEM: &str = "millonésima";

const CURRENCY_CONNECTOR: &str = "con";

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment.
///
/// `apocopate` means the segment is immediately followed by a masculine noun
/// (a scale word, a currency name), so a trailing "uno" shortens to "un" and
/// "veintiuno" to "veintiún". Ignored when feminine.
fn build_segment(n: u32, feminine: bool, apocopate: bool) -> String {
    if n == 0 {
        return String::new();
    }

    // Special case: exact 100 is "cien" (no gender)
    if n == 100 {
        return "cien".to_string();
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;
    let tens_ones = (n % 100) as usize;

    let masculine_ones = if apocopate {
        &ONES_MASC_APOC
    } else {
        &ONES_MASC
    };
    let masculine_twenties = if apocopate {
        &TWENTIES_MASC_APOC
    } else {
        &TWENTIES_MASC
    };
    let ones_words = if feminine { &ONES_FEM } else { masculine_ones };

    let mut result = String::new();

    // Hundreds
    if hundreds > 0 {
        let hundreds_words = if feminine {
            &HUNDREDS_FEM
        } else {
            &HUNDREDS_MASC
        };
        result.push_str(hundreds_words[hundreds]);
    }

    // Tens and ones
    if tens_ones > 0 {
        if !result.is_empty() {
            result.push(' ');
        }
        if tens_ones < 10 {
            // Single digit
            result.push_str(ones_words[tens_ones]);
        } else if tens_ones < 20 {
            // 10-19: teens
            result.push_str(TEENS[ones]);
        } else if tens_ones < 30 {
            // 20-29: special twenties
            let twenties = if feminine {
                &TWENTIES_FEM
            } else {
                masculine_twenties
            };
            result.push_str(twenties[ones]);
        } else if ones == 0 {
            result.push_str(TENS[tens]);
        } else {
            // 30-99: tens y ones
            result.push_str(TENS[tens]);
            result.push_str(" y ");
            result.push_str(ones_words[ones]);
        }
    }

    result
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for 0-999999, the block each long-scale word multiplies.
fn build_below_million(n: u32, feminine: bool, apocopate: bool) -> String {
    if n < 1000 {
        return build_segment(n, feminine, apocopate);
    }

    let thousands = n / 1000;
    let remainder = n % 1000;

    // "mil", not "un mil". Otherwise "mil" is a masculine noun to the
    // multiplier in front of it: masculine forms regardless of the requested
    // gender, and apocopated: "veintiún mil", "treinta y un mil".
    let mut result = if thousands == 1 {
        THOUSAND.to_string()
    } else {
        format!("{} {THOUSAND}", build_segment(thousands, false, true))
    };

    if remainder > 0 {
        result.push(' ');
        result.push_str(&build_segment(remainder, feminine, apocopate));
    }

    result
}

/// Words for a non-negative integer.
///
/// `before_noun` means the whole number is followed by a noun (a currency
/// name), so its trailing segment apocopates: "veintiún euros". A standalone
/// numeral doesn't: `to_cardinal(21)` is "veintiuno".
fn integer_to_words(n: &BigUint, feminine: bool, before_noun: bool) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    if let Some(small) = n.to_u64() {
        if small < 1_000_000 {
            return build_below_million(small as u32, feminine, before_noun);
        }
    }

    build_large_number_words(n, feminine, before_noun)
}

/// Words for one million or more.
///
/// Long scale: each scale word is a million times the previous one, so `n`
/// splits into 6-digit groups, each read whole in front of its scale word:
/// 2_500_000_000 is "dos mil quinientos millones". The multiplier is kept and
/// apocopated: "un millón", "veintiún millones", "mil un millones".
fn build_large_number_words(n: &BigUint, feminine: bool, before_noun: bool) -> String {
    // Groups least-significant first (index 0 = units, 1 = millones, ...).
    // Callers guard the magnitude (CARDINAL_MAX) before reaching here.
    let groups = split_groups(n, 1_000_000);

    let mut result = String::new();

    for (i, &group) in groups.iter().enumerate().skip(1).rev() {
        if group == 0 {
            continue;
        }
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&build_below_million(group, false, true));
        result.push(' ');
        result.push_str(if group == 1 {
            SCALES[i - 1]
        } else {
            SCALES_PLURAL[i - 1]
        });
    }

    if groups[0] > 0 {
        result.push(' ');
        result.push_str(&build_below_million(groups[0], feminine, before_noun));
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

/// Spells a number in Spanish words.
///
/// ```
/// # use n2words::es_es::to_cardinal;
/// assert_eq!(to_cardinal(1_000_000)?, "un millón");
/// assert_eq!(to_cardinal(-17.42)?, "menos diecisiete punto cuarenta y dos");
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

/// Spells a number in Spanish words, with options.
///
/// ```
/// # use n2words::{es_es::{to_cardinal_with, CardinalOptions}, Gender};
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
// Ordinal
// ============================================================================

/// Ordinal words for a 0-999 segment.
///
/// - 11th = "undécimo", 16th = "decimosexto", 21st = "vigesimoprimero"
/// - 31st = "trigésimo primero"
/// - 500th = "quingentésimo", 501st = "quingentésimo primero"
fn build_ordinal_segment(n: u32, feminine: bool) -> String {
    if n == 0 {
        return String::new();
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<&str> = Vec::with_capacity(3);

    if hundreds > 0 {
        parts.push(if feminine {
            ORDINAL_HUNDREDS_FEM[hundreds]
        } else {
            ORDINAL_HUNDREDS_MASC[hundreds]
        });
    }

    if tens == 1 {
        parts.push(if feminine {
            ORDINAL_TEENS_FEM[ones]
        } else {
            ORDINAL_TEENS_MASC[ones]
        });
    } else if tens == 2 {
        parts.push(if feminine {
            ORDINAL_TWENTIES_FEM[ones]
        } else {
            ORDINAL_TWENTIES_MASC[ones]
        });
    } else {
        if tens > 0 {
            parts.push(if feminine {
                ORDINAL_TENS_FEM[tens]
            } else {
                ORDINAL_TENS_MASC[tens]
            });
        }
        if ones > 0 {
            parts.push(if feminine {
                ORDINAL_ONES_FEM[ones]
            } else {
                ORDINAL_ONES_MASC[ones]
            });
        }
    }

    parts.join(" ")
}

/// The cardinal multiplier of a fused "milésimo"/"millonésimo" ordinal.
///
/// The compound is one word stressed on its last element, so the multiplier
/// loses its spaces and written accents, and "y" becomes "i" as in one-word
/// cardinals: dosmilésimo, veintidosmilésimo, treintaiunmilésimo.
fn fused_multiplier(n: u32) -> String {
    if n == 100 {
        return "cien".to_string();
    }

    let tens_ones = (n % 100) as usize;
    let mut result = HUNDREDS_MASC[(n / 100) as usize].to_string();

    if tens_ones < 10 {
        result.push_str(FUSED_ONES[tens_ones]);
    } else if tens_ones < 20 {
        result.push_str(FUSED_TEENS[tens_ones - 10]);
    } else if tens_ones < 30 {
        result.push_str(FUSED_TWENTIES[tens_ones - 20]);
    } else {
        result.push_str(TENS[tens_ones / 10]);
        if tens_ones % 10 > 0 {
            result.push('i');
            result.push_str(FUSED_ONES[tens_ones % 10]);
        }
    }

    result
}

/// Ordinal words for a positive integer below [`ORDINAL_MAX`].
fn integer_to_ordinal(n: u64, feminine: bool) -> String {
    let thousand_word = if feminine {
        ORDINAL_THOUSAND_FEM
    } else {
        ORDINAL_THOUSAND_MASC
    };
    let million_word = if feminine {
        ORDINAL_MILLION_FEM
    } else {
        ORDINAL_MILLION_MASC
    };

    // Fast path: numbers < 1000
    if n < 1000 {
        return build_ordinal_segment(n as u32, feminine);
    }

    // Numbers 1000-999999
    if n < 1_000_000 {
        let thousands = (n / 1000) as u32;
        let remainder = (n % 1000) as u32;

        let mut result = if thousands == 1 {
            thousand_word.to_string()
        } else {
            format!("{}{thousand_word}", fused_multiplier(thousands))
        };

        if remainder > 0 {
            result.push(' ');
            result.push_str(&build_ordinal_segment(remainder, feminine));
        }

        return result;
    }

    // Numbers >= 1,000,000
    let millions = (n / 1_000_000) as u32;
    let remainder = n % 1_000_000;

    let mut result = if millions == 1 {
        million_word.to_string()
    } else {
        format!("{}{million_word}", fused_multiplier(millions))
    };

    if remainder > 0 {
        result.push(' ');
        result.push_str(&integer_to_ordinal(remainder, feminine));
    }

    result
}

options! {
    /// Options for [`to_ordinal_with`].
    pub struct OrdinalOptions {
        /// Grammatical gender
        gender: Gender = Gender::Masculine;
    }
}

/// Spells a positive integer as a Spanish ordinal.
///
/// ```
/// # use n2words::es_es::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "primero");
/// assert_eq!(to_ordinal(2000)?, "dosmilésimo");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidOrdinal`](crate::Error::InvalidOrdinal) for zero, a
/// negative or a fraction; [`Error::OutOfRange`](crate::Error::OutOfRange) at
/// or past [`ORDINAL_MAX`].
pub fn to_ordinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_ordinal_with(value, &OrdinalOptions::default())
}

/// Spells a positive integer as a Spanish ordinal, with options.
///
/// Spanish ordinals agree in gender with the noun they modify.
///
/// ```
/// # use n2words::{es_es::{to_ordinal_with, OrdinalOptions}, Gender};
/// let feminine = OrdinalOptions { gender: Gender::Feminine };
/// assert_eq!(to_ordinal_with(31, &feminine)?, "trigésima primera");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_ordinal`].
pub fn to_ordinal_with<'a>(
    value: impl Into<Number<'a>>,
    options: &OrdinalOptions,
) -> Result<String> {
    let n = parse_ordinal(&value.into())?;
    ORDINAL_MAX.check(&n, None)?;
    let n = n
        .to_u64()
        .expect("below the ordinal ceiling, which fits a u64");
    Ok(integer_to_ordinal(n, options.gender == Gender::Feminine))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// Use "con" between euros and cents
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::EUR, table = CURRENCY_VOCAB;
    }
}

/// Whether `n` is a whole number of millions. Millón and its multiples are
/// nouns, so a round amount of them takes "de" before what it counts:
/// "un millón de euros", "mil millones de euros".
fn is_round_millions(n: &BigUint) -> bool {
    *n >= BigUint::from(1_000_000u32) && (n % 1_000_000u32).is_zero()
}

/// Spells an amount in euros.
///
/// Spanish currency uses masculine gender for euros (el euro) and masculine
/// for céntimos (el céntimo).
///
/// ```
/// # use n2words::es_es::to_currency;
/// assert_eq!(to_currency(1)?, "un euro");
/// assert_eq!(to_currency(0.99)?, "noventa y nueve céntimos");
/// assert_eq!(to_currency(1_000_000)?, "un millón de euros");
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
/// # use n2words::{es_es::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::USD, and: false };
/// assert_eq!(to_currency_with(42.5, &opts)?, "cuarenta y dos dólares cincuenta centavos");
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

/// Registry entry for `es-ES`.
pub static LANGUAGE: Language = Language {
    code: "es-ES",
    name: "European Spanish",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(with_options to_ordinal_with, OrdinalOptions, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
