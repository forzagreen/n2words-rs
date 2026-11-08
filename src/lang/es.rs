//! Spanish (`es`): the bare-tag alias of [`es_es`](crate::es_es).
//!
//! `es` has no single "correct" region, and the `es-*` variants genuinely
//! diverge in both cardinal ceiling and default currency (`es-ES`/EUR,
//! `es-MX`/MXN, `es-US`/USD). This entry point exists so the bare tag
//! resolves without forcing a region subtag anyway, defaulting to European
//! Spanish, the conventional default for bare `es` across most BCP 47
//! tooling. It does *not* speak for `es-MX` or `es-US`: a caller who cares
//! about Mexican or US Spanish should use that module explicitly.
//!
//! Cardinal and ordinal are `es_es`'s own functions. Currency is the one form
//! a bare tag can't forward untouched: a language has no country to take a
//! default currency from, so [`to_currency`] takes the currency explicitly.
//!
//! ```
//! use n2words::{es, Currency};
//!
//! assert_eq!(es::to_cardinal(21)?, "veintiuno");
//! assert_eq!(es::to_ordinal(21)?, "vigesimoprimero");
//! assert_eq!(es::to_currency(42.5, Currency::EUR)?, "cuarenta y dos euros con cincuenta céntimos");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::ES as CURRENCY_VOCAB;
use crate::es_es;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::es_es::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, OrdinalOptions, to_cardinal,
    to_cardinal_with, to_ordinal, to_ordinal_with,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "es-ES";

options! {
    /// Options for [`to_currency_with`]: `es_es`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
        /// Use "con" between the major and minor unit
        and: bool = true;
    }
}

/// Spells an amount in the given currency, with Spanish's default wording.
///
/// # Errors
///
/// As [`es_es::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{es::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions { and: false, ..CurrencyOptions::new(Currency::EUR) };
/// assert_eq!(to_currency_with(42.5, &opts)?, "cuarenta y dos euros cincuenta céntimos");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`es_es::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    es_es::to_currency_with(
        value,
        &es_es::CurrencyOptions {
            and: options.and,
            currency: options.currency,
        },
    )
}

/// Registry entry for `es`.
pub static LANGUAGE: Language = Language {
    code: "es",
    name: "Spanish",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(with_options to_ordinal_with, OrdinalOptions, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
