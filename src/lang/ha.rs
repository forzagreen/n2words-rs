//! Hausa (`ha`): the bare-tag alias of [`ha_ng`](crate::ha_ng).
//!
//! `ha` resolves to `ha-NG`, currently the only Hausa variant this crate
//! implements. This entry point exists so the language can be addressed
//! without a region subtag; it is not a claim that `ha-NG` speaks for every
//! Hausa-speaking region. Cardinal and ordinal are `ha_ng`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language
//! has no country to take a default currency from, so [`to_currency`] takes
//! the currency explicitly.
//!
//! ```
//! use n2words::{ha, Currency};
//!
//! assert_eq!(ha::to_cardinal(42)?, "arba'in da biyu");
//! assert_eq!(ha::to_ordinal(42)?, "na arba'in da biyu");
//! assert_eq!(ha::to_currency(42.5, Currency::NGN)?, "arba'in da biyu naira da hamsin kobo");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::HA as CURRENCY_VOCAB;
use crate::ha_ng;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::ha_ng::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ha-NG";

options! {
    /// Options for [`to_currency_with`]: `ha_ng`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Hausa's wording.
///
/// # Errors
///
/// As [`ha_ng::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{ha::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::NGN);
/// assert_eq!(to_currency_with(1.5, &opts)?, "ɗaya naira da hamsin kobo");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ha_ng::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ha_ng::to_currency_with(
        value,
        &ha_ng::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `ha`.
pub static LANGUAGE: Language = Language {
    code: "ha",
    name: "Hausa",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
