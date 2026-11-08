//! Dutch (`nl`): the bare-tag alias of [`nl_nl`](crate::nl_nl).
//!
//! `nl` resolves to nl-NL, currently the only Dutch variant this crate
//! implements. This entry point exists so `nl` works without a region
//! subtag; it is not a claim that nl-NL speaks for every Dutch-speaking
//! region. Cardinal and ordinal are `nl_nl`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{nl, Currency};
//!
//! assert_eq!(nl::to_cardinal(21)?, "eenentwintig");
//! assert_eq!(nl::to_ordinal(100)?, "honderdste");
//! assert_eq!(nl::to_currency(42.5, Currency::EUR)?, "tweeënveertig euro en vijftig cent");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::NL as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::nl_nl;
use crate::number::Number;
use crate::options::options;

pub use crate::nl_nl::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "nl-NL";

options! {
    /// Options for [`to_currency_with`]: `nl_nl`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
        /// Include "en" between euros and cents
        and: bool = true;
    }
}

/// Spells an amount in the given currency, with Dutch's default wording.
///
/// # Errors
///
/// As [`nl_nl::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{nl::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { and: false, ..CurrencyOptions::new(Currency::EUR) };
/// assert_eq!(to_currency_with(42.5, &opts)?, "tweeënveertig euro vijftig cent");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`nl_nl::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    nl_nl::to_currency_with(
        value,
        &nl_nl::CurrencyOptions {
            and: options.and,
            currency: options.currency,
        },
    )
}

/// Registry entry for `nl`.
pub static LANGUAGE: Language = Language {
    code: "nl",
    name: "Dutch",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
