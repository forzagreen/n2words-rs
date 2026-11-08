//! French (France) (`fr-FR`).
//!
//! - Vigesimal patterns: 70 = soixante-dix, 80 = quatre-vingts, 90 = quatre-vingt-dix
//! - "et" conjunction: vingt et un (21), soixante et onze (71), but NOT quatre-vingt-un
//! - Pluralization: "cents" loses its 's' when followed by more digits
//! - Long scale with -ard forms: milliard, billiard, trilliard
//! - "un" omitted before mille
//! - Optional hundred pairing for 1100-1999: dix-neuf cents (1900), onze cent un (1101)
//!
//! ```
//! use n2words::fr_fr::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "vingt et un");
//! assert_eq!(to_ordinal(21)?, "vingt et unième");
//! assert_eq!(to_currency(42.5)?, "quarante-deux euros et cinquante centimes");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::FR as CURRENCY_VOCAB;
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
    "", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf",
];
const TEENS: [&str; 10] = [
    "dix", "onze", "douze", "treize", "quatorze", "quinze", "seize", "dix-sept", "dix-huit",
    "dix-neuf",
];
const TENS: [&str; 7] = [
    "",
    "",
    "vingt",
    "trente",
    "quarante",
    "cinquante",
    "soixante",
];

// Scale words (even indices: million, billion, trillion, quadrillion)
const SCALES: &[&str] = &["million", "billion", "trillion", "quadrillion"];
const SCALES_ARD: &[&str] = &["milliard", "billiard", "trilliard", "quadrilliard"];

// Long scale: each SCALES entry has a base and an "-ard" form spanning two
// segment groups, so with the units group that's 2 * SCALES.len() + 2 groups
// of 3 digits. Ordinals and currency build on the cardinal, so they share the
// same ceiling.

/// The cardinal ceiling, derived from the scale table: `10^30`.
pub const CARDINAL_MAX: Max = Max::long_scale(SCALES.len() as u32);
/// The ordinal ceiling: `10^30`.
pub const ORDINAL_MAX: Max = Max::long_scale(SCALES.len() as u32);
/// The currency ceiling: `10^30`.
pub const CURRENCY_MAX: Max = Max::long_scale(SCALES.len() as u32);

const THOUSAND: &str = "mille";
const HUNDRED: &str = "cent";
const ZERO: &str = "zéro";
const NEGATIVE: &str = "moins";
const DECIMAL_SEP: &str = "virgule";

// ============================================================================
// Ordinal vocabulary
// ============================================================================

const ORDINAL_SUFFIX: &str = "ième";
const PREMIER: &str = "premier";

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment, with the agreement flags the callers need.
struct Segment {
    /// The words, or empty for zero.
    word: String,
    /// Ends in a plural "cents" (deux cents), which loses its 's' before mille.
    ends_with_cents: bool,
    /// Ends in "quatre-vingts", which loses its 's' before mille.
    ends_with_vingts: bool,
}

