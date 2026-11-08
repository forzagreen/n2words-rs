//! Italian (Italy) (`it-IT`).
//!
//! - Concatenation without spaces within segments ("ventotto" not "venti otto")
//! - Phonetic vowel elision: "venti" + "otto" → "ventotto"
//! - Accent on final "tre" in compounds: "ventitré"
//! - mille/mila alternation for thousands
//! - Scale words: milione/milioni, miliardo/miliardi, etc.
//! - "e" connector before simple final remainder
//!
//! ```
//! use n2words::it_it::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(23)?, "ventitré");
//! assert_eq!(to_ordinal(21)?, "ventunesimo");
//! assert_eq!(to_currency(42.5)?, "quarantadue euro e cinquanta centesimi");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::IT as CURRENCY_VOCAB;
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
    "", "uno", "due", "tre", "quattro", "cinque", "sei", "sette", "otto", "nove",
];
const TEENS: [&str; 10] = [
    "dieci",
    "undici",
    "dodici",
    "tredici",
    "quattordici",
    "quindici",
    "sedici",
    "diciassette",
    "diciotto",
    "diciannove",
];
const TENS: [&str; 10] = [
    "",
    "",
    "venti",
    "trenta",
    "quaranta",
    "cinquanta",
    "sessanta",
    "settanta",
    "ottanta",
    "novanta",
];
const HUNDREDS: [&str; 10] = [
    "",
    "cento",
    "duecento",
    "trecento",
    "quattrocento",
    "cinquecento",
    "seicento",
    "settecento",
    "ottocento",
    "novecento",
];

// Pre-elided tens stems (drop final vowel before 'uno'/'otto')
// vent- (from venti), trent- (from trenta), etc.
const TENS_STEM: [&str; 10] = [
    "", "", "vent", "trent", "quarant", "cinquant", "sessant", "settant", "ottant", "novant",
];

const ZERO: &str = "zero";
const NEGATIVE: &str = "meno";
const DECIMAL_SEP: &str = "virgola";

// Thousands
const THOUSAND_SINGULAR: &str = "mille";
const THOUSAND_PLURAL_SUFFIX: &str = "mila";

// Scale word generation
const SCALE_PREFIXES: &[&str] = &[
    "m", "b", "tr", "quadr", "quint", "sest", "sett", "ott", "nov", "dec",
];

// Long scale: each prefix yields an -ilione (10^6k) and an -iliardo (10^6k+3),
// so the table spans 6 powers of ten per prefix; past it the scale word is
// empty. it-IT shares one ceiling across all three forms.

/// The cardinal ceiling, derived from the prefix table: `10^66`.
pub const CARDINAL_MAX: Max = Max::long_scale(SCALE_PREFIXES.len() as u32);
/// The ordinal ceiling: `10^66`.
pub const ORDINAL_MAX: Max = Max::long_scale(SCALE_PREFIXES.len() as u32);
/// The currency ceiling: `10^66`.
pub const CURRENCY_MAX: Max = Max::long_scale(SCALE_PREFIXES.len() as u32);

// ============================================================================
// Ordinal vocabulary
// ============================================================================

// Irregular ordinals 1-10 (masculine form)
const ORDINAL_ONES: [&str; 11] = [
    "", "primo", "secondo", "terzo", "quarto", "quinto", "sesto", "settimo", "ottavo", "nono",
    "decimo",
];

// Ordinal suffix for 11+
const ORDINAL_SUFFIX: &str = "esimo";

// ============================================================================
// Segment building
// ============================================================================

