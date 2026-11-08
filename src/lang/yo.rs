//! Yoruba (`yo`): the bare-tag alias of [`yo_ng`](crate::yo_ng).
//!
//! `yo` resolves to `yo-NG`, currently the only Yoruba variant this crate
//! implements. This entry point exists so the language can be addressed
//! without a region subtag; it is not a claim that `yo-NG` speaks for every
//! Yoruba-speaking region. Cardinal and ordinal are `yo_ng`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language
//! has no country to take a default currency from, so [`to_currency`] takes
//! the currency explicitly.
//!
//! ```
//! use n2words::{yo, Currency};
//!
//! assert_eq!(yo::to_cardinal(42)?, "èjì lé lógójì");
//! assert_eq!(yo::to_ordinal(1)?, "àkọ́kọ́");
//! assert_eq!(yo::to_currency(42.5, Currency::NGN)?, "èjì lé lógójì náírà àti àádọ́ta kọ́bọ̀");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::YO as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::yo_ng;

pub use crate::yo_ng::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "yo-NG";

options! {
    /// Options for [`to_currency_with`]: `yo_ng`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Yoruba's wording.
///
/// # Errors
///
/// As [`yo_ng::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{yo::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::NGN);
/// assert_eq!(to_currency_with(1.5, &opts)?, "ọ̀kan náírà àti àádọ́ta kọ́bọ̀");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`yo_ng::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    yo_ng::to_currency_with(
        value,
        &yo_ng::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `yo`.
pub static LANGUAGE: Language = Language {
    code: "yo",
    name: "Yoruba",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
