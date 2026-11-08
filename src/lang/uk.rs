//! Ukrainian (`uk`): the bare-tag alias of [`uk_ua`](crate::uk_ua).
//!
//! `uk` resolves to `uk-UA`, currently the only Ukrainian variant this crate
//! implements. The alias exists so the bare tag works without a region
//! subtag; it is not a claim that `uk-UA` speaks for every Ukrainian-speaking
//! region. Cardinal and ordinal are `uk_ua`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{uk, Currency};
//!
//! assert_eq!(uk::to_cardinal(21)?, "двадцять один");
//! assert_eq!(uk::to_currency(42.5, Currency::UAH)?, "сорок двi гривнi п'ятдесят копiйок");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::UK as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::uk_ua;

pub use crate::uk_ua::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "uk-UA";

options! {
    /// Options for [`to_currency_with`]: `uk_ua`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Ukrainian's default wording.
///
/// # Errors
///
/// As [`uk_ua::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{uk::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::UAH);
/// assert_eq!(to_currency_with(1, &opts)?, "одна гривня");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`uk_ua::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    uk_ua::to_currency_with(
        value,
        &uk_ua::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `uk`.
pub static LANGUAGE: Language = Language {
    code: "uk",
    name: "Ukrainian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
