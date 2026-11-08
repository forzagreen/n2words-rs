//! Hindi (`hi`): the bare-tag alias of [`hi_in`](crate::hi_in).
//!
//! `hi` resolves to `hi-IN`, currently the only Hindi variant the crate
//! implements; it is not a claim that `hi-IN` speaks for every Hindi-speaking
//! region. Cardinal and ordinal are `hi_in`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{hi, Currency};
//!
//! assert_eq!(hi::to_cardinal(42)?, "बयालीस");
//! assert_eq!(hi::to_currency(42.5, Currency::INR)?, "बयालीस रुपये पचास पैसे");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::HI as CURRENCY_VOCAB;
use crate::hi_in;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::hi_in::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "hi-IN";

options! {
    /// Options for [`to_currency_with`]: `hi_in`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Hindi's wording.
///
/// # Errors
///
/// As [`hi_in::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{hi::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::INR);
/// assert_eq!(to_currency_with(1, &opts)?, "एक रुपया");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`hi_in::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    hi_in::to_currency_with(
        value,
        &hi_in::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `hi`.
pub static LANGUAGE: Language = Language {
    code: "hi",
    name: "Hindi",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
