//! Danish (`da`): the bare-tag alias of [`da_dk`](crate::da_dk).
//!
//! `da` resolves to `da-DK`, currently the only Danish variant this crate
//! implements; it exists so the language can be named without a region
//! subtag, not as a claim that `da-DK` speaks for every Danish-speaking
//! region. Cardinal and ordinal are `da_dk`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{da, Currency};
//!
//! assert_eq!(da::to_cardinal(42)?, "toogfyrre");
//! assert_eq!(da::to_currency(42.5, Currency::DKK)?, "toogfyrre kroner og halvtreds øre");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::DA as CURRENCY_VOCAB;
use crate::da_dk;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::da_dk::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "da-DK";

options! {
    /// Options for [`to_currency_with`]: `da_dk`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Danish's default wording.
///
/// # Errors
///
/// As [`da_dk::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{da::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::DKK);
/// assert_eq!(to_currency_with(1.01, &opts)?, "en krone og et øre");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`da_dk::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    da_dk::to_currency_with(
        value,
        &da_dk::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `da`.
pub static LANGUAGE: Language = Language {
    code: "da",
    name: "Danish",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
