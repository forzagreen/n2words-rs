//! Russian (`ru`): the bare-tag alias of [`ru_ru`](crate::ru_ru).
//!
//! `ru` resolves to `ru-RU`, currently the only Russian variant this crate
//! implements. The alias exists so the bare tag works without a region
//! subtag; it is not a claim that `ru-RU` speaks for every Russian-speaking
//! region. Cardinal and ordinal are `ru_ru`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{ru, Currency};
//!
//! assert_eq!(ru::to_cardinal(21)?, "двадцать один");
//! assert_eq!(ru::to_currency(42.5, Currency::RUB)?, "сорок два рубля и пятьдесят копеек");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::RU as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::ru_ru;

pub use crate::ru_ru::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ru-RU";

options! {
    /// Options for [`to_currency_with`]: `ru_ru`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
        /// Use "и" between rubles and kopecks
        and: bool = true;
    }
}

/// Spells an amount in the given currency, with Russian's default wording.
///
/// # Errors
///
/// As [`ru_ru::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{ru::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { and: false, ..CurrencyOptions::new(Currency::RUB) };
/// assert_eq!(to_currency_with(42.5, &opts)?, "сорок два рубля пятьдесят копеек");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ru_ru::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ru_ru::to_currency_with(
        value,
        &ru_ru::CurrencyOptions {
            and: options.and,
            currency: options.currency,
        },
    )
}

/// Registry entry for `ru`.
pub static LANGUAGE: Language = Language {
    code: "ru",
    name: "Russian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
