//! Yoruba (Nigeria) (`yo-NG`): Yoruba as used in Nigeria.
//!
//! Yoruba uses a complex vigesimal (base-20) system with:
//!
//! - Additive patterns: 1-4 added to a decade (lé = "plus")
//! - Subtractive patterns: 5-9 subtracted from the next decade (dín = "minus")
//! - Odd decades (30, 50, 70, 90) formed by subtracting 10 from the next even
//!   decade
//! - Even decades (20, 40, 60, 80, 100) are multiples of 20
//!
//! Examples: 21 = ọ̀kan lé lógún (1 + 20), 15 = àrùndínlógún (20 - 5),
//! 50 = àádọ́ta (60 - 10), 45 = àrùndínláàádọ́ta (50 - 5).
//!
//! No fixed scale ceiling: the vigesimal speller composes every magnitude.
//!
//! ```
//! use n2words::yo_ng::{to_cardinal, to_currency, to_ordinal};
//!
//! assert_eq!(to_cardinal(21)?, "ọ̀kan lé lógún");
//! assert_eq!(to_ordinal(3)?, "ìkẹẹ̀ta");
//! assert_eq!(to_currency(42.5)?, "èjì lé lógójì náírà àti àádọ́ta kọ́bọ̀");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::vocab::YO as CURRENCY_VOCAB;
use crate::currency::{Currency, assert_minor_unit};
use crate::language::{Kind, Language, form};
use crate::max::Max;
use crate::number::{Number, parse_cardinal, parse_currency, parse_ordinal};
use crate::options::options;
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

/// The cardinal ceiling: none, the vigesimal speller composes every magnitude.
pub const CARDINAL_MAX: Max = Max::Unbounded;
/// The ordinal ceiling: none.
pub const ORDINAL_MAX: Max = Max::Unbounded;
/// The currency ceiling: none.
pub const CURRENCY_MAX: Max = Max::Unbounded;

// ============================================================================
// Vocabulary
// ============================================================================

/// Basic numbers 1-10.
const ONES: [&str; 11] = [
    "",
    "ọ̀kan", // 1
    "èjì",  // 2
    "ẹ̀ta",  // 3
    "ẹ̀rin", // 4
    "àrùn", // 5
    "ẹ̀fà",  // 6
    "èje",  // 7
    "ẹ̀jọ",  // 8
    "ẹ̀sán", // 9
    "ẹ̀wá",  // 10
];

/// Numbers 11-14 (additive: X + 10, using "lá").
const TEENS_ADD: [&str; 5] = [
    "",
    "ọ̀kànlá", // 11 = 1 + 10
    "èjìlá",  // 12 = 2 + 10
    "ẹ̀talá",  // 13 = 3 + 10
    "ẹ̀rinlá", // 14 = 4 + 10
];

/// Numbers 15-19 (subtractive: 20 - X, using "dín").
const TEENS_SUB: [&str; 5] = [
    "àrùndínlógún", // 15 = 20 - 5
    "ẹ̀rìndínlógún", // 16 = 20 - 4
    "ẹ̀tadínlógún",  // 17 = 20 - 3
    "èjìdínlógún",  // 18 = 20 - 2
    "ọ̀kàndínlógún", // 19 = 20 - 1
];

// Decades (base-20 structure), indexed by decade / 10 (2 = 20 ... 10 = 100).
// Even decades are multiples of 20; odd decades subtract 10 from the next
// even decade.
const DECADES: [&str; 11] = [
    "",
    "",
    "ogún",    // 20 = 20 × 1
    "ọgbọ̀n",   // 30 = 20 + 10 (special word)
    "ogójì",   // 40 = 20 × 2
    "àádọ́ta",  // 50 = 60 - 10
    "ogóta",   // 60 = 20 × 3
    "àádọ́rin", // 70 = 80 - 10
    "ogórin",  // 80 = 20 × 4
    "àádọ́rùn", // 90 = 100 - 10
    "ọgọ́rùn",  // 100 = 20 × 5
];

// Suffixes for adding to decades (lé lógún, lé lọgbọ̀n, etc.), indexed by
// decade / 10.
const DECADE_ADD_SUFFIX: [&str; 11] = [
    "",
    "",
    "lógún",
    "lọgbọ̀n",
    "lógójì",
    "láàádọ́ta",
    "lógóta",
    "láàádọ́rin",
    "lógórin",
    "láàádọ́rùn",
    "lọ́gọ́rùn",
];

