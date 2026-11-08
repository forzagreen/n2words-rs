//! Kannada (`kn`): the bare-tag alias of [`kn_in`](crate::kn_in).
//!
//! `kn` resolves to `kn-IN`, currently the only Kannada variant the crate
//! implements; it is not a claim that `kn-IN` speaks for every
//! Kannada-speaking region. Cardinal and ordinal are `kn_in`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language
//! has no country to take a default currency from, so [`to_currency`] takes
//! the currency explicitly.
//!
//! ```
//! use n2words::{kn, Currency};
//!
//! assert_eq!(kn::to_cardinal(42)?, "ನಲವತ್ತೆರಡು");
//! assert_eq!(kn::to_currency(42.5, Currency::INR)?, "ನಲವತ್ತೆರಡು ರೂಪಾಯಿಗಳು ಐವತ್ತು ಪೈಸೆಗಳು");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::KN as CURRENCY_VOCAB;
use crate::kn_in;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::kn_in::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "kn-IN";

options! {
    /// Options for [`to_currency_with`]: `kn_in`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Kannada's wording.
///
/// # Errors
///
/// As [`kn_in::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{kn::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::INR);
/// assert_eq!(to_currency_with(1, &opts)?, "ಒಂದು ರೂಪಾಯಿ");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`kn_in::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    kn_in::to_currency_with(
        value,
        &kn_in::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `kn`.
pub static LANGUAGE: Language = Language {
    code: "kn",
    name: "Kannada",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
