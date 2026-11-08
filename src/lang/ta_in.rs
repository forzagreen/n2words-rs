//! Tamil (India) (`ta-IN`).
//!
//! - Indian numbering system (ஆயிரம், லட்சம், கோடி)
//! - Tamil script
//! - 3-2-2 grouping pattern
//! - Complete word forms for 0-99
//! - Special hundred word transformations (connected vs standalone)
//! - Per-digit decimal reading
//!
//! ```
//! use n2words::ta_in::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(1234)?, "ஒரு ஆயிரம் இருநூற்று முப்பத்திநான்கு");
//! assert_eq!(to_ordinal(10)?, "பத்துஆவது");
//! assert_eq!(to_currency(42.5)?, "நாற்பத்திரண்டு ரூபாய் ஐம்பது பைசா");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::TA as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, parse_cardinal, parse_currency, parse_ordinal, split_groups_indian};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

// ============================================================================
// Vocabulary
// ============================================================================

const ZERO: &str = "பூஜ்ஜியம்";
const NEGATIVE: &str = "மைனஸ்";
const DECIMAL_SEP: &str = "புள்ளி";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

/// Ordinal suffix.
const ORDINAL_SUFFIX: &str = "ஆவது";

/// Special ordinals for the first few numbers.
const ORDINAL_SPECIAL: [&str; 7] = [
    "",
    "முதல்",
    "இரண்டாவது",
    "மூன்றாவது",
    "நான்காவது",
    "ஐந்தாவது",
    "ஆறாவது",
];

#[rustfmt::skip]
const BELOW_HUNDRED: [&str; 100] = [
    "பூஜ்ஜியம்", "ஒன்று", "இரண்டு", "மூன்று", "நான்கு", "ஐந்து", "ஆறு", "ஏழு", "எட்டு", "ஒன்பது",
    "பத்து", "பதினொன்று", "பன்னிரண்டு", "பதிமூன்று", "பதினான்கு", "பதினைந்து", "பதினாறு", "பதினேழு", "பதினெட்டு", "பத்தொன்பது",
    "இருபது", "இருபத்தொன்று", "இருபத்திரண்டு", "இருபத்திமூன்று", "இருபத்திநான்கு", "இருபத்தைந்து", "இருபத்தாறு", "இருபத்தேழு", "இருபத்தெட்டு", "இருபத்தொன்பது",
    "முப்பது", "முப்பத்தொன்று", "முப்பத்திரண்டு", "முப்பத்திமூன்று", "முப்பத்திநான்கு", "முப்பத்தைந்து", "முப்பத்தாறு", "முப்பத்தேழு", "முப்பத்தெட்டு", "முப்பத்தொன்பது",
    "நாற்பது", "நாற்பத்தொன்று", "நாற்பத்திரண்டு", "நாற்பத்திமூன்று", "நாற்பத்திநான்கு", "நாற்பத்தைந்து", "நாற்பத்தாறு", "நாற்பத்தேழு", "நாற்பத்தெட்டு", "நாற்பத்தொன்பது",
    "ஐம்பது", "ஐம்பத்தொன்று", "ஐம்பத்திரண்டு", "ஐம்பத்திமூன்று", "ஐம்பத்திநான்கு", "ஐம்பத்தைந்து", "ஐம்பத்தாறு", "ஐம்பத்தேழு", "ஐம்பத்தெட்டு", "ஐம்பத்தொன்பது",
    "அறுபது", "அறுபத்தொன்று", "அறுபத்திரண்டு", "அறுபத்திமூன்று", "அறுபத்திநான்கு", "அறுபத்தைந்து", "அறுபத்தாறு", "அறுபத்தேழு", "அறுபத்தெட்டு", "அறுபத்தொன்பது",
    "எழுபது", "எழுபத்தொன்று", "எழுபத்திரண்டு", "எழுபத்திமூன்று", "எழுபத்திநான்கு", "எழுபத்தைந்து", "எழுபத்தாறு", "எழுபத்தேழு", "எழுபத்தெட்டு", "எழுபத்தொன்பது",
    "எண்பது", "எண்பத்தொன்று", "எண்பத்திரண்டு", "எண்பத்திமூன்று", "எண்பத்திநான்கு", "எண்பத்தைந்து", "எண்பத்தாறு", "எண்பத்தேழு", "எண்பத்தெட்டு", "எண்பத்தொன்பது",
    "தொண்ணூறு", "தொண்ணூற்று ஒன்று", "தொண்ணூற்று இரண்டு", "தொண்ணூற்று மூன்று", "தொண்ணூற்று நான்கு", "தொண்ணூற்று ஐந்து", "தொண்ணூற்று ஆறு", "தொண்ணூற்று ஏழு", "தொண்ணூற்று எட்டு", "தொண்ணூற்று ஒன்பது",
];

/// Standalone hundreds (when not followed by a remainder).
const HUNDREDS: [&str; 10] = [
    "",
    "நூறு",
    "இருநூறு",
    "முன்னூறு",
    "நானூறு",
    "ஐநூறு",
    "அறுநூறு",
    "எழுநூறு",
    "எண்நூறு",
    "தொள்ளாயிரம்",
];

/// Connected form of the hundreds (when followed by a remainder).
const HUNDREDS_CONNECTED: [&str; 10] = [
    "",
    "நூற்று",
    "இருநூற்று",
    "முன்னூற்று",
    "நானூற்று",
    "ஐநூற்று",
    "அறுநூற்று",
    "எழுநூற்று",
    "எண்நூற்று",
    "தொள்ளாயிரத்து",
];

