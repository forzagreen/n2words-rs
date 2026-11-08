//! Norwegian Bokmål (`nb`): the bare-tag alias of [`nb_no`](crate::nb_no).
//!
//! `nb` resolves to `nb-NO`, currently the only Norwegian Bokmål variant this crate
//! implements; it exists so the language can be named without a region
//! subtag, not as a claim that `nb-NO` speaks for every Norwegian Bokmål-speaking
//! region. Cardinal and ordinal are `nb_no`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{nb, Currency};
//!
//! assert_eq!(nb::to_cardinal(42)?, "førti-to");
//! assert_eq!(nb::to_currency(42.5, Currency::NOK)?, "førti-to kroner og femti øre");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::NB as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::nb_no;
use crate::number::Number;
use crate::options::options;

pub use crate::nb_no::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "nb-NO";

options! {
    /// Options for [`to_currency_with`]: `nb_no`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Norwegian Bokmål's default wording.
///
/// # Errors
///
/// As [`nb_no::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{nb::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::NOK);
/// assert_eq!(to_currency_with(1.01, &opts)?, "en krone og en øre");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`nb_no::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    nb_no::to_currency_with(
        value,
        &nb_no::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `nb`.
pub static LANGUAGE: Language = Language {
    code: "nb",
    name: "Norwegian Bokmål",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
