//! Punjabi (`pa`): the bare-tag alias of [`pa_in`](crate::pa_in).
//!
//! `pa` resolves to `pa-IN`, currently the only Punjabi variant this crate
//! implements. It exists purely so the bare tag works without a region
//! subtag; it is not a claim that `pa-IN` speaks for every Punjabi-speaking
//! region. Cardinal and ordinal are `pa_in`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{pa, Currency};
//!
//! assert_eq!(pa::to_cardinal(215)?, "ਦੋ ਸੌ ਪੰਦਰਾਂ");
//! assert_eq!(pa::to_currency(42.5, Currency::INR)?, "ਬਿਆਲੀ ਰੁਪਏ ਪੰਜਾਹ ਪੈਸੇ");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::PA as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::pa_in;

pub use crate::pa_in::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "pa-IN";

options! {
    /// Options for [`to_currency_with`]: `pa_in`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Punjabi's wording.
///
/// # Errors
///
/// As [`pa_in::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{pa::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::INR);
/// assert_eq!(to_currency_with(1.01, &opts)?, "ਇੱਕ ਰੁਪਇਆ ਇੱਕ ਪੈਸਾ");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`pa_in::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    pa_in::to_currency_with(
        value,
        &pa_in::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `pa`.
pub static LANGUAGE: Language = Language {
    code: "pa",
    name: "Punjabi",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