// Suffixes for subtracting from decades (dín lógójì, etc.), indexed by
// decade / 10.
const DECADE_SUB_SUFFIX: [&str; 11] = [
    "",
    "",
    "dínlógún",
    "dínlọgbọ̀n",
    "dínlógójì",
    "dínláàádọ́ta",
    "dínlógóta",
    "dínláàádọ́rin",
    "dínlógórin",
    "dínláàádọ́rùn",
    "dínlọ́gọ́rùn",
];

// Scale words
const HUNDRED: &str = "ọgọ́rùn";
/// 200 (special word, historically 200 cowries).
const TWO_HUNDRED: &str = "igba";
/// 400 (20 × 20).
const FOUR_HUNDRED: &str = "irinwó";
const THOUSAND: &str = "ẹgbẹ̀rún";
/// 10,000 (special).
const TEN_THOUSAND: &str = "ẹgbàárùn";
/// 20,000 (bag of cowries).
const TWENTY_THOUSAND: &str = "ọ̀kẹ́";
/// Million (loanword).
const MILLION: &str = "mílíọ̀nù";

const ZERO: &str = "òdo";
const NEGATIVE: &str = "àìní";
const DECIMAL_SEP: &str = "àmì";
/// "and" / "plus" connector.
const AND: &str = "ó lé";

// ============================================================================
// Ordinal Vocabulary
// ============================================================================

// Yoruba ordinals use the "ìkẹ-" prefix: ìkẹta (3rd), ìkẹrin (4th).
// First and second have special forms.
const ORDINAL_FIRST: &str = "àkọ́kọ́";
const ORDINAL_SECOND: &str = "ìkejì";
const ORDINAL_PREFIX: &str = "ìkẹ";

// ============================================================================
// Segment building
// ============================================================================

/// Words for 0-99.
fn build_under_100(n: u32) -> String {
    let n = n as usize;
    if n == 0 {
        return String::new();
    }
    if n <= 10 {
        return ONES[n].to_string();
    }

    // 11-14: additive from 10
    if n <= 14 {
        return TEENS_ADD[n - 10].to_string();
    }

    // 15-19: subtractive from 20
    if n <= 19 {
        return TEENS_SUB[n - 15].to_string();
    }

    // Exact decades
    if n % 10 == 0 {
        return DECADES[n / 10].to_string();
    }

    let decade = n / 10;
    let unit = n % 10;

    // 1-4 are added to the current decade (21 = 1 + 20, not 1 + 30)
    if unit <= 4 {
        return format!("{} lé {}", ONES[unit], DECADE_ADD_SUFFIX[decade]);
    }

    // 5-9 are subtracted from the next decade
    format!("{}{}", ONES[10 - unit], DECADE_SUB_SUFFIX[decade + 1])
}

// ============================================================================
// Cardinal
// ============================================================================

/// Words for 0-999.
fn convert_hundreds(n: u32) -> String {
    if n < 100 {
        return build_under_100(n);
    }

    let hundreds = (n / 100) as usize;
    let remainder = n % 100;

    // Special cases for 200 and 400
    if hundreds == 2 && remainder == 0 {
        return TWO_HUNDRED.to_string();
    }
    if hundreds == 4 && remainder == 0 {
        return FOUR_HUNDRED.to_string();
    }

    // Build hundreds
    let mut result = match hundreds {
        1 => HUNDRED.to_string(),
        2 => TWO_HUNDRED.to_string(),
        4 => FOUR_HUNDRED.to_string(),
        // Other hundreds: X ọgọ́rùn
        _ => format!("{} {HUNDRED}", ONES[hundreds]),
    };

    if remainder > 0 {
        result.push(' ');
        result.push_str(AND);
        result.push(' ');
        result.push_str(&build_under_100(remainder));
    }

    result
}

/// Words for a non-negative integer.
fn integer_to_words(n: &BigUint) -> String {
    if n.is_zero() {
        return ZERO.to_string();
    }

    // Fast path: numbers < 1000
    if let Some(small) = n.to_u64() {
        if small < 1000 {
            return convert_hundreds(small as u32);
        }
    }

    // Build from segments
    let mut parts: Vec<String> = Vec::new();
    let million = BigUint::from(1_000_000u32);

    // Millions: the multiplier is spelled whole, recursively.
    let below_million = if *n >= million {
        let millions = n / &million;
        if millions.is_one() {
            parts.push(format!("{MILLION} kan"));
        } else {
            parts.push(format!("{MILLION} {}", integer_to_words(&millions)));
        }
        n % &million
    } else {
        n.clone()
    };
    let mut remaining = below_million
        .to_u32()
        .expect("below a million, which fits a u32");

    // Thousands
    if remaining >= 1000 {
        let thousands = remaining / 1000;
        remaining %= 1000;

        match thousands {
            1 => parts.push(format!("{THOUSAND} kan")),
            10 => parts.push(TEN_THOUSAND.to_string()),
            20 => parts.push(TWENTY_THOUSAND.to_string()),
            _ => parts.push(format!("{THOUSAND} {}", convert_hundreds(thousands))),
        }
    }

    // Hundreds and below
    if remaining > 0 {
        let words = convert_hundreds(remaining);
        if parts.is_empty() {
            parts.push(words);
        } else {
            parts.push(format!("{AND} {words}"));
        }
    }

    parts.join(", ")
}

