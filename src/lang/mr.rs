//! Marathi (`mr`): the bare-tag alias of [`mr_in`](crate::mr_in).
//!
//! `mr` resolves to `mr-IN`, currently the only Marathi variant the crate
//! implements; it is not a claim that `mr-IN` speaks for every
//! Marathi-speaking region. Cardinal and ordinal are `mr_in`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language
//! has no country to take a default currency from, so [`to_currency`] takes
//! the currency explicitly.
//!
//! ```
//! use n2words::{mr, Currency};
//!
//! assert_eq!(mr::to_cardinal(42)?, "बेचाळीस");
//! assert_eq!(mr::to_currency(42.5, Currency::INR)?, "बेचाळीस रुपये पन्नास पैसे");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::MR as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::mr_in;
use crate::number::Number;
use crate::options::options;

pub use crate::mr_in::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "mr-IN";

options! {
    /// Options for [`to_currency_with`]: `mr_in`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Marathi's wording.
///
/// # Errors
///
/// As [`mr_in::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{mr::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::INR);
/// assert_eq!(to_currency_with(1, &opts)?, "एक रुपया");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`mr_in::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    mr_in::to_currency_with(
        value,
        &mr_in::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `mr`.
pub static LANGUAGE: Language = Language {
    code: "mr",
    name: "Marathi",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