/// Builds the segment word for a number 0-999, with Italian phonetic elision.
///
/// - Tens ending in vowel + uno/otto → drop tens vowel: ventuno, ventotto
/// - Hundreds cento + otto/ottanta → centotto, centottanta (drop 'o')
/// - Final 'tre' in compounds becomes 'tré': ventitré, trentatré
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut result = String::new();

    // Hundreds
    if hundreds > 0 {
        // Elision: *cento + otto/ottanta → *centotto/centottanta (drop final 'o')
        // Only applies when tens = 8 (ottanta) or tens = 0 and ones = 8 (otto)
        let word = HUNDREDS[hundreds];
        if tens == 8 || (tens == 0 && ones == 8) {
            // Remove final 'o' from hundreds: cento→cent, duecento→duecent, etc.
            result.push_str(&word[..word.len() - 1]);
        } else {
            result.push_str(word);
        }
    }

    // Tens and ones
    if tens == 0 && ones == 0 {
        // Nothing more (just hundreds)
    } else if tens == 1 {
        // Teens: 10-19
        result.push_str(TEENS[ones]);
    } else if tens >= 2 {
        // 20-99: handle elision for uno (1) and otto (8)
        if ones == 1 || ones == 8 {
            // Use stem form: vent + uno = ventuno, vent + otto = ventotto
            result.push_str(TENS_STEM[tens]);
            result.push_str(ONES[ones]);
        } else if ones == 3 {
            // Final tre becomes tré
            result.push_str(TENS[tens]);
            result.push_str("tré");
        } else {
            result.push_str(TENS[tens]);
            result.push_str(ONES[ones]);
        }
    } else if ones == 3 && hundreds > 0 {
        // 1-9 (tens == 0): centotré, duecentotré, etc.
        result.push_str("tré");
    } else {
        result.push_str(ONES[ones]);
    }

    result
}

/// Same as [`build_segment`] but returns "un" for 1, for scale context:
/// "un milione" not "uno milione".
fn build_segment_for_scale(n: u32) -> String {
    if n == 1 {
        return "un".to_string();
    }
    build_segment(n)
}

/// Builds the thousands word for 1-999 thousand: mille, duemila, tremila.
fn build_thousands(n: u32) -> String {
    match n {
        0 => String::new(),
        1 => THOUSAND_SINGULAR.to_string(),
        _ => build_segment_unaccented(n) + THOUSAND_PLURAL_SUFFIX,
    }
}

/// Builds a 1-999 segment that another word is joined to. Only a word stressed
/// on its last syllable carries the written accent: "ventitré", but
/// "ventitremila", "ventitremilionesimo".
fn build_segment_unaccented(n: u32) -> String {
    let mut word = build_segment(n);
    // build_segment writes "tré" for a final 3 in every compound (not 3 or 13)
    if n % 10 == 3 && n != 3 && n % 100 != 13 {
        word.pop();
        word.push('e');
    }
    word
}

// ============================================================================
// Scale words
// ============================================================================

/// The scale word for a group index (2 = milione, 3 = miliardo, ...), singular
/// or plural; empty past the prefix table.
fn scale_word(scale_index: usize, plural: bool) -> String {
    if scale_index < 2 {
        return String::new();
    }
    let prefix_index = (scale_index - 2) / 2;
    let is_iardo = (scale_index - 2) % 2 == 1;
    let Some(prefix) = SCALE_PREFIXES.get(prefix_index) else {
        return String::new();
    };
    let suffix = match (is_iardo, plural) {
        (true, false) => "iliardo",
        (true, true) => "iliardi",
        (false, false) => "ilione",
        (false, true) => "ilioni",
    };
    format!("{prefix}{suffix}")
}

// ============================================================================
// Cardinal
// ============================================================================

/// Converts a non-negative integer to Italian words.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    if let Some(small) = n.to_u64() {
        // Fast path: numbers < 1000
        if small < 1000 {
            return build_segment(small as u32);
        }

        // Fast path: numbers < 1,000,000 (thousands)
        if small < 1_000_000 {
            let thousands = (small / 1000) as u32;
            let remainder = (small % 1000) as u32;
            if remainder == 0 {
                return build_thousands(thousands);
            }
            // Concatenate thousands + remainder; a final "tre" joined to them
            // takes the accent like any compound: "milletré", "duemilatré"
            let tail = if remainder == 3 {
                "tré".to_string()
            } else {
                build_segment(remainder)
            };
            return build_thousands(thousands) + &tail;
        }
    }

    // For numbers >= 1,000,000, use scale decomposition
    build_large_number_words(n)
}

