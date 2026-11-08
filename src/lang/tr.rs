//! Turkish (`tr`): the bare-tag alias of [`tr_tr`](crate::tr_tr).
//!
//! `tr` resolves to `tr-TR`, currently the only Turkish variant this crate
//! implements; it is not a claim that `tr-TR` speaks for every
//! Turkish-speaking region. Cardinal and ordinal are `tr_tr`'s own functions.
//! Currency is the one form a bare tag can't forward untouched: a language has
//! no country to take a default currency from, so [`to_currency`] takes the
//! currency explicitly.
//!
//! ```
//! use n2words::{tr, Currency};
//!
//! assert_eq!(tr::to_cardinal(21)?, "yirmi bir");
//! assert_eq!(tr::to_currency(42.5, Currency::TRY)?, "kırk iki lira elli kuruş");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::TR as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::tr_tr;

pub use crate::tr_tr::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "tr-TR";

options! {
    /// Options for [`to_currency_with`]: `tr_tr`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Turkish's wording.
///
/// # Errors
///
/// As [`tr_tr::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{tr::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::TRY);
/// assert_eq!(to_currency_with(1.01, &opts)?, "bir lira bir kuruş");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`tr_tr::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    tr_tr::to_currency_with(
        value,
        &tr_tr::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `tr`.
pub static LANGUAGE: Language = Language {
    code: "tr",
    name: "Turkish",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