/// Builds the words for a 0-999 segment.
fn build_segment(n: u32) -> Segment {
    if n == 0 {
        return Segment {
            word: String::new(),
            ends_with_cents: false,
            ends_with_vingts: false,
        };
    }

    let tens_ones = n % 100;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<String> = Vec::new();
    let mut ends_with_cents = false;
    let mut ends_with_vingts = false;

    // Hundreds
    if hundreds > 0 {
        if hundreds == 1 {
            parts.push(HUNDRED.to_string());
        } else if tens_ones == 0 {
            // "deux cents", "trois cents" (with 's')
            parts.push(format!("{} {HUNDRED}s", ONES[hundreds]));
            ends_with_cents = true;
        } else {
            // "deux cent", "trois cent" (no 's' when followed by more)
            parts.push(format!("{} {HUNDRED}", ONES[hundreds]));
        }
    }

    // Tens and ones - vigesimal pattern
    if tens_ones == 0 {
        // Just hundreds, nothing more
    } else if tens_ones < 10 {
        // Single digit
        parts.push(ONES[tens_ones as usize].to_string());
    } else if tens_ones < 20 {
        // 10-16: regular teens; 17-19: dix-sept, dix-huit, dix-neuf
        parts.push(TEENS[(tens_ones - 10) as usize].to_string());
    } else if tens_ones < 70 {
        // 20-69: standard tens + ones
        let t = (tens_ones / 10) as usize;
        let o = (tens_ones % 10) as usize;
        if o == 0 {
            parts.push(TENS[t].to_string());
        } else if o == 1 {
            // "et un" for 21, 31, 41, 51, 61
            parts.push(format!("{} et {}", TENS[t], ONES[1]));
        } else {
            parts.push(format!("{}-{}", TENS[t], ONES[o]));
        }
    } else if tens_ones < 80 {
        // 70-79: soixante-dix, soixante et onze, soixante-douze...
        let remainder = tens_ones - 60;
        if remainder == 11 {
            // 71: soixante et onze
            parts.push("soixante et onze".to_string());
        } else {
            // 70, 72-79: soixante-dix, soixante-douze...
            parts.push(format!("soixante-{}", TEENS[(remainder - 10) as usize]));
        }
    } else if tens_ones == 80 {
        // 80: quatre-vingts (with 's')
        parts.push("quatre-vingts".to_string());
        ends_with_vingts = true;
    } else {
        // 81-99: quatre-vingt-un, quatre-vingt-dix...
        let remainder = tens_ones - 80;
        if remainder < 10 {
            // 81-89
            parts.push(format!("quatre-vingt-{}", ONES[remainder as usize]));
        } else {
            // 90-99
            parts.push(format!("quatre-vingt-{}", TEENS[(remainder - 10) as usize]));
        }
    }

    // Join parts with a space (between hundreds and the rest)
    Segment {
        word: parts.join(" "),
        ends_with_cents,
        ends_with_vingts,
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// The scale word for French long scale with the -ard pattern
/// (1 = thousand, 2 = million, 3 = milliard, ...), agreed with `segment`.
fn scale_word(scale_index: usize, segment: u32) -> String {
    if scale_index == 1 {
        return THOUSAND.to_string();
    }

    // Even indices (2, 4, 6, 8): million, billion, trillion, quadrillion
    // Odd indices > 1 (3, 5, 7, 9): milliard, billiard, trilliard, quadrilliard
    if scale_index % 2 == 0 {
        let Some(base_word) = SCALES.get(scale_index / 2 - 1) else {
            return String::new();
        };
        if segment > 1 {
            format!("{base_word}s")
        } else {
            (*base_word).to_string()
        }
    } else {
        let Some(ard_word) = SCALES_ARD.get((scale_index - 1) / 2 - 1) else {
            return THOUSAND.to_string();
        };
        if segment > 1 {
            format!("{ard_word}s")
        } else {
            (*ard_word).to_string()
        }
    }
}

/// Drops the plural 's' of "cents" / "quatre-vingts" before "mille".
fn strip_plural_before_mille(segment: Segment) -> String {
    let mut word = segment.word;
    if segment.ends_with_cents || segment.ends_with_vingts {
        word.pop();
    }
    word
}

/// Replaces every space with a hyphen when the hyphen separator is on.
fn hyphenate(result: String, with_hyphen: bool) -> String {
    if with_hyphen {
        result.replace(' ', "-")
    } else {
        result
    }
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint, with_hyphen: bool, hundred_pairing: bool) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    if let Some(small) = n.to_u64() {
        // Fast path: numbers < 1000
        if small < 1000 {
            return hyphenate(build_segment(small as u32).word, with_hyphen);
        }

        // Hundred pairing: 1100-1999 counted in hundreds, "onze cents" to
        // "dix-neuf cent quatre-vingt-dix-neuf". Only eleven to nineteen
        // hundreds take this form: 1000-1099 has no "dix cents", and from 2000
        // on it's always "deux mille ...". "cent" follows its usual agreement:
        // plural when it ends the number (dix-neuf cents), invariable when
        // followed (dix-neuf cent un).
        if hundred_pairing && (1100..=1999).contains(&small) {
            let num = small as u32;
            let tens_ones = num % 100;
            let mut result = format!("{} {HUNDRED}", TEENS[(num / 100 - 10) as usize]);
            if tens_ones == 0 {
                result.push('s');
            } else {
                result.push(' ');
                result.push_str(&build_segment(tens_ones).word);
            }
            return hyphenate(result, with_hyphen);
        }

        // Fast path: numbers < 1,000,000 (thousands)
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;
            let sep = if with_hyphen { '-' } else { ' ' };

            let mut result = if thousands == 1 {
                // "mille" not "un mille"
                THOUSAND.to_string()
            } else {
                // A segment ending in "cents" or "vingts" loses its 's' before mille
                let mut words = strip_plural_before_mille(build_segment(thousands));
                words.push(sep);
                words.push_str(THOUSAND);
                words
            };

            if remainder > 0 {
                result.push(sep);
                result.push_str(&build_segment(remainder).word);
            }

            return hyphenate(result, with_hyphen);
        }
    }

    // For numbers >= 1,000,000, use scale decomposition
    build_large_number_words(n, with_hyphen)
}

