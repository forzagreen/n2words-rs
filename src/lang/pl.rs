//! Polish (`pl`): the bare-tag alias of [`pl_pl`](crate::pl_pl).
//!
//! `pl` resolves to `pl-PL`, currently the only Polish variant this crate
//! implements. The alias exists so the bare tag works without a region
//! subtag; it is not a claim that `pl-PL` speaks for every Polish-speaking
//! region. Cardinal and ordinal are `pl_pl`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{pl, Currency};
//!
//! assert_eq!(pl::to_cardinal(21)?, "dwadzieścia jeden");
//! assert_eq!(pl::to_currency(42.5, Currency::PLN)?, "czterdzieści dwa złote pięćdziesiąt groszy");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::PL as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::pl_pl;

pub use crate::pl_pl::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "pl-PL";

options! {
    /// Options for [`to_currency_with`]: `pl_pl`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Polish's default wording.
///
/// # Errors
///
/// As [`pl_pl::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{pl::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::PLN);
/// assert_eq!(to_currency_with(1, &opts)?, "jeden złoty");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`pl_pl::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    pl_pl::to_currency_with(
        value,
        &pl_pl::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `pl`.
pub static LANGUAGE: Language = Language {
    code: "pl",
    name: "Polish",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
