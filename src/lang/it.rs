//! Italian (`it`): the bare-tag alias of [`it_it`](crate::it_it).
//!
//! `it` resolves to `it-IT`, currently the only Italian variant this crate
//! implements. The alias exists so the bare tag works without a region
//! subtag; it is not a claim that `it-IT` speaks for every Italian-speaking
//! region. Cardinal and ordinal are `it_it`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{it, Currency};
//!
//! assert_eq!(it::to_cardinal(28)?, "ventotto");
//! assert_eq!(it::to_ordinal(21)?, "ventunesimo");
//! assert_eq!(it::to_currency(42.5, Currency::EUR)?, "quarantadue euro e cinquanta centesimi");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::IT as CURRENCY_VOCAB;
use crate::it_it;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::it_it::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "it-IT";

options! {
    /// Options for [`to_currency_with`]: `it_it`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
        /// Use "e" between euros and centesimi
        and: bool = true;
    }
}

/// Spells an amount in the given currency, with Italian's default wording.
///
/// # Errors
///
/// As [`it_it::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{it::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { and: false, ..CurrencyOptions::new(Currency::EUR) };
/// assert_eq!(to_currency_with(42.5, &opts)?, "quarantadue euro cinquanta centesimi");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`it_it::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    it_it::to_currency_with(
        value,
        &it_it::CurrencyOptions {
            and: options.and,
            currency: options.currency,
        },
    )
}

/// Registry entry for `it`.
pub static LANGUAGE: Language = Language {
    code: "it",
    name: "Italian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
