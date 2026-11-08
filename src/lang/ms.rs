//! Malay (`ms`): the bare-tag alias of [`ms_my`](crate::ms_my).
//!
//! `ms` resolves to `ms-MY`, currently the only Malay variant this crate
//! implements; it is not a claim that `ms-MY` speaks for every Malay-speaking
//! region. Cardinal and ordinal are `ms_my`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{ms, Currency};
//!
//! assert_eq!(ms::to_cardinal(58)?, "lima puluh lapan");
//! assert_eq!(ms::to_ordinal(1000000)?, "kesejuta");
//! assert_eq!(ms::to_currency(42.5, Currency::MYR)?, "empat puluh dua ringgit lima puluh sen");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::MS as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::ms_my;
use crate::number::Number;
use crate::options::options;

pub use crate::ms_my::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ms-MY";

options! {
    /// Options for [`to_currency_with`]: `ms_my`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Malay's wording.
///
/// # Errors
///
/// As [`ms_my::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{ms::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::MYR);
/// assert_eq!(to_currency_with(1.01, &opts)?, "satu ringgit satu sen");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ms_my::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ms_my::to_currency_with(
        value,
        &ms_my::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `ms`.
pub static LANGUAGE: Language = Language {
    code: "ms",
    name: "Malay",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