/// Builds words for numbers >= 1,000,000.
fn build_large_number_words(n: &BigUint) -> String {
    let groups = split_groups(n, 1000);
    let mut parts: Vec<String> = Vec::new();

    // Process from highest scale down
    for (scale_index, &segment) in groups.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if scale_index >= 2 {
            // Millions and above: "segment scaleWord"
            parts.push(format!(
                "{} {}",
                build_segment_for_scale(segment),
                scale_word(scale_index, segment != 1)
            ));
        } else if scale_index == 1 {
            parts.push(build_thousands(segment));
        } else {
            parts.push(build_segment(segment));
        }
    }

    join_parts_with_connector(&parts)
}

/// Joins parts with Italian connector rules: "e" before a simple
/// (non-compound) final segment.
fn join_parts_with_connector(parts: &[String]) -> String {
    let Some((last, init)) = parts.split_last() else {
        return String::new();
    };
    if init.is_empty() {
        return last.clone();
    }
    let mut result = init.join(" ");
    // A last part without a space has no scale word
    if last.contains(' ') {
        result.push(' ');
    } else {
        result.push_str(" e ");
    }
    result.push_str(last);
    result
}

/// Words for the digits after the decimal point: leading zeros one by one,
/// then the rest as a single number.
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

    // The rest as a single number
    let remainder = &decimal[zeros..];
    if !remainder.is_empty() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&integer_to_words(&digits_to_biguint(remainder)));
    }

    result
}

/// Spells a number in Italian words.
///
/// ```
/// # use n2words::it_it::to_cardinal;
/// assert_eq!(to_cardinal(28)?, "ventotto");
/// assert_eq!(to_cardinal(2000)?, "duemila");
/// assert_eq!(to_cardinal(1000000)?, "un milione");
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

/// Converts a cardinal word to ordinal form by dropping the final vowel and
/// adding -esimo.
fn cardinal_to_ordinal(cardinal: &str) -> String {
    // Handle accented 'é' at end (tré → tre + esimo = treesimo)
    if let Some(stem) = cardinal.strip_suffix('é') {
        return format!("{stem}e{ORDINAL_SUFFIX}");
    }

    // "sei" keeps its final vowel: ventisei → ventiseiesimo
    if cardinal.ends_with("sei") {
        return format!("{cardinal}{ORDINAL_SUFFIX}");
    }

    // Handle "mila" ending (duemila → duemillesimo, not duemilesimo)
    // The ordinal of thousand is "millesimo" (from mille), not "milesimo"
    if let Some(stem) = cardinal.strip_suffix("mila") {
        return format!("{stem}mill{ORDINAL_SUFFIX}");
    }

    // Handle "mille" ending (mille → millesimo)
    if let Some(stem) = cardinal.strip_suffix("mille") {
        return format!("{stem}mill{ORDINAL_SUFFIX}");
    }

    // Drop final regular vowel before adding -esimo
    let mut chars = cardinal.chars();
    if chars
        .next_back()
        .is_some_and(|last| "aeiouàèìòù".contains(last))
    {
        return format!("{}{ORDINAL_SUFFIX}", chars.as_str());
    }

    // If it doesn't end in a vowel, just add the suffix
    format!("{cardinal}{ORDINAL_SUFFIX}")
}

/// Converts a positive integer to Italian ordinal words (masculine form).
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special cases: 1-10 have irregular forms
    if let Some(small) = n.to_u64().filter(|&small| small <= 10) {
        return ORDINAL_ONES[small as usize].to_string();
    }

    if (n % 1_000_000u32).is_zero() {
        return build_scale_ordinal(n);
    }

    // For 11+, convert cardinal and apply -esimo
    cardinal_to_ordinal(&integer_to_words(n))
}

