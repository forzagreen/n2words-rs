//! Greek (Greece) (`el-GR`).
//!
//! - Space-separated number composition
//! - Implicit "one" (ένα) omission before scale words
//! - Irregular hundreds (διακόσια, τριακόσια, etc.)
//! - Per-digit decimal reading
//!
//! ```
//! use n2words::el_gr::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(42)?, "σαράντα δύο");
//! assert_eq!(to_ordinal(42)?, "τεσσαρακοστός δεύτερος");
//! assert_eq!(to_currency(42.5)?, "σαράντα δύο ευρώ πενήντα λεπτά");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::EL as CURRENCY_VOCAB;
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
    "ένα",
    "δύο",
    "τρία",
    "τέσσερα",
    "πέντε",
    "έξι",
    "επτά",
    "οκτώ",
    "εννέα",
];

const TEENS: [&str; 10] = [
    "δέκα",
    "έντεκα",
    "δώδεκα",
    "δεκατρία",
    "δεκατέσσερα",
    "δεκαπέντε",
    "δεκαέξι",
    "δεκαεπτά",
    "δεκαοκτώ",
    "δεκαεννέα",
];

const TENS: [&str; 10] = [
    "",
    "",
    "είκοσι",
    "τριάντα",
    "σαράντα",
    "πενήντα",
    "εξήντα",
    "εβδομήντα",
    "ογδόντα",
    "ενενήντα",
];

// Greek has irregular hundreds
const HUNDREDS: [&str; 10] = [
    "",
    "εκατό",
    "διακόσια",
    "τριακόσια",
    "τετρακόσια",
    "πεντακόσια",
    "εξακόσια",
    "επτακόσια",
    "οκτακόσια",
    "εννιακόσια",
];

const THOUSAND: &str = "χίλια";

const ZERO: &str = "μηδέν";
const NEGATIVE: &str = "μείον";
const DECIMAL_SEP: &str = "κόμμα";

// Short scale
const SCALES: &[&str] = &["εκατομμύριο", "δισεκατομμύριο", "τρισεκατομμύριο"];

// Supported magnitude ceiling (checked at the public entry points), derived
// from the scale table.

/// The cardinal ceiling, derived from the scale table: `10^15`.
pub const CARDINAL_MAX: Max = Max::western(SCALES.len() as u32 + 1);
/// The ordinal ceiling: `10^9` (the ordinal builder stops at millions).
pub const ORDINAL_MAX: Max = Max::bounded(9);
/// The currency ceiling: `10^15`.
pub const CURRENCY_MAX: Max = Max::western(SCALES.len() as u32 + 1);

// Ordinal vocabulary
const ORDINAL_ONES: [&str; 10] = [
    "",
    "πρώτος",
    "δεύτερος",
    "τρίτος",
    "τέταρτος",
    "πέμπτος",
    "έκτος",
    "έβδομος",
    "όγδοος",
    "ένατος",
];

const ORDINAL_TEENS: [&str; 10] = [
    "δέκατος",
    "ενδέκατος",
    "δωδέκατος",
    "δέκατος τρίτος",
    "δέκατος τέταρτος",
    "δέκατος πέμπτος",
    "δέκατος έκτος",
    "δέκατος έβδομος",
    "δέκατος όγδοος",
    "δέκατος ένατος",
];

const ORDINAL_TENS: [&str; 10] = [
    "",
    "",
    "εικοστός",
    "τριακοστός",
    "τεσσαρακοστός",
    "πεντηκοστός",
    "εξηκοστός",
    "εβδομηκοστός",
    "ογδοηκοστός",
    "ενενηκοστός",
];

const ORDINAL_HUNDREDS: [&str; 10] = [
    "",
    "εκατοστός",
    "διακοσιοστός",
    "τριακοσιοστός",
    "τετρακοσιοστός",
    "πεντακοσιοστός",
    "εξακοσιοστός",
    "επτακοσιοστός",
    "οκτακοσιοστός",
    "εννιακοσιοστός",
];

const ORDINAL_THOUSAND: &str = "χιλιοστός";
const ORDINAL_MILLION: &str = "εκατομμυριοστός";

// ============================================================================
// Segment building
// ============================================================================