/// Words for the digits after the decimal point, read one by one.
fn decimal_part_to_words(decimal: &str) -> String {
    let digits: Vec<&str> = decimal
        .bytes()
        .map(|b| {
            let d = (b - b'0') as usize;
            if d == 0 { ZERO } else { ONES[d] }
        })
        .collect();
    digits.join(" ")
}

/// Spells a number in Yoruba words.
///
/// ```
/// # use n2words::yo_ng::to_cardinal;
/// assert_eq!(to_cardinal(15)?, "àrùndínlógún");
/// assert_eq!(to_cardinal(1001)?, "ẹgbẹ̀rún kan, ó lé ọ̀kan");
/// assert_eq!(to_cardinal("3.14")?, "ẹ̀ta àmì ọ̀kan ẹ̀rin");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value.
pub fn to_cardinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    let number = parse_cardinal(&value.into())?;

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

/// Ordinal words for a positive integer: àkọ́kọ́ (1st), ìkejì (2nd),
/// ìkẹẹ̀ta (3rd), ìkẹẹ̀rin (4th), ...
fn integer_to_ordinal(n: &BigUint) -> String {
    // Special forms
    if n.is_one() {
        return ORDINAL_FIRST.to_string();
    }
    if *n == BigUint::from(2u32) {
        return ORDINAL_SECOND.to_string();
    }

    // For 3+, the ìkẹ- prefix + cardinal
    format!("{ORDINAL_PREFIX}{}", integer_to_words(n))
}

/// Spells a positive integer as a Yoruba ordinal.
///
/// ```
/// # use n2words::yo_ng::to_ordinal;
/// assert_eq!(to_ordinal(1)?, "àkọ́kọ́");
/// assert_eq!(to_ordinal(2)?, "ìkejì");
/// assert_eq!(to_ordinal(3)?, "ìkẹẹ̀ta");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidOrdinal`](crate::Error::InvalidOrdinal) for zero, a
/// negative or a fraction.
pub fn to_ordinal<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    let n = parse_ordinal(&value.into())?;
    Ok(integer_to_ordinal(&n))
}

// ============================================================================
// Currency
// ============================================================================

options! {
    /// Options for [`to_currency_with`].
    pub struct CurrencyOptions {
        /// ISO 4217 currency code to name the amount in
        currency: Currency = Currency::NGN, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in Nigerian naira (náírà and kọ́bọ̀).
///
/// ```
/// # use n2words::yo_ng::to_currency;
/// assert_eq!(to_currency(42)?, "èjì lé lógójì náírà");
/// assert_eq!(to_currency(1.5)?, "ọ̀kan náírà àti àádọ́ta kọ́bọ̀");
/// assert_eq!(to_currency(-5)?, "àìní àrùn náírà");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::InvalidNumber`](crate::Error::InvalidNumber) for a malformed
/// value.
pub fn to_currency<'a>(value: impl Into<Number<'a>>) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::default())
}

/// Spells an amount in any currency Yoruba has words for.
///
/// # Errors
///
/// As [`to_currency`], plus
/// [`Error::UnsupportedCurrency`](crate::Error::UnsupportedCurrency) for a
/// currency Yoruba has no words for and
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

    // Naira part: shown if non-zero, or if there are no kobo. The nouns are
    // invariable, so the first form serves every count.
    if !money.major.is_zero() || money.minor == 0 {
        result.push_str(&integer_to_words(&money.major));
        result.push(' ');
        result.push_str(words.major_at(0));
    }

    // Kobo part
    if money.minor > 0 {
        if !money.major.is_zero() {
            result.push_str(" àti ");
        }
        result.push_str(&integer_to_words(&BigUint::from(money.minor)));
        result.push(' ');
        result.push_str(words.minor_at(0));
    }

    Ok(result)
}

/// Registry entry for `yo-NG`.
pub static LANGUAGE: Language = Language {
    code: "yo-NG",
    name: "Yoruba (Nigeria)",
    kind: Kind::Full,
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