/// Builds the ordinal of a number ending in a million+ scale word. That scale
/// word forms one word with its multiplier, which drops "un" as "mille" never
/// takes it: milionesimo, duemilionesimo, miliardesimo.
fn build_scale_ordinal(n: &BigUint) -> String {
    let groups = split_groups(n, 1000);
    // Lowest non-zero group (scale_index 2 = milione, 3 = miliardo, ...)
    let scale_index = groups
        .iter()
        .enumerate()
        .skip(2)
        .find(|&(_, &g)| g != 0)
        .map_or(2, |(i, _)| i);
    let segment = groups[scale_index];
    let divisor = BigUint::from(1000u32).pow(scale_index as u32);
    let higher = n - BigUint::from(segment) * divisor;

    let multiplier = if segment == 1 {
        String::new()
    } else {
        build_segment_unaccented(segment)
    };
    // milione → milion-esimo, miliardo → miliard-esimo
    let mut scale = scale_word(scale_index, false);
    scale.pop();
    let ordinal = format!("{multiplier}{scale}{ORDINAL_SUFFIX}");

    if higher.is_zero() {
        ordinal
    } else {
        format!("{} {ordinal}", integer_to_words(&higher))
    }
}

/// Spells a number as an Italian ordinal (masculine form).
///
/// Italian ordinals: primo, secondo, terzo... (1-10 irregular); for 11+ the
/// cardinal word drops its final vowel and takes -esimo.
///
/// ```
/// # use n2words::it_it::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "primo");
/// assert_eq!(to_ordinal(11)?, "undicesimo");
/// assert_eq!(to_ordinal(1000)?, "millesimo");
/// assert_eq!(to_ordinal(1000000)?, "milionesimo");
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
    // Ordinals are derived from the cardinal speller, so they share its ceiling.
    ORDINAL_MAX.check(&n, None)?;
    Ok(integer_to_ordinal(&n))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// Use "e" between euros and centesimi
        and: bool = true;
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::EUR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in euros.
///
/// ```
/// # use n2words::it_it::to_currency;
/// assert_eq!(to_currency(1)?, "un euro");
/// assert_eq!(to_currency(0.01)?, "un centesimo");
/// assert_eq!(to_currency(1000000)?, "un milione di euro");
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

/// Spells an amount in any currency Italian has words for.
///
/// ```
/// # use n2words::it_it::{to_currency_with, CurrencyOptions};
/// let opts = CurrencyOptions { and: false, ..Default::default() };
/// assert_eq!(to_currency_with(42.5, &opts)?, "quarantadue euro cinquanta centesimi");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Italian has no words for and
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

    // Euros part
    if !money.major.is_zero() || money.minor == 0 {
        // Use "un" for 1 euro instead of "uno"
        if money.major.is_one() {
            result.push_str("un");
        } else {
            result.push_str(&integer_to_words(&money.major));
        }
        // A round amount of millions takes "di" before the currency, as
        // milione and miliardo are nouns: "un milione di euro"
        if money.major >= BigUint::from(1_000_000u32) && (&money.major % 1_000_000u32).is_zero() {
            result.push_str(" di ");
        } else {
            result.push(' ');
        }
        // Euro is invariable (doesn't change for plural in Italian)
        result.push_str(words.major_at(0));
    }

    // Centesimi part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(if options.and { " e " } else { " " });
        }
        // Use "un" for 1 centesimo instead of "uno"
        if money.minor == 1 {
            result.push_str("un");
        } else {
            result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        }
        result.push(' ');
        result.push_str(words.minor_at(if money.minor == 1 { 0 } else { 1 }));
    }

    Ok(result)
}

/// Registry entry for `it-IT`.
pub static LANGUAGE: Language = Language {
    code: "it-IT",
    name: "Italian (Italy)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