/// Ones for decimal reading.
const ONES: [&str; 9] = [
    "ஒன்று",
    "இரண்டு",
    "மூன்று",
    "நான்கு",
    "ஐந்து",
    "ஆறு",
    "ஏழு",
    "எட்டு",
    "ஒன்பது",
];

/// The word for 1 at a scale position ("one thousand", "one lakh", ...).
const ONE_AT_SCALE: &str = "ஒரு";

/// Scale words: index 0 = units (empty), 1 = thousand, 2 = lakh, etc.
const SCALE_WORDS: &[&str] = &[
    "",
    "ஆயிரம்",
    "லட்சம்",
    "கோடி",
    "அரபு",
    "கராபு",
    "நீல்",
    "பத்ம",
    "சங்கு",
];

// 3-2-2 Indian grouping: a 3-digit base segment, then 2 digits per scale word
// (SCALE_WORDS[0] = "" is the units slot). Past the table the scale word is
// dropped, which collapses the magnitude — so cap there.

/// The cardinal ceiling, derived from the scale table: `10^19`.
pub const CARDINAL_MAX: Max = Max::indian(SCALE_WORDS.len() as u32);
/// The ordinal ceiling: `10^19`.
pub const ORDINAL_MAX: Max = Max::indian(SCALE_WORDS.len() as u32);
/// The currency ceiling: `10^19`.
pub const CURRENCY_MAX: Max = Max::indian(SCALE_WORDS.len() as u32);

// ============================================================================
// Segment Building
// ============================================================================

/// Words for a 0-999 segment.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    if n < 100 {
        return BELOW_HUNDRED[n as usize].to_string();
    }

    let hundreds = (n / 100) as usize;
    let remainder = (n % 100) as usize;

    if remainder == 0 {
        return HUNDREDS[hundreds].to_string();
    }

    // Use the connected form when followed by a remainder.
    format!(
        "{} {}",
        HUNDREDS_CONNECTED[hundreds], BELOW_HUNDRED[remainder]
    )
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Words for a non-negative integer.
///
/// South Asian 3-2-2 grouping: first 3 digits, then groups of 2.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Fast path: numbers < 1000.
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return build_segment(small as u32);
        }
    }

    // Segments least-significant first: the 3-digit base, then 2 digits each.
    let segments = split_groups_indian(n, 1000, 100);

    // Build the result from most-significant to least.
    let mut result = String::new();
    for (i, &segment) in segments.iter().enumerate().rev() {
        if segment == 0 {
            continue;
        }
        if !result.is_empty() {
            result.push(' ');
        }

        if i == 0 {
            result.push_str(&build_segment(segment));
        } else if segment == 1 {
            // Use "ஒரு" for 1 at scale positions.
            result.push_str(ONE_AT_SCALE);
        } else {
            result.push_str(BELOW_HUNDRED[segment as usize]);
        }

        if i > 0 {
            if let Some(scale) = SCALE_WORDS.get(i).filter(|s| !s.is_empty()) {
                result.push(' ');
                result.push_str(scale);
            }
        }
    }

    result
}

/// Words for the decimal portion of a number, read digit by digit.
fn decimal_part_to_words(decimal: &str) -> String {
    let mut result = String::new();
    for byte in decimal.bytes() {
        let word = match byte {
            b'0' => ZERO,
            b'1'..=b'9' => ONES[(byte - b'1') as usize],
            _ => continue,
        };
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    }
    result
}

/// Spells a number in Tamil words.
///
/// ```
/// # use n2words::ta_in::to_cardinal;
/// assert_eq!(to_cardinal(999)?, "தொள்ளாயிரத்து தொண்ணூற்று ஒன்பது");
/// assert_eq!(to_cardinal(3.14)?, "மூன்று புள்ளி ஒன்று நான்கு");
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
    // The fraction is spelled digit by digit, so only the integer part has a ceiling.
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

/// Ordinal words for a positive integer.
///
/// Tamil ordinals: the first has a special form, 2-6 have special suffixes,
/// then the -ஆவது suffix.
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special ordinals for 1-6.
    if let Some(small) = n.to_u64() {
        if (1..=6).contains(&small) {
            return ORDINAL_SPECIAL[small as usize].to_string();
        }
    }

    // For 7 and above, add the suffix to the cardinal.
    let mut result = integer_to_words(n);
    result.push_str(ORDINAL_SUFFIX);
    result
}

/// Spells a positive integer as a Tamil ordinal.
///
/// ```
/// # use n2words::ta_in::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "முதல்");
/// assert_eq!(to_ordinal(2)?, "இரண்டாவது");
/// assert_eq!(to_ordinal(1001)?, "ஒரு ஆயிரம் ஒன்றுஆவது");
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
    // Ordinals build on the cardinal speller, so they share its ceiling.
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
        currency: Currency = Currency::INR, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Indian rupees.
///
/// ```
/// # use n2words::ta_in::to_currency;
/// assert_eq!(to_currency(1)?, "ஒன்று ரூபாய்");
/// assert_eq!(to_currency(0.01)?, "ஒன்று பைசா");
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

/// Spells an amount in any currency Tamil has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Tamil has no words for and
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

    // Rupees part: shown if non-zero, or if there are no paise. Tamil nouns
    // do not inflect for number here, so the first form serves every count.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(0));
    }

    // Paise part.
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

/// Registry entry for `ta-IN`.
pub static LANGUAGE: Language = Language {
    code: "ta-IN",
    name: "Tamil (India)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
