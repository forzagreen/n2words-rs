//! Azerbaijani (`az`): the bare-tag alias of [`az_az`](crate::az_az).
//!
//! `az` resolves to `az-AZ`, currently the only Azerbaijani variant this
//! crate implements; it is not a claim that `az-AZ` speaks for every
//! Azerbaijani-speaking region. Cardinal and ordinal are `az_az`'s own
//! functions. Currency is the one form a bare tag can't forward untouched: a
//! language has no country to take a default currency from, so
//! [`to_currency`] takes the currency explicitly.
//!
//! ```
//! use n2words::{az, Currency};
//!
//! assert_eq!(az::to_cardinal(42)?, "qırx iki");
//! assert_eq!(az::to_currency(42.5, Currency::AZN)?, "qırx iki manat əlli qəpik");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::az_az;
use crate::currency::Currency;
use crate::currency::vocab::AZ as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::az_az::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "az-AZ";

options! {
    /// Options for [`to_currency_with`]: `az_az`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Azerbaijani's wording.
///
/// # Errors
///
/// As [`az_az::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{az::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::AZN);
/// assert_eq!(to_currency_with(1.01, &opts)?, "bir manat bir qəpik");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`az_az::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    az_az::to_currency_with(
        value,
        &az_az::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `az`.
pub static LANGUAGE: Language = Language {
    code: "az",
    name: "Azerbaijani",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
