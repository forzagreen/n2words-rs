//! Bangla (`bn`): the bare-tag alias of [`bn_bd`](crate::bn_bd).
//!
//! `bn` resolves to `bn-BD`, currently the only Bangla variant the crate
//! implements; it is not a claim that `bn-BD` speaks for every Bangla-speaking
//! region. Cardinal and ordinal are `bn_bd`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{bn, Currency};
//!
//! assert_eq!(bn::to_cardinal(42)?, "বেয়াল্লিশ");
//! assert_eq!(bn::to_currency(42.5, Currency::BDT)?, "বেয়াল্লিশ টাকা পঞ্চাশ পয়সা");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::bn_bd;
use crate::currency::Currency;
use crate::currency::vocab::BN as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;

pub use crate::bn_bd::{CARDINAL_MAX, CURRENCY_MAX, ORDINAL_MAX, to_cardinal, to_ordinal};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "bn-BD";

options! {
    /// Options for [`to_currency_with`]: `bn_bd`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Bangla's wording.
///
/// # Errors
///
/// As [`bn_bd::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{bn::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::BDT);
/// assert_eq!(to_currency_with(1, &opts)?, "এক টাকা");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`bn_bd::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    bn_bd::to_currency_with(
        value,
        &bn_bd::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `bn`.
pub static LANGUAGE: Language = Language {
    code: "bn",
    name: "Bangla",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(plain to_cardinal, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