/// Words for a number of one million or more, by 3-digit scale segments.
fn build_large_number_words(n: &BigUint, with_hyphen: bool) -> String {
    let segments = split_groups(n, 1000);

    let mut parts: Vec<String> = Vec::new();
    for (scale_index, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        let built = build_segment(segment);
        if scale_index == 0 {
            // Units segment
            parts.push(built.word);
        } else if scale_index == 1 {
            // Thousands: "mille" not "un mille"
            if segment == 1 {
                parts.push(THOUSAND.to_string());
            } else {
                // Strip 's' from cents/vingts before mille
                parts.push(strip_plural_before_mille(built));
                parts.push(scale_word(scale_index, segment));
            }
        } else {
            // Million and above
            parts.push(built.word);
            parts.push(scale_word(scale_index, segment));
        }
    }

    let sep = if with_hyphen { "-" } else { " " };
    hyphenate(parts.join(sep), with_hyphen)
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
fn decimal_part_to_words(decimal: &str, with_hyphen: bool) -> String {
    let sep = if with_hyphen { "-" } else { " " };
    let mut result = String::new();

    // Leading zeros
    let zeros = decimal.bytes().take_while(|&b| b == b'0').count();
    for _ in 0..zeros {
        if !result.is_empty() {
            result.push_str(sep);
        }
        result.push_str(ZERO);
    }

    // The rest as a single number
    let remainder = &decimal[zeros..];
    if !remainder.is_empty() {
        if !result.is_empty() {
            result.push_str(sep);
        }
        result.push_str(&integer_to_words(
            &digits_to_biguint(remainder),
            with_hyphen,
            false,
        ));
    }

    result
}

options! {
    /// Options for [`to_cardinal_with`].
    pub struct CardinalOptions {
        /// Use hyphens between all words
        with_hyphen_separator: bool = false;
        /// Count hundreds past ten for 1100-1999 (e.g., "dix-neuf cents" instead of "mille neuf cents")
        hundred_pairing: bool = false;
    }
}

/// Spells a number in French words.
///
/// ```
/// # use n2words::fr_fr::to_cardinal;
/// assert_eq!(to_cardinal(80)?, "quatre-vingts");
/// assert_eq!(to_cardinal(1000000)?, "un million");
/// assert_eq!(to_cardinal(-17.42)?, "moins dix-sept virgule quarante-deux");
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

/// Spells a number in French words, with options.
///
/// ```
/// # use n2words::fr_fr::{to_cardinal_with, CardinalOptions};
/// let pairing = CardinalOptions { hundred_pairing: true, ..Default::default() };
/// assert_eq!(to_cardinal_with(1900, &pairing)?, "dix-neuf cents");
/// let hyphens = CardinalOptions { with_hyphen_separator: true, ..Default::default() };
/// assert_eq!(to_cardinal_with(21602, &hyphens)?, "vingt-et-un-mille-six-cent-deux");
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

    let with_hyphen = options.with_hyphen_separator;
    let sep = if with_hyphen { "-" } else { " " };

    let mut result = String::new();
    if number.negative {
        result.push_str(NEGATIVE);
        result.push_str(sep);
    }
    result.push_str(&integer_to_words(
        &number.integer,
        with_hyphen,
        options.hundred_pairing,
    ));
    if let Some(fraction) = &number.fraction {
        result.push_str(sep);
        result.push_str(DECIMAL_SEP);
        result.push_str(sep);
        result.push_str(&decimal_part_to_words(fraction, with_hyphen));
    }
    Ok(result)
}

// ============================================================================
// Ordinal
// ============================================================================

/// Turns a cardinal word into its ordinal form:
/// - drop a final -e before -ième (quatre → quatrième)
/// - cinq → cinquième (a -u- before -ième)
/// - neuf → neuvième (f → v before -ième)
fn cardinal_to_ordinal(cardinal: &str) -> String {
    // cinq → cinquième (add 'u')
    if cardinal.ends_with("cinq") {
        return format!("{cardinal}u{ORDINAL_SUFFIX}");
    }

    // neuf → neuvième (f → v)
    if cardinal.ends_with("neuf") {
        let stem = &cardinal[..cardinal.len() - 1];
        return format!("{stem}v{ORDINAL_SUFFIX}");
    }

    // Drop the plural -s of cents/vingts/millions/etc. (quatre-vingts →
    // quatre-vingtième). "trois" and "six" also end in s, but not as a plural.
    const PLURALS: [&str; 10] = [
        "cents",
        "vingts",
        "millions",
        "milliards",
        "billions",
        "billiards",
        "trillions",
        "trilliards",
        "quadrillions",
        "quadrilliards",
    ];
    if PLURALS.iter().any(|plural| cardinal.ends_with(plural)) {
        let stem = &cardinal[..cardinal.len() - 1];
        return format!("{stem}{ORDINAL_SUFFIX}");
    }

    // Drop a final -e before adding -ième (quatre → quatrième)
    if let Some(stem) = cardinal.strip_suffix('e') {
        return format!("{stem}{ORDINAL_SUFFIX}");
    }

    // Default: just add -ième
    format!("{cardinal}{ORDINAL_SUFFIX}")
}

