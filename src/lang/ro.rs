//! Romanian (`ro`): the bare-tag alias of [`ro_ro`](crate::ro_ro).
//!
//! `ro` resolves to `ro-RO`, currently the only Romanian variant this crate
//! implements. The alias exists so the bare tag works without a region
//! subtag; it is not a claim that `ro-RO` speaks for every Romanian-speaking
//! region. Cardinal and ordinal are `ro_ro`'s own functions. Currency is the
//! one form a bare tag can't forward untouched: a language has no country to
//! take a default currency from, so [`to_currency`] takes the currency
//! explicitly.
//!
//! ```
//! use n2words::{ro, Currency};
//!
//! assert_eq!(ro::to_cardinal(21)?, "douăzeci și unu");
//! assert_eq!(ro::to_ordinal(2)?, "al doilea");
//! assert_eq!(ro::to_currency(1, Currency::RON)?, "un leu");
//! # Ok::<(), n2words::Error>(())
//! ```

use crate::Result;
use crate::currency::Currency;
use crate::currency::vocab::RO as CURRENCY_VOCAB;
use crate::language::{Kind, Language, form};
use crate::number::Number;
use crate::options::options;
use crate::ro_ro;

pub use crate::ro_ro::{
    CARDINAL_MAX, CURRENCY_MAX, CardinalOptions, ORDINAL_MAX, to_cardinal, to_cardinal_with,
    to_ordinal,
};

/// The variant this alias resolves to.
pub const ALIAS_OF: &str = "ro-RO";

options! {
    /// Options for [`to_currency_with`]: `ro_ro`'s, with `currency` required.
    pub struct CurrencyOptions {
        required
        /// ISO 4217 currency code to name the amount in
        currency: Currency, table = CURRENCY_VOCAB;
    }
}

/// Spells an amount in the given currency, with Romanian's default wording.
///
/// # Errors
///
/// As [`ro_ro::to_currency_with`].
pub fn to_currency<'a>(value: impl Into<Number<'a>>, currency: Currency) -> Result<String> {
    to_currency_with(value, &CurrencyOptions::new(currency))
}

/// Spells an amount in the given currency, with options.
///
/// ```
/// # use n2words::{ro::{to_currency_with, CurrencyOptions}, Currency};
/// let opts = CurrencyOptions::new(Currency::RON);
/// assert_eq!(to_currency_with(2.5, &opts)?, "doi lei cincizeci de bani");
/// # Ok::<(), n2words::Error>(())
/// ```
///
/// # Errors
///
/// As [`ro_ro::to_currency_with`].
pub fn to_currency_with<'a>(
    value: impl Into<Number<'a>>,
    options: &CurrencyOptions,
) -> Result<String> {
    ro_ro::to_currency_with(
        value,
        &ro_ro::CurrencyOptions {
            currency: options.currency,
        },
    )
}

/// Registry entry for `ro`.
pub static LANGUAGE: Language = Language {
    code: "ro",
    name: "Romanian",
    kind: Kind::Alias { target: ALIAS_OF },
    cardinal: Some(form!(with_options to_cardinal_with, CardinalOptions, CARDINAL_MAX)),
    ordinal: Some(form!(plain to_ordinal, ORDINAL_MAX)),
    currency: Some(form!(with_options to_currency_with, CurrencyOptions, CURRENCY_MAX)),
};
