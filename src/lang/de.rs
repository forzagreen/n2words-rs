//! German (`de`): the bare-tag alias of [`de_de`](crate::de_de).
//!
//! `de` resolves to de-DE, currently the only German variant this crate
//! implements. This entry point exists purely so `de` works without a region
//! subtag; it is not a claim that de-DE speaks for every German-speaking
//! region. Cardinal and ordinal are `de_de`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{de, Currency};
//!
//! assert_eq!(de::to_cardinal(21)?, "einundzwanzig");
//! assert_eq!(de::to_ordinal(3)?, "dritte");
//! assert_eq!(de::to_currency(42.5, Currency::EUR)?, "zweiundvierzig Euro und fünfzig Cent");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::DE as CURRENCY_VOCAB;
use crate::de_de;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::de_de::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "de-DE";

options! {
    /// Options for [`to_currency_with`]: `de_de`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
        /// Use "und" between euros and cents
        and: bool = true;
    }
}

/// Spells an amount in the given currency, with German's default wording.
///
/// # Errors
///
/// As [`de_de::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{de::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { and: false, ..CurrencyOptions::new(Currency::EUR) };
/// assert_eq!(to_currency_with(42.5, &opts)?, "zweiundvierzig Euro fünfzig Cent");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`de_de::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    de_de::to_currency_with(
        value,
        &de_de::CurrencyOptions {
            and: options.and,
            currency: options.currency,
        },
    )
}

/// Registry entry for `de`.
pub static LANGUAGE: Language = Language {
    code: "de",
    name: "German",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