/// Ordinal words for a positive integer.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special case: 1 → premier
    if n.is_one() {
        return PREMIER.to_string();
    }

    // Get the cardinal form and convert it to an ordinal
    let cardinal = integer_to_words(n, false, false);

    // "un million" is a count; the rank drops the "un", as "mille" gives
    // "millième": "millionième", "milliardième" (Académie française). With
    // anything after the scale word the "un" stays: "un million unième".
    if let Some(rest) = cardinal.strip_prefix("un ") {
        if !rest.contains(' ') {
            return cardinal_to_ordinal(rest);
        }
    }

    cardinal_to_ordinal(&cardinal)
}

/// Spells a positive integer as a French ordinal.
///
/// French ordinals: premier (1st), then cardinal + ième, with
/// quatre → quatrième, cinq → cinquième, neuf → neuvième.
///
/// ```
/// # use n2words::fr_fr::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "premier");
/// assert_eq!(to_ordinal(9)?, "neuvième");
/// assert_eq!(to_ordinal(1000)?, "millième");
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
        /// Use "et" between euros and centimes
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::EUR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in euros.
///
/// ```
/// # use n2words::fr_fr::to_currency;
/// assert_eq!(to_currency(1)?, "un euro");
/// assert_eq!(to_currency(0.99)?, "quatre-vingt-dix-neuf centimes");
/// assert_eq!(to_currency(1000000)?, "un million d'euros");
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

/// Spells an amount in any currency French has words for.
///
/// ```
/// # use n2words::{fr_fr::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { currency: Currency::TND, and: true };
/// assert_eq!(to_currency_with("1.500", &opts)?, "un dinar et cinq cents millimes");
/// let opts = CurrencyOptions { and: false, ..Default::default() };
/// assert_eq!(to_currency_with(42.5, &opts)?, "quarante-deux euros cinquante centimes");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency French has no words for and
/// [`Error::NoMinorUnit`](crate::Error::NoMinorUnit) for a fractional amount
/// in a currency without one.
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    // The currency decides how many decimal digits the parser keeps: TND read
    // at the default 2 would turn "1.500" into 50 millimes instead of 500.
    let words = CURRENCY_VOCAB.words(options.currency)?;
    let money = parse_currency(&value.into(), options.currency.minor_unit_digits())?;
    CURRENCY_MAX.check(&money.major, None)?; // minor units are <= 999, safe
    assert_minor_unit(money.minor, options.currency)?;

    let mut result = String::new();
    if money.negative {
        result.push_str(NEGATIVE);
        result.push(' ');
    }

    // Major units
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major, false, false));
        // In French, 0 and 1 are singular: "zéro euro", "un euro"
        let unit = if money.major <= BigUint::from(1u32) {
            words.major_at(0)
        } else {
            words.major_at(1)
        };
        // Million, milliard and the scale words above them are nouns, so a
        // round amount of them takes "de" before what it counts, elided before
        // a vowel: "un million d'euros", "deux milliards de dinars".
        if money.major >= BigUint::from(1_000_000u32) && (&money.major % 1_000_000u32).is_zero() {
            if unit.starts_with(['a', 'e', 'i', 'o', 'u', 'é', 'è', 'ê']) {
                result.push_str(" d'");
            } else {
                result.push_str(" de ");
            }
        } else {
            result.push(' ');
        }
        result.push_str(unit);
    }

    // Minor units
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(if options.and { " et " } else { " " });
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor), false, false));
        result.push(' ');
        result.push_str(words.minor_at(if money.minor == 1 { 0 } else { 1 }));
    }

    Ok(result)
}

/// Registry entry for `fr-FR`.
pub static LANGUAGE: Language = Language {
    code: "fr-FR",
    name: "French (France)",
    kind: Kind::Full,
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