/// Words for a 0-999 segment.
fn build_segment(n: u32) -> String {
    if n == 0 {
        return String::new();
    }

    let ones = (n % 10) as usize;
    let tens = ((n / 10) % 10) as usize;
    let hundreds = (n / 100) as usize;

    let mut parts: Vec<String> = Vec::new();

    // Hundreds (irregular forms)
    if hundreds > 0 {
        parts.push(HUNDREDS[hundreds].to_string());
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
        parts.push(format!("{} {}", TENS[tens], ONES[ones]));
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

            // Omit "ένα" before χίλια
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
            // Thousands - omit "ένα" before χίλια
            if segment == 1 {
                parts.push(THOUSAND.to_string());
            } else {
                parts.push(format!("{segment_word} {THOUSAND}"));
            }
        } else {
            // Millions+ - omit "ένα" before scale words
            let scale_word = SCALES[scale_index - 2];
            if segment == 1 {
                parts.push(scale_word.to_string());
            } else {
                parts.push(format!("{segment_word} {scale_word}"));
            }
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

/// Spells a number in Greek words.
///
/// ```
/// # use n2words::el_gr::to_cardinal;
/// assert_eq!(to_cardinal(21)?, "είκοσι ένα");
/// assert_eq!(to_cardinal(1000)?, "χίλια");
/// assert_eq!(to_cardinal("3.14")?, "τρία κόμμα ένα τέσσερα");
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

/// Ordinal words for tens and ones (0-99).
fn build_ordinal_tens_ones(n: u32) -> String {
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
    format!("{} {}", ORDINAL_TENS[tens], ORDINAL_ONES[ones])
}

/// Ordinal words for a non-negative integer below [`ORDINAL_MAX`].
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
            return ORDINAL_HUNDREDS[hundreds].to_string();
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
            return format!("{} {ORDINAL_THOUSAND}", build_segment(thousands));
        }

        // Cardinal thousands + ordinal remainder
        let result = if thousands == 1 {
            THOUSAND.to_string()
        } else {
            format!("{} {THOUSAND}", build_segment(thousands))
        };

        if remainder < 100 {
            return format!("{result} {}", build_ordinal_tens_ones(remainder));
        }

        let rem_hundreds = (remainder / 100) as usize;
        let rem_tens_ones = remainder % 100;

        if rem_tens_ones == 0 {
            return format!("{result} {}", ORDINAL_HUNDREDS[rem_hundreds]);
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
        return format!("{} {ORDINAL_MILLION}", build_segment(millions));
    }

    // Cardinal millions + ordinal remainder
    let result = if millions == 1 {
        SCALES[0].to_string()
    } else {
        format!("{} {}", build_segment(millions), SCALES[0])
    };

    format!("{result} {}", integer_to_ordinal(remainder))
}

/// Spells a positive integer as a Greek ordinal (masculine).
///
/// ```
/// # use n2words::el_gr::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "πρώτος");
/// assert_eq!(to_ordinal(21)?, "εικοστός πρώτος");
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
    // Below 10^9 after the check, so it fits comfortably.
    Ok(integer_to_ordinal(n.to_u64().expect("checked below 10^9")))
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
/// ```
/// # use n2words::el_gr::to_currency;
/// assert_eq!(to_currency(1)?, "ένα ευρώ");
/// assert_eq!(to_currency(2.5)?, "δύο ευρώ πενήντα λεπτά");
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

/// Spells an amount in any currency Greek has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Greek has no words for and
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

    let mut parts: Vec<String> = Vec::new();

    if money.negative {
        parts.push(NEGATIVE.to_string());
    }

    // Euros
    if !money.major.is_zero() || money.minor == 0 {
        let euro_form = words.major_at(if money.major.is_one() { 0 } else { 1 });
        parts.push(format!("{} {euro_form}", integer_to_words(&money.major)));
    }

    // Cents (λεπτά)
    if money.minor > 0 {
        let cent_form = words.minor_at(if money.minor == 1 { 0 } else { 1 });
        parts.push(format!(
            "{} {cent_form}",
            integer_to_words(&BigUint::from(money.minor))
        ));
    }

    Ok(parts.join(" "))
}

/// Registry entry for `el-GR`.
pub static LANGUAGE: Language = Language {
    code: "el-GR",
    name: "Greek (Greece)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
